use ethers::types::U256;

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

pub const MAXLEN: usize = 2;
