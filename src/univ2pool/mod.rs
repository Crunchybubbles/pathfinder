use serde::{Serialize, Deserialize};
use ethers::types::{U256, Address};
use ethers::contract::abigen;
use std::sync::Arc;


use ethers::providers::{Provider, Http};
use crate::constants::ZERO;

abigen!(UniV2Calc, "univ2calc.json");
abigen!(FlashBotsUniV2Query, "FlashBotsUniswapQuery.json");




#[derive(Debug, Deserialize)]
struct _TokenV2 {
    id: String,
}
#[derive(Debug, Serialize, Deserialize, Clone)]    
pub struct TokenV2 {
    pub id: Address,
    pub reserves: U256,
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
    pub async fn flash_all_factorys(fac_addrs: Vec<Address>, query_contract: Arc<FlashBotsUniV2Query<Provider<Http>>>) -> Vec<UniV2Pool> {
	let mut pools: Vec<UniV2Pool> = vec![];
	for factory in fac_addrs {
	    let p = UniV2Pool::from_flash(factory, Arc::clone(&query_contract)).await;
	    println!("number of pools {} for factory {}", p.len(), factory);
	    for a in p {
		pools.push(a);
	    }
	}
	return pools;

    }
    
    pub async fn from_flash(fac_addr: Address, query_contract: Arc<FlashBotsUniV2Query<Provider<Http>>>) -> Vec<UniV2Pool> {
	let mut pools: Vec<UniV2Pool> = vec![];
	let mut start = U256::from_dec_str("0").unwrap();
	let mut stop = U256::from_dec_str("999").unwrap();

	loop {
	    let r = query_contract.get_pairs_by_index_range(fac_addr, start, stop).call().await;
	    let mut addrs: Vec<Address> = vec![];
	    match r {
		Ok(p) => {
		    for i in 0..p.len() {
			let token0 = TokenV2 {id: p[i][0], reserves: ZERO};
			let token1 = TokenV2 {id: p[i][1], reserves: ZERO};
			pools.push(UniV2Pool {id: p[i][2], token0, token1});
			addrs.push(p[i][2]);
		    }  
		}
		Err(_) => {
		    break;
		}
	    }
	    let r2 = query_contract.get_reserves_by_pairs(addrs).call().await;   
	    match r2 {
		Ok(d) => {
		    for i in 0..d.len() {
			pools[i].token0.reserves = d[i][0];
			pools[i].token1.reserves = d[i][1];
		    }
		}
		Err(e) => {eprint!("{}", e)}
	    }
	    start = stop;
	    stop = start + 999;
	    println!("{}", pools.len());
	    
	}
	return pools;
    }
    
    fn from_univ2pool(pool: _UniV2Pool) -> UniV2Pool {
	let id: Address = pool.id.parse::<Address>().unwrap();
	let token0_id: Address = pool.token0.id.parse::<Address>().unwrap();
		
	let token0 = TokenV2 {id: token0_id, reserves: ZERO};

	let token1_id: Address = pool.token1.id.parse::<Address>().unwrap();
		
	let token1 = TokenV2 {id: token1_id, reserves: ZERO};
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

    pub async fn get_amount_out(&self, zf1: bool, amount: U256) -> U256 {
	let mut reserve_out: U256;
	let mut reserve_in: U256;
	
	match zf1 {
	    true => {
		reserve_out = self.token0.reserves;
		reserve_in = self.token1.reserves;
	    }
	    false => {
		reserve_out = self.token1.reserves;
		reserve_in = self.token0.reserves;
	    }
	}
	
	if self.token0.reserves > ZERO && self.token1.reserves > ZERO {
	    let amount_in_with_fee: U256;
	    if let Some(r) = amount.checked_mul(U256{0:[997,0,0,0]}) {
		amount_in_with_fee = r;
	    } else {
		return ZERO;
	    }
	    let numerator: U256;
	    if let Some(r) = amount_in_with_fee.checked_mul(reserve_out) {
		numerator = r;
	    } else {
		return ZERO;
	    }
	    let denominator: U256;
	    if let Some(r) = reserve_in.checked_mul(U256{0:[1000,0,0,0]}) {
		if let Some(rr) = r.checked_add(amount_in_with_fee) {
		    denominator = rr;
		} else {
		    return ZERO;
		}
	    } else {
		return ZERO;
	    }
	    let amount_out: U256;
	    if let Some(r) = numerator.checked_div(denominator) {
		amount_out = r;
	    } else {
		return ZERO;
	    }
	    let kpre: U256;
	    if let Some(r) = reserve_in.checked_mul(reserve_out) {
		kpre = r;
	    } else {
		return ZERO;
	    }
	    if let Some(r) = reserve_out.checked_add(amount_in_with_fee) {
		reserve_out = r;
	    } else {
		return ZERO;
	    }
	    
	    if let Some(r) = reserve_in.checked_sub(amount_out) {
		reserve_in = r;
	    } else {
		return ZERO;
	    }

	    if let Some(r) = reserve_in.checked_mul(reserve_out) {
		if r <= kpre {
		    return ZERO;
		} else {
		    return amount_out;
		}
	    } else {
		return ZERO;
	    }
	} else {
	    return ZERO;
	}
    }

    pub async fn get_amount_in(&self, zf1: bool, amount: U256) -> U256 {
	let reserve_out: U256;
	let reserve_in: U256;
	
	match zf1 {
	    true => {
		reserve_out = self.token0.reserves;
		reserve_in = self.token1.reserves;
	    }
	    false => {
		reserve_out = self.token1.reserves;
		reserve_in = self.token0.reserves;
	    }
	}
	
	if self.token0.reserves > ZERO && self.token1.reserves > ZERO {
	    let numerator: U256;
	    if let Some(r) = reserve_in.checked_mul(amount) {
		if let Some(rr) = r.checked_mul(U256{0: [1000,0,0,0]}) {
		    numerator = rr;
		} else {
		    return ZERO;
		}
	    } else {
		return ZERO;
	    }
	    
	    let denominator: U256;
	    if let Some(r) = reserve_out.checked_sub(amount) {
		if let Some(rr) = r.checked_mul(U256{0:[977,0,0,0]}) {
		    denominator = rr;
		} else {
		    return ZERO;
		}
	    } else {
		return ZERO;
	    }
	    
	    let amount_in: U256;
	    if let Some(r) = numerator.checked_div(denominator) {
		amount_in = r + U256{0:[1,0,0,0]};
		return amount_in;
	    } else {
		return ZERO;
	    }

	    // let kpre: U256;
	    // if let Some(r) = reserve_in.checked_mul(reserve_out) {
	    // 	kpre = r;
	    // } else {
	    // 	return ZERO;
	    // }

	    // if let Some(r) = reserve_out.checked_add(amount_in) {
	    // 	reserve_out = r;
	    // } else {
	    // 	return ZERO;
	    // }

	    // if let Some(r) = reserve_in.checked_sub(amount) {
	    // 	reserve_in = r;
	    // } else {
	    // 	return ZERO;
	    // }

	    // if let Some(r) = reserve_in.checked_mul(reserve_out) {
	    // 	if r <= kpre {
	    // 	    return ZERO;
	    // 	} else {
	    // 	    return amount_in;
	    // 	}
	    // } else {
	    // 	return ZERO;
	    // }
	    

	} else {
	    return ZERO;
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
