use crate::{
    univ2pool::UniV2Calc,
    univ3pool::UniV3Calc,
    pool::Pool,
};
use ethers::types::{U256, Address};
use ethers::providers::{Provider, Http};
use std::sync::Arc;

pub struct Calculator {
    v2: Arc<UniV2Calc<Provider<Http>>>,
    v3: Arc<UniV3Calc<Provider<Http>>>,
}

impl Calculator {
    pub fn new(v2_address: &str, v3_address: &str, client: Arc<Provider<Http>>) -> Self {
	let v2_addr = v2_address.parse::<Address>().unwrap();
	let v3_addr = v3_address.parse::<Address>().unwrap();
	
	
	Calculator {
	    v2: Arc::new(UniV2Calc::new(v2_addr, Arc::clone(&client))),
	    v3: Arc::new(UniV3Calc::new(v3_addr, Arc::clone(&client)))
	}
    }
    
    pub async fn amount_out_for_an_input(&self, pool: &Pool, zf1: bool, amount_in: U256) -> U256 {
	let token_in: &Address;
	match zf1 {
	    true => {token_in = pool.token0()}
	    false => {token_in = pool.token1()}
	    
	}
	match pool {
	    Pool::V2(p) => {p.amount_out(token_in, amount_in, Arc::clone(&self.v2)).await}
	    Pool::V3(p) => {p.amount_out(token_in, amount_in, Arc::clone(&self.v3)).await}

	}
    }

    pub async fn amount_in_for_an_output(&self, pool: &Pool, zf1: bool, amount_out: U256) -> U256 {
	let token_in: &Address;
	match zf1 {
	    true => {token_in = pool.token0()}
	    false => {token_in = pool.token1()}
	    
	}
	match pool {
	    Pool::V2(p) => {p.amount_out(token_in, amount_out, Arc::clone(&self.v2)).await}
	    Pool::V3(p) => {p.amount_in_for_out(zf1, amount_out, Arc::clone(&self.v3)).await}

	}
	
	
    }
}
