use ethers::types::{U256, H160};
use hex_literal::hex;

pub const MAX_SQRT_PRICE: U256 = U256{0:[
    6743328256752651557,
    17280870778742802505,
    4294805859,
    0,
]};

pub const MIN_SQRT_PRICE: U256 = U256{0:[
    4295128740,
    0,
    0,
    0,
]};

pub const ZERO: U256 = U256{0:[0,0,0,0]};

pub const MAXLEN: usize = 3;

pub const UNIV2_FAC: H160 = H160(hex!("5c69bee701ef814a2b6a3edd4b1652cb9cc5aa6f"));
pub const SUSHI_FAC: H160 = H160(hex!("c0aee478e3658e2610c5f7a4a2e1777ce9e4f2ac"));
pub const SHIBA_FAC: H160 = H160(hex!("115934131916c8b277dd010ee02de363c09d037c"));
