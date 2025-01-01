use serde::{Deserialize, Serialize};
use uint::construct_uint;
construct_uint! {
    // Construct as unsigned 256-bit integer
    // consisting of 4 x 64-bit words
    #[derive(Serialize, Deserialize)]
    pub struct U256(4);
}

pub const INITIAL_REWARD: u64 = 50;
pub const HALVING_INTERVAL: u64 = 210;
pub const IDEAL_BLOCK_TIME: u64 = 10;
pub const MIN_TARGET: U256 = U256([
    0xFFFF_FFFF_FFFF_FFFF,
    0xFFFF_FFFF_FFFF_FFFF,
    0xFFFF_FFFF_FFFF_FFFF,
    0x0000_FFFF_FFFF_FFFF,
]);
pub const DIFICULTY_UPDATE_INTERVAL: u64 = 50;

pub mod sha256;
pub mod types;
pub mod util;
pub mod crypto;
mod error;