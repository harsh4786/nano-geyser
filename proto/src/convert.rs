// this module will convert the protobuf types to rust types and vice versa

use std::{
    convert::{TryFrom, TryInto},
    str::FromStr,
};
use solana_ledger::{
    shred::{
        ShredCommonHeader,
        ShredFlags,
        merkle::{ShredCode, ShredData}, ShredVariant, Shred
    },
    
};
use solana_sdk::{
    hash::Hash,
    pubkey::Pubkey,
    signature::Signature,
    transaction::{Transaction, TransactionError, VersionedTransaction},
};

use nano_geyser::MerkleShredCode;

use crate::nano_geyser;

use self::nano_geyser::MerkleShredData;



// impl From<ShredVariant> for nano_geyser::shred_variant::Shred{
//     fn from(value: ShredVariant) -> Self {
//       match value {
//           ShredVariant::LegacyCode  => nano_geyser::shred_variant::Shred::Legacycode( ShredCode {

//           }),
//           ShredVariant::MerkleCode(s) => nano_geyser::ShredVariant::MerkleCode,
//           ShredVariant::MerkleData(s) => nano_geyser::ShredVariant::MerkleData,
//       }
//     }
// }


// impl From<ShredCommonHeader> for nano_geyser::ShredCommonHeader{
//     fn from(value: ShredCommonHeader) -> Self {
//         Self { 
//             //fucking private fields................................................................
//             // update: fixed that
//             signature: value.signature.to_string(),
//              variant: nano_geyser::ShredVariant::from(value.shred_variant), 
//              slot: value.slot, 
//              index: value.index, 
//              version: value.version as u32, 
//              fec_set_index: value.fec_set_index
//         }
//     }
// }
// impl From<ShredCode> for MerkleShredCode{
//     fn from(value: ShredCode) -> Self {
//         Self { 
//             header: Some(nano_geyser::ShredCommonHeader::from(value.common_header)), 
//             c_header: value.coding_header, 
//             payload: value.payload, 
//         }
//     }
// }

// impl From<ShredData> for MerkleShredData{
//     fn from(value: ShredData) -> Self {
//         Self{
//             header: value.common_header, 
//             d_header: value.data_header,
//             payload: value.payload, 
//         }   
//     }
// }





