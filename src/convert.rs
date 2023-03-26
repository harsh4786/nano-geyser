// this module will convert the protobuf types to rust types and vice versa

use std::{
    convert::{TryFrom, TryInto},
    str::FromStr,
};
use solana_ledger::{
    shred::{
        ShredCommonHeader,
        ShredFlags,
        merkle::{ShredCode, ShredData}
    },
    
};
use solana_sdk::{
    hash::Hash,
    pubkey::Pubkey,
    signature::Signature,
    transaction::{Transaction, TransactionError, VersionedTransaction},
};

use nano_geyser::MerkleShredCode;

use self::nano_geyser::MerkleShredData;
pub mod nano_geyser{
    tonic::include_proto!("nano_geyser");
}


impl From<ShredCommonHeader> for nano_geyser::ShredCommonHeader{
    fn from(value: ShredCommonHeader) -> Self {
        Self { 
            //fucking private fields................................................................
            // update: fixed that
            signature: value.signature,
             variant: value.shred_variant, 
             slot: value.slot, 
             index: value.index, 
             version: value.version, 
             fec_set_index: value.fec_set_index
        }
    }
}
impl From<ShredCode> for MerkleShredCode{
    fn from(value: ShredCode) -> Self {
        Self { 
            header: value.common_header, 
            c_header: value.coding_header, 
            payload: value.payload, 
        }
    }
}

impl From<ShredData> for MerkleShredData{
    fn from(value: ShredData) -> Self {
        Self{
            header: value.common_header, 
            d_header: value.data_header,
            payload: value.payload, 
        }   
    }
}





