use serde_json::Value;
use std::fs;
use std::io;
use std::collections::HashMap;
use std::time::Instant;
use ethers::types::{Address, U256};
use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::BufReader;

#[derive(Debug, Deserialize)]
struct _Token {
    id: String,
    symbol: String,
    decimals: String,
}

#[derive(Debug, Deserialize)]
struct _Tick {
    tickIdx: String,
    liquidityNet: String,
}

#[derive(Debug, Deserialize)]
struct _UniV3Pool {
    id: String,
    token0: _Token,
    token1: _Token,
    feeTier: String,
    tick: String,
    ticks: Vec<_Tick>,
}

#[derive(Debug, Serialize, Deserialize)]    
struct Tick {
    tick_idx: i64,
    liquidity_net: U256,
}
#[derive(Debug, Serialize, Deserialize)]    
struct Token {
    id: Address,
    symbol: String,
    decimals: u64,
}
#[derive(Debug, Serialize, Deserialize)]    
struct UniV3Pool {
    id: Address,
    token0: Token,
    token1: Token,
    fee: u64,
    tick: i64,
    ticks: Vec<Tick>,
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

}
#[derive(Debug, Deserialize)]
struct _TokenV2 {
    id: String,
}
#[derive(Debug, Serialize, Deserialize)]    
struct TokenV2 {
    id: Address,
}

#[derive(Debug, Deserialize)]
struct _UniV2Pool {
    id: String,
    token0: _TokenV2,
    token1: _TokenV2,  
}

#[derive(Debug, Serialize, Deserialize)]    
struct UniV2Pool {
    id: Address,
    token0: TokenV2,
    token1: TokenV2,
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
}

#[derive(Debug, Serialize, Deserialize)]    
enum Pool {
    V3(UniV3Pool),
    V2(UniV2Pool),
}

fn uni3() -> io::Result<Vec<UniV3Pool>> {
    let mut pools: Vec<UniV3Pool> = Vec::new();
    let uni3: Vec<_UniV3Pool> = serde_json::from_str(&fs::read_to_string("uni3data.txt")?)?;
    for pool in uni3 {
	pools.push(UniV3Pool::from_univ3pool(pool));
    }
    return Ok(pools);
}

fn uni2(filename: &str) -> io::Result<Vec<UniV2Pool>> {
    let data: Vec<_UniV2Pool> = serde_json::from_str(&fs::read_to_string(filename)?)?;
    let mut pools: Vec<UniV2Pool> = Vec::new();
    for d in data {
	pools.push(UniV2Pool::from_univ2pool(d));
    }
    return Ok(pools);
}
    
fn load_pools() -> io::Result<Vec<Pool>> {
    let uni3 = uni3().unwrap();
    let uni2pools = uni2("uni2data.txt").unwrap();
    let sushi = uni2("sushipools.txt").unwrap();

    let mut pools: Vec<Pool> = Vec::new();

    for pool in uni3 {
	let p = Pool::V3(pool);
	pools.push(p);
    }

    for pool in uni2pools {
	let p = Pool::V2(pool);
	pools.push(p);
    }

    for pool in sushi {
	let p = Pool::V2(pool);
	pools.push(p);
    }

    
    return Ok(pools);    
}

fn market_map(pools: &Vec<Pool>) -> HashMap<Address, Vec<Address>> {
    let mut mm: HashMap<Address, Vec<Address>> = HashMap::new();
    for pool in pools {
	match pool {
	    Pool::V2(pooldata) => {
		if !mm.contains_key(&pooldata.token0.id) {
		    let mut v: Vec<Address> = Vec::new();
		    v.push(pooldata.id);
		    mm.insert(pooldata.token0.id, v);
		} else {
		    let mut v: Vec<Address> = mm.get(&pooldata.token0.id).unwrap().to_vec();
		    v.push(pooldata.id);
		    mm.insert(pooldata.token0.id, v.to_vec());
		}

		if !mm.contains_key(&pooldata.token1.id) {
		    let mut v: Vec<Address> = Vec::new();
		    v.push(pooldata.id);
		    mm.insert(pooldata.token1.id, v);
		} else {
		    let mut v: Vec<Address> = mm.get(&pooldata.token1.id).unwrap().to_vec();
		    v.push(pooldata.id);
		    mm.insert(pooldata.token1.id, v.to_vec());
		}
	    },
	    Pool::V3(pooldata) => {
		if !mm.contains_key(&pooldata.token0.id) {
		    let mut v: Vec<Address> = Vec::new();
		    v.push(pooldata.id);
		    mm.insert(pooldata.token0.id, v);
		} else {
		    let mut v: Vec<Address> = mm.get(&pooldata.token0.id).unwrap().to_vec();
		    v.push(pooldata.id);
		    mm.insert(pooldata.token0.id, v.to_vec());
		}

		if !mm.contains_key(&pooldata.token1.id) {
		    let mut v: Vec<Address> = Vec::new();
		    v.push(pooldata.id);
		    mm.insert(pooldata.token1.id, v);
		} else {
		    let mut v: Vec<Address> = mm.get(&pooldata.token1.id).unwrap().to_vec();
		    v.push(pooldata.id);
		    mm.insert(pooldata.token1.id, v.to_vec());
		}
	    }
	}
	
    }


    return mm;
}


fn save_market_map() -> std::io::Result<()> {
    let pools = load_pools().unwrap();
    let mm = market_map(&pools);
    let mut file = File::create("market_map.txt")?;
    serde_json::to_writer(file, &mm)?;
    Ok(())

}

fn load_market_map() -> std::io::Result<HashMap<Address, Vec<Address>>> {
    let file = File::open("market_map.txt")?;
    let reader = BufReader::new(file);
    let mm: HashMap<Address, Vec<Address>> = serde_json::from_reader(reader)?;
    Ok(mm)
}


fn main() {
    let mm = load_market_map().unwrap();
    let weth = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".parse::<Address>().unwrap();
    println!("{:#?}", mm.get(&weth));
}
