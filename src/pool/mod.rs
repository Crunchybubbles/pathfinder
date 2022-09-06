#![allow(unused_imports)]
use crate::{
    univ3pool::{UniV3Pool, UniV3Calc, uni3},
    univ2pool::{UniV2Pool, UniV2Calc, FlashBotsUniV2Query},
    constants::{SHIBA_FAC, SUSHI_FAC, UNIV2_FAC},
};
use ethers::{
    types::{Address, U256},
    providers::{Provider, Http},
};
use std::sync::Arc;

use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]    
pub enum Pool {
    V3(UniV3Pool),
    V2(UniV2Pool),
}

impl Pool {
    pub fn token_out(&self, token_in: &Address) -> &Address {
	match self {
	    Pool::V2(pooldata) => {
		return pooldata.token_out(token_in)
	    }

	    Pool::V3(pooldata) => {
		return pooldata.token_out(token_in)
	    }
	}
    }

    pub fn token0(&self) -> &Address {
	match self {
	    Pool::V2(pool) => {
		return &pool.token0.id;
	    }

	    Pool::V3(pool) => {
		return &pool.token0.id;
	    }
	}
    }


    pub fn token1(&self) -> &Address {
	match self {
	    Pool::V2(pool) => {
		return &pool.token1.id;
	    }

	    Pool::V3(pool) => {
		return &pool.token1.id;
	    }
	}
    }

    pub fn addr(&self) -> &Address {
	match self {
	    Pool::V2(pool) => {
		return &pool.id;
	    }

	    Pool::V3(pool) => {
		return &pool.id;
	    }
	}
    }

    pub async fn amount_out(&self, token_in: &Address, amount: U256, uni_v2_calc: Arc<UniV2Calc<Provider<Http>>>, uni_v3_calc: Arc<UniV3Calc<Provider<Http>>>) -> U256 {
	match self {
	    Pool::V2(pool) => {
		return pool.amount_out(token_in, amount, uni_v2_calc).await;
	    }
	    Pool::V3(pool) => {
		return pool.amount_out(token_in, amount, uni_v3_calc).await;
	    }
	}
    }

}

impl PartialEq for Pool {
    fn eq(&self, other: &Self) -> bool {
	self.addr() == other.addr()
    }
   
}

    
pub async fn load_pools(query_contract: Arc<FlashBotsUniV2Query<Provider<Http>>>) -> std::io::Result<Vec<Pool>> {
    let uni3 = uni3().unwrap();
    //let uni2pools = uni2("uni2data.txt").unwrap();
    //let sushi = uni2("sushipools.txt").unwrap();
    //let sushi_fac = "0xC0AEe478e3658e2610c5F7A4A2E1777cE9e4f2Ac".parse::<Address>().unwrap();
    //let univ2_fac = "0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f".parse::<Address>().unwrap();
    //let shiba_fac = "0x115934131916C8b277DD010Ee02de363c09d037c".parse::<Address>().unwrap();
    

    let v2 = UniV2Pool::flash_all_factorys(vec![SHIBA_FAC ,SUSHI_FAC, UNIV2_FAC], query_contract).await;
    let mut pools: Vec<Pool> = Vec::new();
    for pool in uni3 {
	let p = Pool::V3(pool);
	pools.push(p);

    }

    for pool in v2 {
	let p = Pool::V2(pool);
	pools.push(p);
    
    }

 
    
    return Ok(pools);    
}

