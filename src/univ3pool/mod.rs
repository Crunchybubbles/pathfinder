use serde::{Serialize, Deserialize};
use ethers::types::{Address, U256, I256};
use ethers::providers::{Provider, Http};
use ethers::contract::abigen;
use std::sync::Arc;
use crate::constants::{ZERO, MIN_SQRT_PRICE, MAX_SQRT_PRICE};

abigen!(UniV3Calc, "UniV3Calc.json");

#[derive(Debug, Deserialize)]
struct _Token {
    id: String,
    symbol: String,
    decimals: String,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
struct _Tick {
    tickIdx: String,
    liquidityNet: String,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
struct _UniV3Pool {
    id: String,
    token0: _Token,
    token1: _Token,
    feeTier: String,
    tick: String,
    ticks: Vec<_Tick>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]    
pub struct Tick {
    tick_idx: i64,
    liquidity_net: U256,
}
#[derive(Debug, Serialize, Deserialize, Clone)]    
pub struct Token {
    pub id: Address,
    pub symbol: String,
    pub decimals: u64,
}
#[derive(Debug, Serialize, Deserialize, Clone)]    
pub struct UniV3Pool {
    pub id: Address,
    pub token0: Token,
    pub token1: Token,
    pub fee: u64,
    pub tick: i64,
    pub ticks: Vec<Tick>,
}

impl UniV3Pool {
    fn from_univ3pool(pool: _UniV3Pool) -> UniV3Pool {
	let id: Address = pool.id.parse::<Address>().unwrap();
	let token0_id: Address = pool.token0.id.parse::<Address>().unwrap();
	let token0_symbol: String = pool.token0.symbol;
	let token0_decimals: u64 = pool.token0.decimals.parse::<u64>().unwrap();
	let token0 = Token {id: token0_id, symbol: token0_symbol, decimals: token0_decimals};

	let token1_id: Address = pool.token1.id.parse::<Address>().unwrap();
	let token1_symbol: String = pool.token1.symbol;
	let token1_decimals: u64 = pool.token1.decimals.parse::<u64>().unwrap();
	let token1 = Token {id: token1_id, symbol: token1_symbol, decimals: token1_decimals};

	let fee: u64 = pool.feeTier.parse::<u64>().unwrap();
	let tick: i64 = pool.tick.parse::<i64>().unwrap();
	let mut ticks: Vec<Tick> = Vec::new();
	for t in pool.ticks {
	    let tick_idx: i64 = t.tickIdx.parse::<i64>().unwrap();
	    let liquidity_net: U256 = t.liquidityNet.parse::<U256>().unwrap();
	    ticks.push(Tick{tick_idx, liquidity_net});
	}
	UniV3Pool{id,token0,token1,fee,tick,ticks}

	
    }

    pub fn token_out(&self, token_in: &Address) -> &Address {
	if token_in == &self.token0.id {
	    return &self.token1.id;
	} else {
	    return &self.token0.id;
	}
    }

    fn zf1(&self, token_in: &Address) -> bool {
	if *token_in == self.token0.id {
	    return true;
	} else {
	    return false;
	}
	
    }

    pub async fn amount_out(&self, token_in: &Address, amount_in: U256, swap_calc_contract: Arc<UniV3Calc<Provider<Http>>>) -> U256 {

	let zero_for_one = self.zf1(token_in);
	let spl = sqrt_price_limit(zero_for_one);
	let amount: I256 = I256::try_from(amount_in).unwrap();
	    
	let r = swap_calc_contract.calc_v_3_swap(self.id, zero_for_one, amount, spl).call().await;
	match r {
	    Ok(d) => {
		if d.0.is_positive() {
		    return U256::try_from(d.1.abs()).unwrap();
		} else {
		    return U256::try_from(d.0.abs()).unwrap();
		}
	    }
	    Err(e) => {
		println!("{}", e);
		return ZERO;
	    }
	}
    }

    pub async fn amount_in_for_out(&self, direction: bool, amount_out: U256, swap_calc_contract: Arc<UniV3Calc<Provider<Http>>>) -> U256 {
	let spl = sqrt_price_limit(direction);
	let amount: I256 = I256::try_from(amount_out).unwrap() * I256::minus_one();
	
	let r = swap_calc_contract.calc_v_3_swap(self.id, direction, amount, spl).call().await;
	match r {
	    Ok(d) => {
		if d.0.is_positive() {
		    return U256::try_from(d.0.abs()).unwrap();
		} else {
		    return U256::try_from(d.1.abs()).unwrap();
		}
	    }
	    Err(e) => {
		println!("{}", e);
		return ZERO;
	    }
	}
    }
}


fn sqrt_price_limit(zf1: bool) -> U256 {
    if zf1 {
	return MIN_SQRT_PRICE;
    } else {
	return MAX_SQRT_PRICE;
    }
}

pub fn uni3() -> std::io::Result<Vec<UniV3Pool>> {
    let mut pools: Vec<UniV3Pool> = Vec::new();
    let uni3: Vec<_UniV3Pool> = serde_json::from_str(&std::fs::read_to_string("uni3data.txt")?)?;
    for pool in uni3 {
	pools.push(UniV3Pool::from_univ3pool(pool));
    }
    return Ok(pools);
}


