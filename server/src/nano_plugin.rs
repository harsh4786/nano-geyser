use std::{
    fs::File,
    io::Read,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::SystemTime, fmt,
};

use bs58;
use crossbeam_channel::{bounded, Sender, TrySendError, unbounded};
use log::*;
use serde_derive::Deserialize;
use serde_json;
use solana_geyser_plugin_interface::geyser_plugin_interface::{
    GeyserPlugin, GeyserPluginError, SlotStatus, ReplicaEntryInfoVersions,
    Result as PluginResult,
};
use tokio::{runtime::Runtime, sync::oneshot};
use tonic::transport::Server;
use nano_geyser::nano_geyser::{SlotUpdate, EntryUpdate, TimestampedEntryNotification, TimestampedSlotUpdate, SlotUpdateStatus};
use solana_ledger::entry_notifier_service::{EntryNotification};
use crate::server::{NanoGeyserService, GeyserServiceConfig};
use nano_geyser::nano_geyser::{
    nano_geyser_server::{
        NanoGeyserServer,
    },
    // MerkleShredData,
    // MerkleShredCode,
    // Shred,
};
#[derive(Debug)]
pub struct PluginData{
    runtime: Runtime,
    server_exit_sender: oneshot::Sender<()>,
   // block_header_sender: Sender<BlockHeader>,
    entry_sender: Sender<TimestampedEntryNotification>,
    slot_update_sender: Sender<TimestampedSlotUpdate>,
    highest_write_slot: Arc<AtomicU64>,
}

#[derive(Default, Debug)]
pub struct NanoGeyserPlugin {
    /// Initialized on initial plugin load.
    data: Option<PluginData>,
    
}
//  impl fmt::Debug for NanoGeyserPlugin {
//     fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result{
//         Ok(())
//     }
//  }
#[derive(Clone, Debug, Deserialize)]
pub struct NanoConfig{
    geyser_config: GeyserServiceConfig,
    bind_address: String,
    entry_update_buffer_size: usize,
    slot_update_buffer_size: usize,
}

// #[derive(Clone, Debug, Deserialize)]
// pub struct GeyserServiceConfig {
//     /// Cadence of heartbeats.
//     heartbeat_interval_ms: u64,

//     /// Individual subscriber buffer size.
//     subscriber_buffer_size: usize,
// }
impl GeyserPlugin for NanoGeyserPlugin{
    fn name(&self) -> &'static str {
        "nano-geyser-plugin"
    }
    fn on_load(&mut self, config_path: &str) -> PluginResult<()> {
        solana_logger::setup_with_default("info");
        info!(
            "Loading plugin {:?} from config_path {:?}",
            self.name(),
            config_path
        );
        let mut file = File::open(config_path)?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)?;
        let config: NanoConfig =
            serde_json::from_str(&buf).map_err(|err| GeyserPluginError::ConfigFileReadError {
                msg: format!("Error deserializing PluginConfig: {err:?}"),
            })?;

        let addr = config.bind_address.parse().map_err(|err| GeyserPluginError::ConfigFileReadError { msg: format!("Error parsing bind address") })?;
        let highest_write_slot = Arc::new(AtomicU64::new(0));
        let (slot_update_sender, slot_update_receiver) = bounded(config.slot_update_buffer_size);
        let (entry_update_sender, entry_update_receiver) = bounded(config.entry_update_buffer_size);
        let svc = NanoGeyserService::new(
            config.geyser_config,
            highest_write_slot.clone(),
            slot_update_receiver,
            entry_update_receiver
        );
        let server = NanoGeyserServer::new(svc);
        let (server_exit_tx, server_exit_rx) = oneshot::channel();
        let runtime = Runtime::new().unwrap();
        runtime.spawn(Server::builder().add_service(server).serve_with_shutdown(addr, async move{
            let _ = server_exit_rx.await;
        }));
        self.data = Some(
            PluginData { runtime, server_exit_sender: server_exit_tx, entry_sender: entry_update_sender,  slot_update_sender, highest_write_slot }
        );
        Ok(())
    }
    fn on_unload(&mut self) {
        
    }

    // fn update_account(
    //     &mut self,
    //     account: ReplicaAccountInfoVersions,
    //     slot: u64,
    //     is_startup: bool,
    // ) -> PluginResult {
    //     Ok(())
    // }

    fn notify_end_of_startup(&self) -> PluginResult<()> {
        Ok(())
    }

    fn update_slot_status(
        &self,
        slot: u64,
        parent: Option<u64>,
        status: SlotStatus,
    ) -> PluginResult<()>{
        let data = self.data.as_ref().expect("plugin must be initialized");
        debug!("Updating slot {:?} at with status {:?}", slot, status);

        let status = match status{
            SlotStatus::Processed => SlotUpdateStatus::Processed,
            SlotStatus::Rooted => SlotUpdateStatus::Rooted,
            SlotStatus::Confirmed => SlotUpdateStatus::Confirmed,
        };
        match data.slot_update_sender.try_send(TimestampedSlotUpdate { 
            ts: Some(prost_types::Timestamp::from(SystemTime::now())),
            update: Some(SlotUpdate{
                 slot,
                 parent_slot: parent,
                 status: status as i32,
            })
         }) {
            Ok(_) => Ok(()),
            Err(TrySendError::Full(_)) => {
                warn!("channel queue is full");
                Ok(())
            }
            Err(TrySendError::Disconnected(_))=> {
                error!("slot send error");
                Err(GeyserPluginError::SlotStatusUpdateError {
                    msg: "slot_update channel disconnected, exiting".to_string(),
                })
            }

        }
    }
    fn notify_entry(&self, entry: ReplicaEntryInfoVersions) -> PluginResult<()> {
        Ok(())
    }

    // fn notify_transaction(
    //     &mut self,
    //     transaction: ReplicaTransactionInfoVersions,
    //     slot: u64,
    // ) -> PluginResult {
    //     Ok(())
    // }

    // fn notify_block_metadata(&mut self, blockinfo: ReplicaBlockInfoVersions) -> PluginResult {
    //     Ok(())
    // }

    // fn account_data_notifications_enabled(&self) -> bool {
    //     false
    // }
    fn entry_notifications_enabled(&self) -> bool {
        true
    }

}


#[no_mangle]
#[allow(improper_ctypes_definitions)]
/// # Safety
///
/// This function returns the Plugin pointer as trait GeyserPlugin.
pub unsafe extern "C" fn _create_plugin() -> *mut dyn GeyserPlugin {
    let plugin = NanoGeyserPlugin::default();
    let plugin: Box<dyn GeyserPlugin> = Box::new(plugin);
    Box::into_raw(plugin)
}