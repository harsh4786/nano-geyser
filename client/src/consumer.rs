use std::{collections::{
    HashMap,

}, sync::{atomic::AtomicBool, Arc}};

use nano_geyser::nano_geyser::{EntrySummary, EntryUpdate, 
    SubscribeEntryUpdateRequest, TimestampedEntryNotification,
    nano_geyser_client::NanoGeyserClient
};
//use log::*;
use lru::LruCache;
use thiserror::Error;
use tokio::{
    sync::mpsc::UnboundedSender,
    time::{interval, Instant},
};
use tonic::{codegen::InterceptedService, transport::Channel, Response, Status};
use crate::{
    types::{Entryupdate, EntrySumm},
    interceptor::GrpcInterceptor,
};


#[derive(Clone)]
pub struct GeyserConsumer {
    /// Geyser client.
    client: NanoGeyserClient<InterceptedService<Channel, GrpcInterceptor>>,

    /// Exit signal.
    exit: Arc<AtomicBool>,
}

impl GeyserConsumer{
    pub fn new(
        client: NanoGeyserClient<InterceptedService<Channel, GrpcInterceptor>>,
        exit: Arc<AtomicBool>,
    ) -> Self {
        Self { client, exit }
    }

    pub async fn consume_entry_updates(
        &self,
        entry_updates_tx: UnboundedSender<Entryupdate>,

    ) -> Result<()>{
        let mut client = self.client.clone();
        let resp = client.subscribe_entry_updates(SubscribeEntryUpdateRequest{}).await?;
        let mut stream = resp.into_inner();


        Ok(())
    }
}