use solana_sdk::{pubkey::Pubkey, clock::Slot};
use nano_geyser::nano_geyser::{EntrySummary, EntryUpdate, TimestampedEntryNotification, SlotUpdate, TimestampedSlotUpdate,};

pub enum SlotUpdatestatus {
    Processed,
    Rooted,
    Confirmed,
}

impl From<nano_geyser::nano_geyser::SlotUpdateStatus> for SlotUpdatestatus {
    fn from(value: nano_geyser::nano_geyser::SlotUpdateStatus) -> Self {
        match value {
            nano_geyser::nano_geyser::SlotUpdateStatus::Processed => Self::Processed,
            nano_geyser::nano_geyser::SlotUpdateStatus::Rooted => Self::Rooted,
            nano_geyser::nano_geyser::SlotUpdateStatus::Confirmed => Self::Confirmed,            
        }
    }
}

pub struct Slotupdate {
    pub parent_slot: Option<Slot>,
    pub slot: Slot,
    pub status: SlotUpdatestatus,
}

impl From<nano_geyser::nano_geyser::SlotUpdate> for Slotupdate {
    fn from(value: nano_geyser::nano_geyser::SlotUpdate) -> Self {
        let status = nano_geyser::nano_geyser::SlotUpdateStatus::from_i32(value.status).unwrap();
        Self { parent_slot: value.parent_slot, slot: value.slot, status: SlotUpdatestatus::from(status)}
    }
}



pub struct EntrySumm{
   pub slot: u64,
   pub hash: Vec<u8>,
   pub num_transactions: u64,
}
impl From<nano_geyser::nano_geyser::EntrySummary> for EntrySumm{
    fn from(value: nano_geyser::nano_geyser::EntrySummary) -> Self {
        Self { slot: value.slot, hash: value.hash, num_transactions: value.num_transactions }
    }
}
pub struct Entryupdate{
    pub slot: u64,
    pub index: u64,
    pub summary: EntrySumm
}
impl From<nano_geyser::nano_geyser::EntryUpdate>  for Entryupdate{
    fn from(value: nano_geyser::nano_geyser::EntryUpdate) -> Self {
        Self { 
            slot: value.slot,
            index: value.index, 
            summary: EntrySumm::from(value.summary.unwrap())
        }
    }
}

