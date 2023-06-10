use std::{
    collections::{HashMap, HashSet},
    fmt::{Debug, Display, Formatter},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    thread::{Builder, JoinHandle},
    time::{Duration, Instant},
};
use crossbeam_channel::{tick, unbounded, Receiver, RecvError, Sender};
use log::*;
use nano_geyser::nano_geyser::{SlotUpdate, TimestampedEntryNotification, 
    TimestampedSlotUpdate, SubscribeEntryUpdateRequest, SubscribeSlotUpdateRequest,
    nano_geyser_server::NanoGeyser,
};
use once_cell::sync::OnceCell;
use serde_derive::Deserialize;
use thiserror::Error;
use tokio::sync::mpsc::{channel, error::TrySendError as TokioTrySendError, Sender as TokioSender};
use tonic::{metadata::MetadataValue, Request, Response, Status};
use uuid::Uuid;

use crate::{nano_plugin::NanoConfig, subscription_stream::SubscriptionStream};



#[derive(Clone)]
struct SubscriptionClosedSender {
    inner: Sender<SubscriptionClosedEvent>,
}

struct SlotUpdateSubscription {
    subscription_tx: SlotUpdateSender,
}

struct EntryUpdateSubscription { 
    subscription_tx: EntryUpdateSender,
}

type SlotUpdateSender = TokioSender<Result<TimestampedSlotUpdate, Status>>;
type EntryUpdateSender = TokioSender<Result<TimestampedEntryNotification, Status>>;



#[derive(Error, Debug)]
pub enum GeyserServiceError {
    #[error("GeyserStreamMessageError")]
    GeyserStreamMessageError(#[from] RecvError),

    #[error("The receiving side of the channel is full")]
    NotificationReceiverFull,

    #[error("The receiver is disconnected")]
    NotificationReceiverDisconnected,
}

type GeyserServiceResult<T> = Result<T, GeyserServiceError>;
#[allow(clippy::enum_variant_names)]
enum SubscriptionAddedEvent {
    SlotUpdateSubscription {
        uuid: Uuid,
        notification_sender: SlotUpdateSender,
    },
    EntryUpdateSubscription{
        uuid: Uuid,
        notification_sender: EntryUpdateSender,
    }
}

pub trait StreamClosedSender<E: Send + 'static>: Send + 'static {
    type Error: Display;
    fn send(&self, event: E) -> Result<(), Self::Error>;
}

impl StreamClosedSender<SubscriptionClosedEvent> for SubscriptionClosedSender {
    type Error = crossbeam_channel::TrySendError<SubscriptionClosedEvent>;

    fn send(&self, event: SubscriptionClosedEvent) -> Result<(), Self::Error> {
        self.inner.try_send(event)
    }
}


// trait HeartbeatStreamer {
//     fn send_heartbeat(&self) -> GeyserServiceResult<()>;
// }

trait ErrorStatusStreamer{
    fn stream_error(&self, status: Status) -> GeyserServiceResult<()>;
}

impl ErrorStatusStreamer for SlotUpdateSubscription{
    fn stream_error(&self, status: Status) -> GeyserServiceResult<()> {
        self.subscription_tx.try_send(Err(status)).map_err(|e| match e {
            TokioTrySendError::Full(_) => GeyserServiceError::NotificationReceiverFull,
            TokioTrySendError::Closed(_) => {GeyserServiceError::NotificationReceiverDisconnected}
        })
    }
}

impl ErrorStatusStreamer for EntryUpdateSubscription{
    fn stream_error(&self, status: Status) -> GeyserServiceResult<()> {
        self.subscription_tx.try_send(Err(status)).map_err(|e| match e {
            TokioTrySendError::Full(_) => GeyserServiceError::NotificationReceiverFull,
            TokioTrySendError::Closed(_) => {GeyserServiceError::NotificationReceiverDisconnected}
        })
    }
}



static VOTE_PROGRAM_ID: OnceCell<Vec<u8>> = OnceCell::new();
pub const HIGHEST_WRITE_SLOT_HEADER: &str = "highest-write-slot";
pub struct NanoGeyserService{
    highest_write_Slot: Arc<AtomicU64>,
    service_config: GeyserServiceConfig,
    subscription_added_tx: Sender<SubscriptionAddedEvent>,
    subscription_closed_sender: SubscriptionClosedSender,
    t_hdl: JoinHandle<()>,
}
impl NanoGeyserService{
    pub fn new(
        service_config: NanoGeyserService,
        highest_write_Slot: Arc<AtomicU64>,
        slot_update_rx: Receiver<TimestampedSlotUpdate>,
        entry_update_rx: Receiver<TimestampedEntryNotification>,
    ) -> Self{
        let (subscription_added_tx, subscription_added_rx) = unbounded();
        let (subscription_closed_tx, subscription_closed_rx) = unbounded();
        let t_hdl = Self::event_loop(slot_update_rx, entry_update_rx, subscription_added_rx, subscription_closed_rx);
        Self { 
            highest_write_Slot, 
            service_config, 
            subscription_added_tx, 
            subscription_closed_sender: subscription_closed_tx, 
            t_hdl 
        }
        
    }

