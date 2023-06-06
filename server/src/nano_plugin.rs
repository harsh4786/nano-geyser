use std::{
    fs::File,
    io::Read,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::SystemTime,
};

use bs58;
use crossbeam_channel::{bounded, Sender, TrySendError};
use log::*;
use serde_derive::Deserialize;
use serde_json;
use solana_geyser_plugin_interface::geyser_plugin_interface::{
    GeyserPlugin, GeyserPluginError, ReplicaAccountInfoVersions, ReplicaBlockInfoVersions,
    ReplicaTransactionInfoVersions, SlotStatus,
};
use tokio::{runtime::Runtime, sync::oneshot};
use tonic::transport::Server;
use nano_geyser::nano_geyser::SlotUpdate;
use solana_ledger::entry_notifier_service::{EntryNotifier, EntryNotifcation};
use crate::server::{GeyserService, GeyserServiceConfig};
use crate::types::{SlotUpdateStatus};
use nano_geyser::nano_geyser::{
    nano_geyser_server::{
        NanoGeyserServer,
    },

    MerkleShredData,
    MerkleShredCode,
    Shred,
};
#[derive(Debug)]
pub struct PluginData{
    runtime: Runtime,
    server_exit_sender: oneshot::Sender<()>,
   // block_header_sender: Sender<BlockHeader>,
    entry_sender: Sender<EntryNotification>,
    slot_update_sender: Sender<SlotUpdate>,
    highest_write_slot: Arc<AtomicU64>,
}
pub type PluginResult = Result<(), GeyserPluginError>;

#[derive(Default, Debug)]
pub struct NanoGeyserPlugin {
    /// Initialized on initial plugin load.
    data: Option<PluginData>,
    
}
#[derive(Clone, Debug, Deserialize)]
pub struct NanoConfig{
    geyser_config: GeyserServiceConfig,
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
    fn on_load(&mut self, _config_file: &str) -> PluginResult {
        solana_logger::setup_with_default("info");
        info!(
            "Loading plugin {:?} from config_path {:?}",
            self.name(),
            config_path
        );
        let mut buf = String::new();
        

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

    fn notify_end_of_startup(&mut self) -> PluginResult {
        Ok(())
    }

    fn update_slot_status(
        &mut self,
        slot: u64,
        parent: Option<u64>,
        status: SlotStatus,
    ) -> PluginResult {
        let data = self.data.as_ref().expect("plugin must be initialized");
        debug!("Updating slot {:?} at with status {:?}", slot, status);

        let status = match status{
            SlotStatus::Processed => SlotUpdateStatus::Processed,
            SlotStatus::Rooted => SlotUpdateStatus::Rooted,
            SlotStatus::Confirmed => SlotUpdateStatus::Confirmed,
        };
        match data.slot_update_sender.try_send(SlotUpdate { slot, parent_slot, status: status as i32}) {
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
    fn notify_entry(&self, entry: ReplicaEntryInfoVersions) -> Result<()> {
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


