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
//use log::*;
use serde_derive::Deserialize;
use serde_json;
use solana_geyser_plugin_interface::geyser_plugin_interface::{
    GeyserPlugin, GeyserPluginError, ReplicaAccountInfoVersions, ReplicaBlockInfoVersions,
    ReplicaTransactionInfoVersions, SlotStatus,
};
use tokio::{runtime::Runtime, sync::oneshot};
use tonic::transport::Server;
//use crate::server::{GeyserService, GeyserServiceConfig};
use nano_geyser::nano_geyser::{
    nano_geyser_server::{
        NanoGeyserServer,
    },
    MerkleShredData,
    MerkleShredCode,
    BlockHeader,
    Shred,
};
#[derive(Debug)]
pub struct PluginData{
    runtime: Runtime,
    block_header_sender: Sender<BlockHeader>,
    shred_senders: Sender<Shred>,
}
pub type PluginResult = Result<(), GeyserPluginError>;

#[derive(Default, Debug)]
pub struct NanoGeyserPlugin {
    /// Initialized on initial plugin load.
    data: Option<PluginData>,
}


impl GeyserPlugin for NanoGeyserPlugin{
    fn name(&self) -> &'static str {
        "nano-geyser-plugin"
    }
    fn on_load(&mut self, _config_file: &str) -> PluginResult {
        Ok(())
    }
    fn on_unload(&mut self) {
        
    }

    fn update_account(
        &mut self,
        account: ReplicaAccountInfoVersions,
        slot: u64,
        is_startup: bool,
    ) -> PluginResult {
        Ok(())
    }

    fn notify_end_of_startup(&mut self) -> PluginResult {
        Ok(())
    }

    fn update_slot_status(
        &mut self,
        slot: u64,
        parent: Option<u64>,
        status: SlotStatus,
    ) -> PluginResult {
        Ok(())
    }

    fn notify_transaction(
        &mut self,
        transaction: ReplicaTransactionInfoVersions,
        slot: u64,
    ) -> PluginResult {
        Ok(())
    }

    fn notify_block_metadata(&mut self, blockinfo: ReplicaBlockInfoVersions) -> PluginResult {
        Ok(())
    }

    fn account_data_notifications_enabled(&self) -> bool {
        true
    }

    fn transaction_notifications_enabled(&self) -> bool {
        false
    }
}

