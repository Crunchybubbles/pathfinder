#![allow(dead_code, unused_imports, unused_variables)]
use serde_json::Value;
use std::{
    fs,
    io,
    collections::{HashMap, VecDeque},
    sync::Arc,
    time::Instant,
};
use ethers::types::{Address, U256};
use serde::{Serialize, Deserialize};
use ethers::{prelude::*, utils::parse_ether};
use ahash::AHashMap;

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
struct Tick {
    tick_idx: i64,
    liquidity_net: U256,
}
#[derive(Debug, Serialize, Deserialize, Clone)]    
struct Token {
    id: Address,
    symbol: String,
    decimals: u64,
}
#[derive(Debug, Serialize, Deserialize, Clone)]    
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

    fn token_out(&self, token_in: &Address) -> &Address {
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

    async fn amount_out(&self, token_in: &Address, amount_in: U256, swap_calc_contract: Arc<SwapCalc<Provider<Http>>>) -> U256 {

	let zero_for_one = self.zf1(token_in);
	let spl = sqrt_price_limit(zero_for_one);
	let amount: I256 = I256::try_from(amount_in).unwrap();
	    
	let r = swap_calc_contract.calc_v_3_swap(self.id, zero_for_one, amount, spl).call().await;
	if let Ok(d) = r {
	    if d.0.is_positive() {
		return U256::try_from(d.1.abs()).unwrap();
	    } else {
		return U256::try_from(d.0.abs()).unwrap();
	    }
	} else {
	    return ZERO;
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

#[derive(Debug, Deserialize)]
struct _TokenV2 {
    id: String,
}
#[derive(Debug, Serialize, Deserialize, Clone)]    
struct TokenV2 {
    id: Address,
}

#[derive(Debug, Deserialize)]
struct _UniV2Pool {
    id: String,
    token0: _TokenV2,
    token1: _TokenV2,  
}

#[derive(Debug, Serialize, Deserialize, Clone)]    
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

    fn token_out(&self, token_in: &Address) -> &Address {
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

    async fn amount_out(&self, token_in: &Address, amount: U256, swap_calc_contract: Arc<SwapCalc<Provider<Http>>>) -> U256 {
	let zero_for_one = self.zf1(token_in);
	let r = swap_calc_contract.calc_univ_2_amount_out(self.id, zero_for_one, amount).call().await;
	match r {
	    Ok(d) => {return d},
	    Err(d) => {return ZERO},
	}
	
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]    
enum Pool {
    V3(UniV3Pool),
    V2(UniV2Pool),
}

impl Pool {
    fn token_out(&self, token_in: &Address) -> &Address {
	match self {
	    Pool::V2(pooldata) => {
		return pooldata.token_out(token_in)
	    }

	    Pool::V3(pooldata) => {
		return pooldata.token_out(token_in)
	    }
	}
    }

    fn token0(&self) -> &Address {
	match self {
	    Pool::V2(pool) => {
		return &pool.token0.id;
	    }

	    Pool::V3(pool) => {
		return &pool.token0.id;
	    }
	}
    }


    fn token1(&self) -> &Address {
	match self {
	    Pool::V2(pool) => {
		return &pool.token1.id;
	    }

	    Pool::V3(pool) => {
		return &pool.token1.id;
	    }
	}
    }

    fn addr(&self) -> &Address {
	match self {
	    Pool::V2(pool) => {
		return &pool.id;
	    }

	    Pool::V3(pool) => {
		return &pool.id;
	    }
	}
    }

    async fn amount_out(&self, token_in: &Address, amount: U256, swap_calc_contract: Arc<SwapCalc<Provider<Http>>>) -> U256 {
	match self {
	    Pool::V2(pool) => {
		return pool.amount_out(token_in, amount, swap_calc_contract).await;
	    }
	    Pool::V3(pool) => {
		return pool.amount_out(token_in, amount, swap_calc_contract).await;
	    }
	}
    }

}

impl PartialEq for Pool {
    fn eq(&self, other: &Self) -> bool {
	self.addr() == other.addr()
    }
   
}
// struct Graph<T, G> {
//     nodes: Vec<Box<T>>,
//     edges: HashMap<G, Vec<Box<T>>>,
// }

// impl Graph<Pool, Address> {
//     fn new(pools: Vec<Pool>) -> Self {
// 	let mut nodes: Vec<Box<Pool>> = Vec::new();
// 	let mut edges: HashMap<Address, Vec<Box<Pool>>> = HashMap::new();
// 	for pool in pools {
// 	    let token0 = *pool.token0();
// 	    let token1 = *pool.token1();
// 	    let b = Box::new(pool);
// 	    if let Some(stuff) = edges.get_mut(&token0) {
// 		stuff.push(b.clone());
// 	    } else {
// 		edges.insert(token0, vec![b.clone()]);
// 	    }

// 	    if let Some(stuff) = edges.get_mut(&token1) {
// 		stuff.push(b.clone());
// 	    } else {
// 		edges.insert(token1, vec![b.clone()]);
// 	    }
// 	    nodes.push(b);
    
// 	}

// 	return Graph{nodes, edges};
//     }
    
// }
#[derive(Debug, Copy, Clone)]
struct Step<'a> {
    pool: &'a Box<Pool>,
    token_in: &'a Address,
    token_out: &'a Address,
    amount_in: U256,
    amount_out: U256,
}

#[derive(Debug, Clone)]
struct SwapPath<'a>{
    steps: Vec<Step<'a>>,
    addrs: Vec<Address>,
}


impl <'a> SwapPath<'a> {
    fn contains_token(&self, target: &Address) -> bool {
	let mut r: bool = false;
	for step in self.steps.iter() {
	    if step.token_out == target {
		r = true;
		break;
	    }
	}
	return r;
    }

    fn show(&self) {
	for step in self.steps.iter() {
	    println!("address {}", step.pool.addr());
	    println!("tokenIn {}", step.token_in);
	    println!("tokenOu {}", step.token_out);
	    
	}
	println!("");
    }
    

}

impl<'a> PartialEq for SwapPath<'a> {
    fn eq(&self, other: &Self) -> bool {
	let mut v: bool = false;
	if self.steps.len() != other.steps.len() {
	    return v;
	} else {
	    for i in 0..self.steps.len() {
		if self.steps[i].pool != other.steps[i].pool {
		    return v;
		}
	    }
	    v = true;
	    return v;
	}
    }
}

struct Graph<T, G> {
    edges: HashMap<G, Vec<Box<T>>>,
}

impl Graph<Pool, Address> {
    fn new(pools: Vec<Pool>) -> Self {
	let mut edges: HashMap<Address, Vec<Box<Pool>>> = HashMap::with_capacity(pools.len());
	for pool in pools {
	    let token0 = *pool.token0();
	    let token1 = *pool.token1();
	    let b = Box::new(pool);
	    if let Some(stuff) = edges.get_mut(&token0) {
		stuff.push(b.clone());
	    } else {
		edges.insert(token0, vec![b.clone()]);
	    }

	    if let Some(stuff) = edges.get_mut(&token1) {
		stuff.push(b);
	    } else {
		edges.insert(token1, vec![b]);
	    }
    
	}

	return Graph{edges};
    }
    
    async fn dfs_each_token_once(&self, start: &Address, end: &Address) -> Vec<SwapPath> {
	let mut stack: Vec<SwapPath> = Vec::with_capacity(90000);
	let mut paths: Vec<SwapPath> = Vec::with_capacity(500);
//	const MAXLEN_MIN_ONE: usize = MAXLEN - 1;
//	const MAXCAP: usize = MAXLEN + 1;


	let pools = self.edges.get(start).unwrap();
	for pool in pools {
	    let token_out = pool.token_out(start);
	    let token_in = pool.token_out(token_out);
	    let mut steps: Vec<Step> = Vec::with_capacity(MAXLEN);
	    steps.push(Step{pool, token_in, token_out, amount_in: ZERO, amount_out: ZERO});
	    let mut addrs: Vec<Address> = Vec::with_capacity(MAXLEN);
	    addrs.push(*pool.addr());
	    let sp = SwapPath{steps, addrs};
	    if token_out == end {
		paths.push(sp);
	    } else {
		stack.push(sp);
	    }	 
	}
	loop {
	    // println!("stacke len {}", stack.len());
	    // if let Some(last) = paths.last() {
	    // 	println!("path len {:#?}", last.steps);
	    // }

	    // if let Some(last) = paths.last() {
	    // 	println!("path len {}", last.steps.len());
	    // 	println!("paths found {}",paths.len()); 
	    // }
	    
	    if let Some(path) = stack.pop() {
		let last = path.steps.len() - 1;
		let last_step = path.steps[last];
		if let Some(pools) = self.edges.get(&last_step.token_out) {
		    for pool in pools {
			let mut p = path.clone();
			if !p.addrs.contains(&pool.addr()) {
			    let token_in = last_step.token_out;
			    let token_out =  pool.token_out(token_in);
			    if !p.contains_token(&token_out) {
				let step = Step{pool, token_in, token_out, amount_in: ZERO, amount_out: ZERO};
				p.steps.push(step);
				if step.token_out == end {
				    paths.push(p);
				} else {
				    if p.steps.len() < MAXLEN {
					stack.push(p);
				    }
				}
			
			    }
			}
		    }
		    
		}
	    } else {
		break;
	    }
	}
	//println!("{}", paths.len());
	return paths;
    }

    fn markets_of_token(&self, token: &Address) -> Option<Vec<Box<Pool>>> {
	if let Some(pools) = self.edges.get(token) {
	    Some(pools.clone())
	} else {
	    None
	}

    }

}

struct Raph<P, A, I> {
    pools: Vec<P>,
    tokens: AHashMap<A, I>,
    ttp: AHashMap<I, Vec<I>>,
    itt: AHashMap<I, A>,
    ptt: Vec<[I; 2]>
    
}

impl Raph<Pool, Address, usize> {
    fn new(pools: Vec<Pool>) -> Self {
	let mut tokens: AHashMap<Address, usize> = AHashMap::with_capacity(pools.len());
	let mut ttp: AHashMap<usize, Vec<usize>> = AHashMap::with_capacity(pools.len());
	let mut token_index: usize = 0;
	let mut itt: AHashMap<usize, Address> = AHashMap::with_capacity(pools.len());
	let mut ptt: Vec<[usize; 2]> = Vec::with_capacity(pools.len());
	let mut token0: usize = 0;
	let mut token1: usize = 0;
	
	for (pool_index, pool) in pools.clone().iter().enumerate() {
	    if let Some(token0_index) = tokens.get(pool.token0()) {
		if let Some(a) = ttp.get_mut(token0_index) {
		    a.push(pool_index);
		    token0 = *token0_index;
		}
		    
	    } else {
		tokens.insert(*pool.token0(), token_index);
		itt.insert(token_index, *pool.token0());
		ttp.insert(token_index, vec![pool_index]);
		token0 = token_index;
		token_index += 1;
		
	    }
	    if let Some(token1_index) = tokens.get(pool.token1()) {
		if let Some(a) = ttp.get_mut(token1_index) {
		    a.push(pool_index);
		    token1 = *token1_index;
		}
		    
	    } else {
		tokens.insert(*pool.token1(), token_index);
		itt.insert(token_index, *pool.token1());
		ttp.insert(token_index, vec![pool_index]);
		token1 = token_index;
		token_index += 1;
	    }
	    
	    ptt.push([token0, token1]);

	    
	}
	    
	Raph{pools, tokens, ttp, itt, ptt}
    }

    fn markets_of_token(&self, token: &Address) -> Option<Vec<Pool>> {
	
	if let Some(token_index) = self.tokens.get(token) {
	    if let Some(pool_indices) = self.ttp.get(token_index) {
		let mut pools: Vec<Pool> = Vec::with_capacity(pool_indices.len());
		for pool_index in pool_indices.iter() {
		    pools.push(self.pools[*pool_index].clone());
		}
		return Some(pools);
	    } else {
		return None;
	    }  
	} else {
	    return None;
	}

    }

    async fn find_path(&self, start: &Address, finish: &Address) -> Option<Vec<Path>> {
	let mut stack: Vec<Path> = Vec::with_capacity(100000);
	let mut found_paths: Vec<Path> = Vec::with_capacity(100000);
	if let Some(target_index) = self.tokens.get(finish) {
	    if let Some(token_index) = self.tokens.get(start) {
		if let Some(pool_indices) = self.ttp.get(token_index) {
		    for pool_index in pool_indices.iter() {
			let step = PathStep{pool: pool_index, token_in: token_index, token_out: self.token_out(*pool_index, *token_index)};
			if step.token_out == target_index {
			    found_paths.push(Path{steps: vec![step]});
			} else {
			    let mut steps: Vec<PathStep> = Vec::with_capacity(MAXLEN);
			    steps.push(step);
			    stack.push(Path{steps});
			}
		    }

		} else {
		    return None;
		}
	    } else {
		return None;
	    }
	

	    loop {
		if let Some(path) = stack.pop() {
		    let last = path.steps.last().unwrap();
		    if let Some(pools) = self.ttp.get(last.token_out) {
			for pool in pools.iter() {
			    if !path.contains(pool) {
				let token_out = self.token_out(*pool, *last.token_out);
				if !path.used_token(token_out) {
				    let mut p = path.clone();
				    let step = PathStep{pool, token_in: last.token_out, token_out};
				    p.steps.push(step);
				    if p.steps.last().unwrap().token_out == target_index {
					found_paths.push(p);
				    } else {
					if p.steps.len() < MAXLEN {
					    stack.push(p);
					}
				    }
				}
				
			    }
			}
		    }
		    
		} else {
		    break;
		}
	    }
	} else {
	    return None;
	}

    
	return Some(found_paths);
    }

    fn token_out(&self, pool: usize, token_in: usize) -> &usize {
	if self.ptt[pool][0] == token_in {
	    return &self.ptt[pool][1];
	} else {
	    return &self.ptt[pool][0];
	}
    }
}
#[derive(Clone)]
struct Path <'a> {
    steps: Vec<PathStep<'a>>
}

impl <'a> Path <'a> {
    fn contains(&self, pool: &usize) -> bool {
	let mut r = false;
	for step in self.steps.iter() {
	    if step.pool == pool {
		r = true;
		break;
	    }
	}
	return r;
    }

    fn used_token(&self, token: &usize) -> bool {
	let mut r = false;
	for step in self.steps.iter() {
	    if step.token_out == token {
		r = true;
		break;
	    }
	}
	return r;
    }
}

#[derive(Clone)]
struct PathStep <'a> {
    pool: &'a usize,
    token_in: &'a usize,
    token_out: &'a usize,
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

abigen!(SwapCalc, "/Users/jasper/learningswaps/build/contracts/TickTest.json");

const MAX_SQRT_PRICE: U256 = U256{0:[
    6743328256752651557,
    17280870778742802505,
    4294805859,
    0,
]};

const MIN_SQRT_PRICE: U256 = U256{0:[
    4295128740,
    0,
    0,
    0,
]};

const ZERO: U256 = U256{0:[0,0,0,0]};

const MAXLEN: usize = 2;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let pools = load_pools().unwrap();
    //let now = Instant::now();
    //let mm = market_map(&pools);
    //let mm_time = now.elapsed();
    // let now = Instant::now();
    // //let pooldata = pools_to_map(pools);
    // let graph = Graph::new(pools);
    // let pd_time = now.elapsed();

    //let pools = uni3().unwrap();
    
    //println!("{}", mm_time.as_millis());
    // println!("{}", pd_time.as_millis());
    //let anvil = Anvil::new().fork("https://mainnet.infura.io/v3/cb7a603124c4411ba12877599e494814").spawn();
    //let wallet: LocalWallet = anvil.keys()[0].clone().into();
    //let provider = Provider::<Http>::try_from(anvil.endpoint())?;
    // let provider = Provider::<Http>::try_from("http://127.0.0.1:8545")?;
    // let addr = "0x9E4c14403d7d9A8A782044E86a93CAE09D7B2ac9".parse::<Address>().unwrap();
    // let client = Arc::new(provider);
    // let uni_calc = Arc::new(SwapCalc::new(addr, client));

    // let amount = U256::from_dec_str("100000").unwrap();
    let t = Instant::now();
    let graph = Graph::new(pools.clone());
    let took = t.elapsed();
    println!("{}", took.as_millis());

    let t = Instant::now();
    let raph = Raph::new(pools.clone());
    let took = t.elapsed();
    println!("{}", took.as_millis());

    //let weth = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".parse::<Address>().unwrap();
    let yfi = "0x0bc529c00C6401aEF6D220BE8C6Ea1667F6Ad93e".parse::<Address>().unwrap();

    let t = Instant::now();
    let g = graph.dfs_each_token_once(&yfi, &yfi).await;
    let gt = t.elapsed();
    
    let t = Instant::now();
    let r = raph.find_path(&yfi, &yfi).await.unwrap();
    let rt = t.elapsed();


    println!("{}", gt.as_millis());
    println!("{}", rt.as_millis());

    println!("{}", g.len());
    println!("{}", r.len());
    

//     let now = Instant::now();
// //    let r = graph.bfs(pools[0].token0(), pools[0].token1(), amount, uni_calc).await;
//     let r = graph.dfs_each_token_once(&yfi, &yfi).await;
//     let took = now.elapsed();
//     let pathcount = r.len();
    
//     for path in r {
// 	path.show();
	
//     }
//     println!("{}", pathcount);
//     println!("{}", took.as_millis());
//     println!("{}", graph.edges.len());
    // for pool in pools {	
    // 	let r = pool.amount_out(pool.token0(), amount, uni_calc.clone()).await;
    // 	println!("{}", r);
    // }

    
    // let pool = "0x04916039B1f59D9745Bf6E0a21f191D1e0A84287".parse::<Address>().unwrap();
    // let direction = false;
    // let amount = U256::from_dec_str("50000000000000000000")?;
    
    // println!("{}", &pool);
    // println!("{}", direction);
    // println!("{}", &amount);
    
   // println!("{:#?}", MAX_SQRT_PRICE.0);
    // let r = uni_calc.ez_v_3_calc(pool, amount, direction).call().await?;

    // println!("{:?}", r);
      
    
    
    
    // for pool in pools {
    // 	let token_in = pool.token0();
    // 	println!("{:x} {:x}", token_in, pool.addr());
    // 	println!("{}", pool.amount_out(token_in, amount, uni_calc.clone()).await);
    // }
    

    // let pool = "0x04916039b1f59d9745bf6e0a21f191d1e0a84287".parse::<Address>().unwrap();
    // let amount = parse_ether("1").unwrap();
    // let zf1 = true;
    // let r = uni_calc.ez_v_3_calc(pool, amount, zf1).call().await.unwrap();
    // println!("{:#?}", r);
    Ok(())

    //println!("{}", graph.nodes.len());
    // for node in graph.nodes {
// 	match *node {
// 	    Pool::V2(_) => {}
// 	    Pool::V3(_) => {println!("im doing stuff")}
// 	}
// //	println!("{:#?}", *node);
    //     }
    // for (token, pools) in graph.edges.iter() {

    // 	if pools.len() > 2 {
    // 	    println!("{}", token);
    // 	    println!("{}", pools.len());
    // 	    println!("");
    // 	}
	

    // }
    //let path = graph.find_path(yfi, weth);
    //println!("{:?}", path);
    // println!("{:#?}", mm.get(&weth));
}