    fn handle_subscription_closed(
        maybe_subscription_closed: Result<SubscriptionClosedEvent, RecvError>,
        // account_update_subscriptions: &mut HashMap<Uuid, AccountUpdateSubscription>,
        // partial_account_update_subscriptions: &mut HashMap<Uuid, PartialAccountUpdateSubscription>,
        slot_update_subscriptions: &mut HashMap<Uuid, SlotUpdateSubscription>,
        entry_update_subscriptions: &mut HashMap<Uuid, EntryUpdateSubscription>,
        // program_update_subscriptions: &mut HashMap<Uuid, AccountUpdateSubscription>,
        // transaction_update_subscriptions: &mut HashMap<Uuid, TransactionUpdateSubscription>,
        // block_update_subscriptions: &mut HashMap<Uuid, BlockUpdateSubscription>,
    ) -> GeyserServiceResult<()> {
        let subscription_closed = maybe_subscription_closed?;
        info!("closing subscription: {:?}", subscription_closed);
    
        match subscription_closed {
            SubscriptionClosedEvent::SlotUpdateSubscription(subscription_id) => {
                let _ = slot_update_subscriptions.remove(&subscription_id);
            }
            SubscriptionClosedEvent::EntryUpdateSubscription(entry_id) => {
                let _ = entry_update_subscriptions.remove(&entry_id);
            }
        }
    
        Ok(())
    }
    fn event_loop(
        slot_update_rx: Receiver<TimestampedSlotUpdate>,
        entry_update_rx: Receiver<TimestampedEntryNotification>,
        subscription_added_rx: Receiver<SubscriptionAddedEvent>,
        subscription_closed_rx: Receiver<SubscriptionClosedEvent>,
    ) -> JoinHandle<()> {
        Builder::new()
        .name("nano-geyser-event-loop".to_string())
        .spawn(move || {
            info!("Starting event loop");
            let mut slot_update_subscriptions : HashMap<Uuid, SlotUpdateSubscription> = HashMap::new();
            let mut entry_update_subscriptions: HashMap<Uuid, EntryUpdateSubscription> = HashMap::new();
    
            loop{
                crossbeam_channel::select! {
                    recv(subscription_added_rx) -> maybe_subscription_added => {
                        info!("Received new subscription");
                        if let Err(e) = Self::handle_subscription_added(maybe_subscription_added, &mut slot_update_subscriptions,&mut entry_update_subscriptions){
                            error!("error adding new subscription {}", e);
                            return;
                        }
                    }
                    recv(slot_update_rx) -> maybe_slot_update => {
                        info!("Received slot update subscription");
                        if let Err(e) = Self::handle_slot_update_event(maybe_slot_update, &slot_update_subscriptions) {
                            error!("error handling slot update events {}", e);
                            return;
                        }
                    }
                    recv(entry_update_rx) -> maybe_entry_update =>{
                        info!("Received entry update subscription");
                        if let Err(e) = Self::handle_entry_update_event(maybe_entry_update, &entry_update_subscriptions){
                            error!("error handling entry update events {}", e);
                            return;
                        }
                    }
                    recv(subscription_closed_rx) -> maybe_subscription_closed => {
                        info!("received closed event");
                        if let Err(e) = Self::handle_subscription_closed(maybe_subscription_closed, &mut slot_update_subscriptions, &mut entry_update_subscriptions ){
                            error!("error handling closed event: {}", e);
                            return;
                        }
                    }
                }
            }
    
        })
    }

