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
    ReplicaTransactionInfoVersions, Result as PluginResult, SlotStatus,
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

pub struct PluginData{
    runtime: Runtime,
    block_header_sender: Sender<BlockHeader>,
    shred_senders: Sender<Shred>,
}


#[derive(Default)]
pub struct NanoGeyserPlugin {
    /// Initialized on initial plugin load.
    data: Option<PluginData>,
}


impl GeyserPlugin for NanoGeyserPlugin{
    fn name(&self) -> &'static str {
        "nano-geyser-plugin"
    }
    fn on_load(&mut self, _config_file: &str) -> PluginResult<()> {
        
    }
    fn on_unload(&mut self) {
        
    }
}

