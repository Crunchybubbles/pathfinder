#![allow(unused_imports)]
use crate::{
    univ3pool::{UniV3Pool, UniV3Calc, uni3},
    univ2pool::{UniV2Pool, UniV2Calc, FlashBotsUniV2Query},
    constants::{SHIBA_FAC, SUSHI_FAC, UNIV2_FAC},
};
use ethers::{
    types::{Address, U256, Transaction, Block, U64},
    providers::{Provider, Http, Middleware},
    
};
use std::{sync::Arc, fs::File, io::BufReader};

use ahash::AHashMap;


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

 
    pub async fn check_and_update(mut pools: Vec<Self>, set: &AHashMap<Address, usize>, block: Block<Transaction>, query_contract: Arc<FlashBotsUniV2Query<Provider<Http>>>) -> Vec<Self> {
	let mut pools_to_update: Vec<Address> = Vec::with_capacity(pools.len());
	let mut pool_indices: Vec<usize> =  Vec::with_capacity(pools.len());

	for tx in block.transactions.iter() {
	    if let Some(r) = &tx.access_list {
		for item in r.0.iter() {
		    if let Some(index) = set.get(&item.address) {
			match &pools[*index] {
			    Pool::V2(p) => {
				pools_to_update.push(p.id);
				pool_indices.push(*index);
			    }
			    Pool::V3(_) => {}
			}

		    } 
		}
	    }
	}	    
	
	println!("updating {} pools", pools_to_update.len());
	println!("{:#?}", pools_to_update);
	let r = match query_contract.get_reserves_by_pairs(pools_to_update).call().await {
	    Ok(t) => {t},
	    Err(e) => {eprint!("{}", e); return pools}
	};
	for (i,reserves) in r.iter().enumerate() {
	    let index = pool_indices[i];
	    match &mut pools[index] {
		Pool::V2(pool) => {
		    pool.token0.reserves = reserves[0];
		    pool.token1.reserves = reserves[1];
		}
		Pool::V3(_) => {}

	    }
	}
	return pools;
    }

    

}

impl PartialEq for Pool {
    fn eq(&self, other: &Self) -> bool {
	self.addr() == other.addr()
    }
   
}

    
pub async fn load_pools(query_contract: Arc<FlashBotsUniV2Query<Provider<Http>>>) -> std::io::Result<Vec<Pool>> {
    let uni3 = uni3().unwrap();
 
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

pub fn save_pools(pools: &Vec<Pool>) -> std::io::Result<()> {
    serde_json::to_writer(File::create("pools.txt")?, pools)?;
    println!("saved pool data!");
    Ok(())
}

pub fn load_pools_from_save() -> std::io::Result<Vec<Pool>> {
    let file = File::open("pools.txt")?;
    let reader = BufReader::new(file);
    let pools: Vec<Pool> = serde_json::from_reader(reader)?;
    Ok(pools)
}

#[derive(Serialize, Deserialize)]
pub struct PoolSave {
    pub pools: Vec<Pool>,
    pub block: U64,
}

impl PoolSave {
    pub async fn save(pools: Vec<Pool>, block: U64) -> std::io::Result<()> {
	let info = PoolSave{pools, block};
	serde_json::to_writer(File::create("pools.txt")?, &info)?;
	Ok(())
    }

    pub fn load() -> std::io::Result<PoolSave> {
	let file = File::open("pools.txt")?;
	let reader = BufReader::new(file);
	let pools: PoolSave = serde_json::from_reader(reader)?;
	Ok(pools)
    }
}