    fn handle_entry_update_event(
        maybe_entry_update: Result<TimestampedEntryNotification, RecvError>,
        entry_update_subscriptions: HashMap<Uuid, EntryUpdateSubscription> 
    ) -> GeyserServiceResult<Vec<Uuid>>{
         let entry_update = maybe_entry_update?;
         let failed_entry_updates = entry_update_subscriptions.iter().filter_map(|(uuid, sub)|{
            if matches!(
                sub.subscription_tx.try_send(Ok(entry_update.clone())),
                Err(TokioTrySendError::Closed(_))
            ) {
                Some(*uuid)
            }
            else{
                None
            }
    
         }).collect();
    
         Ok(failed_entry_updates)
    
    }
    fn handle_slot_update_event(
        maybe_slot_update: Result<TimestampedSlotUpdate, RecvError>,
        slot_update_subscriptions: &HashMap<Uuid, SlotUpdateSubscription>
    ) -> GeyserServiceResult<Vec<Uuid>>{
        let slot_update = maybe_slot_update?;
        let failed_subscription_ids = slot_update_subscriptions.iter().filter_map(|(uuid, sub)| {
            if matches!(sub.subscription_tx.try_send(Ok(slot_update.clone())), Err(TokioTrySendError::Closed(_))){
                Some(*uuid)
            }
            else{
                None
            }
        }).collect();
        Ok(failed_subscription_ids)
    }
    fn handle_subscription_added(
        maybe_subscription_added: Result<SubscriptionAddedEvent, RecvError>,
        slot_update_subscriptions: &mut HashMap<Uuid, SlotUpdateSubscription>,
        entry_update_subscriptions: &mut HashMap<Uuid, EntryUpdateSubscription>,
    ) -> GeyserServiceResult<()>{
        let subscription_added = maybe_subscription_added?;
        info!("Subscription added");
        match subscription_added {
            SubscriptionAddedEvent::SlotUpdateSubscription { uuid, notification_sender: subscription_tx } => {
                slot_update_subscriptions.insert(uuid, SlotUpdateSubscription { subscription_tx });
            }
            SubscriptionAddedEvent::EntryUpdateSubscription { uuid, notification_sender: subscription_tx } => {
                entry_update_subscriptions.insert(uuid, EntryUpdateSubscription { subscription_tx });
            }
        }
        Ok(())
    }
    pub fn join(self) {
        self.t_hdl.join().unwrap();
    }
    

}

#[derive(Debug)]
enum SubscriptionClosedEvent { 
    SlotupdateSubscription(Uuid),
    EntryUpdateSubscription(Uuid)
}

pub struct GeyserServiceConfig{
    //maybe add heartbeats?
    subscriber_buffer_size: usize,
}


#[tonic::async_trait]
impl NanoGeyser for NanoGeyserService{
    type SubscribeSlotUpdatesStream = SubscriptionStream<Uuid, TimestampedSlotUpdate>;
    async fn subscribe_slot_updates(&self,request: Request<SubscribeSlotUpdateRequest> ) -> Result<Response<Self::SubscribeSlotUpdatesStream>, Status>{

    }
    type SubscribeEntryUpdatesStream = SubscriptionStream<Uuid, TimestampedEntryNotification>;
    async fn subscribe_entry_updates(&self,request: Request<SubscribeEntryUpdateRequest>) -> Result<Response<Self::SubscribeEntryUpdatesStream>, Status>{}

}







