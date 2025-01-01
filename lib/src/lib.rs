use serde::{Deserialize, Serialize};
use uint::construct_uint;
construct_uint! {
    // Construct as unsigned 256-bit integer
    // consisting of 4 x 64-bit words
    #[derive(Serialize, Deserialize)]
    pub struct U256(4);
}
pub mod sha256;
pub mod types;
pub mod util;
pub mod crypto;