use solana_sdk::{pubkey::Pubkey, clock::Slot};




pub enum SlotUpdateStatus {
    Processed,
    Rooted,
    Confirmed,
}