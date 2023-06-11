use solana_sdk::{pubkey::Pubkey, clock::Slot};
use nano_geyser::nano_geyser::{EntrySummary, EntryUpdate, TimestampedEntryNotification, SlotUpdate, TimestampedSlotUpdate,};

pub enum SlotUpdateStatus {
    Processed,
    Rooted,
    Confirmed,
}
impl From<nano_geyser::nano_geyser::SlotUpdateStatus> for SlotUpdateStatus {
    fn from(value: nano_geyser::nano_geyser::SlotUpdateStatus) -> Self {
        match value {
            nano_geyser::nano_geyser::SlotUpdateStatus::Processed => Self::Processed,
            nano_geyser::nano_geyser::SlotUpdateStatus::Rooted => Self::Rooted,
            nano_geyser::nano_geyser::SlotUpdateStatus::Confirmed => Self::Confirmed,            
        }
    }
}

