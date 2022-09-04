use serde::{Serialize, Deserialize};
use ethers::types::{U256, Address};
use ethers::contract::abigen;
use std::sync::Arc;


use ethers::providers::{Provider, Http};
use crate::constants::ZERO;

abigen!(UniV2Calc, "/Users/jasper/Betelgeuse/build/contracts/univ2calc.json");



#[derive(Debug, Deserialize)]
struct _TokenV2 {
    id: String,
}
#[derive(Debug, Serialize, Deserialize, Clone)]    
pub struct TokenV2 {
    pub id: Address,
}

#[derive(Debug, Deserialize)]
struct _UniV2Pool {
    id: String,
    token0: _TokenV2,
    token1: _TokenV2,  
}

#[derive(Debug, Serialize, Deserialize, Clone)]    
pub struct UniV2Pool {
    pub id: Address,
    pub token0: TokenV2,
    pub token1: TokenV2,
}

impl UniV2Pool {
    fn from_univ2pool(pool: _UniV2Pool) -> UniV2Pool {
	let id: Address = pool.id.parse::<Address>().unwrap();
	let token0_id: Address = pool.token0.id.parse::<Address>().unwrap();
		
	let token0 = TokenV2 {id: token0_id};

	let token1_id: Address = pool.token1.id.parse::<Address>().unwrap();
		
	let token1 = TokenV2 {id: token1_id};
	UniV2Pool{id, token0, token1}

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

    pub async fn amount_out(&self, token_in: &Address, amount: U256, swap_calc_contract: Arc<UniV2Calc<Provider<Http>>>) -> U256 {
	let zero_for_one = self.zf1(token_in);
	let r = swap_calc_contract.calc_univ_2_amount_out(self.id, zero_for_one, amount).call().await;
	match r {
	    Ok(d) => {return d},
	    Err(d) => {println!("{}", d); return ZERO},
	}
	
    }
}

pub fn uni2(filename: &str) -> std::io::Result<Vec<UniV2Pool>> {
    let data: Vec<_UniV2Pool> = serde_json::from_str(&std::fs::read_to_string(filename)?)?;
    let mut pools: Vec<UniV2Pool> = Vec::new();
    for d in data {
	pools.push(UniV2Pool::from_univ2pool(d));
    }
    return Ok(pools);
}
