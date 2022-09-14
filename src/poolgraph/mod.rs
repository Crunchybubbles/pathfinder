#![allow(dead_code)]
use ahash::AHashMap;
use crate::{pool::Pool, constants::{MAXLEN, ZERO}, univ2pool::FlashBotsUniV2Query};
use ethers::{providers::{Provider, Http}, types::{Address, Transaction, Block, U64, U256}};
use std::sync::Arc;
use crossbeam::channel::{unbounded, Receiver, Sender};


#[derive(Clone)]
pub struct Graph<P, A, I> {
    pub pools: Vec<P>,
    set: AHashMap<A, I>,
    tokens: AHashMap<A, I>,
    ttp: AHashMap<I, Vec<I>>,
    itt: AHashMap<I, A>,
    ptt: Vec<[I; 2]>
    
}

impl Graph<Pool, Address, usize> {
    pub fn new(pools: Vec<Pool>) -> Self {
	let mut tokens: AHashMap<Address, usize> = AHashMap::with_capacity(pools.len());
	let mut ttp: AHashMap<usize, Vec<usize>> = AHashMap::with_capacity(pools.len());
	let mut token_index: usize = 0;
	let mut itt: AHashMap<usize, Address> = AHashMap::with_capacity(pools.len());
	let mut ptt: Vec<[usize; 2]> = Vec::with_capacity(pools.len());
	let mut token0: usize = 0;
	let mut token1: usize = 0;
	let mut set: AHashMap<Address, usize> = AHashMap::with_capacity(pools.len());
	
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
	    set.insert(*pool.addr(), pool_index);
	    
	}
	    
	Graph{pools, set, tokens, ttp, itt, ptt}
    }

    pub async fn update_pool_data(&mut self, block: Block<Transaction>, query_contract: Arc<FlashBotsUniV2Query<Provider<Http>>>) {
	self.pools = Pool::check_and_update((*self.pools).to_vec(), &self.set, block, query_contract).await;
	
    }

    pub fn markets_of_token(&self, token: &Address) -> Option<Vec<Pool>> {
	
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

    fn token_out(&self, pool: usize, token_in: usize) -> &usize {
	if self.ptt[pool][0] == token_in {
	    return &self.ptt[pool][1];
	} else {
	    return &self.ptt[pool][0];
	}
    }

    pub fn path_from_indices(&self, paths: &Vec<Path>) -> Vec<SwapPath> {
	let mut swap_paths: Vec<SwapPath> = Vec::with_capacity(paths.capacity());
	for path in paths {
	    let mut swap_path = SwapPath{steps: Vec::with_capacity(path.steps.capacity()), good: true};
	    
	    for step in path.steps.iter() {
		let pool = self.pools.get(step.pool).unwrap();
		let token_in = self.itt.get(&step.token_in).unwrap();
		let token_out = self.itt.get(&step.token_out).unwrap();
		swap_path.steps.push(SwapStep{pool, token_in, token_out});
	    }
	    swap_paths.push(swap_path);
	}
	return swap_paths;

    }
    
    pub async fn full_update(&mut self, query_contract: Arc<FlashBotsUniV2Query<Provider<Http>>>) {
	let mut pools_to_update: Vec<Address> = Vec::with_capacity(self.pools.len());
	let mut indices: Vec<usize> = Vec::with_capacity(self.pools.len());
	for (i, p) in self.pools.iter().enumerate() {
	    match p {
		Pool::V2(pool) => {
		    pools_to_update.push(pool.id);
		    indices.push(i);
		}
		Pool::V3(_) => {}
		
	    }
	}
	
	let batch_size = 1000;
//	let mut pool_buffer: Vec<Address> = Vec::with_capacity(batch_size);	    
//	let mut index_buffer: Vec<usize> = Vec::with_capacity(batch_size);
	let mut start: usize = 0;
	let mut end: usize = 999;
	let total = pools_to_update.len();
	loop {
	    println!("{} {}", start, end);
	    let reserves = query_contract.get_reserves_by_pairs((pools_to_update[start..end]).to_vec()).call().await.unwrap();
	    for (i, pool_index) in indices[start..end].iter().enumerate() {
		match &mut self.pools[*pool_index] {
		    Pool::V2(pool) => {
			pool.token0.reserves = reserves[i][0];
			pool.token1.reserves = reserves[i][1];
		    }
		    Pool::V3(_) => {}
		}
	    }
	    
	    start = end;
	    
	    if start == total {
		break;
	    }
	    if end + batch_size < total {
		
		end += batch_size;
	    } else {
		end = total;
	    }

	   
	}
	    
	    //	    println!("fetching reserves");
	    
	//     for _ in 0..batch_size {
	// 	if let Some(pool) = pools_to_update.pop() {
	// 	    let index = indices.pop().unwrap();
	// 	    pool_buffer.push(pool);
	// 	    index_buffer.push(index);
	// 	} else {
	// 	    break 'dance;
	// 	}

	//     }
	    
	//     let reserves = query_contract.get_reserves_by_pairs(pool_buffer.clone()).call().await.unwrap();
	//     for i in 0..batch_size {
	// 	assert_eq!(pool_buffer[i], *self.pools[index_buffer[i]].addr());
	// 	match &mut self.pools[index_buffer[i]] {
	// 	    Pool::V2(p) => {
	// 		p.token0.reserves = reserves[i][0];
	// 		p.token1.reserves = reserves[i][1];
			
	// 	    }
	// 	    Pool::V3(_) => {}
	// 	}
	//     }

	//     pool_buffer.clear();
	//     index_buffer.clear();
	    
	// }
	
    }


    pub async fn save_pools(&self, b: U64) -> std::io::Result<()> {
	crate::pool::PoolSave::save(self.pools.clone(), b).await?;
	Ok(())
    }
} 
#[derive(Debug, Clone)]
pub struct SwapStep<'a> {
    pub pool: &'a Pool,
    pub token_in: &'a Address,
    pub token_out: &'a Address,
}
#[derive(Debug, Clone)]
pub struct SwapPath <'a> {
    pub steps: Vec<SwapStep<'a>>,
    pub good: bool

}



impl<'a> SwapPath <'a>{
    #[allow(unused_assignments)]
    pub async fn swap_along_path(&self, initial_amount_in: U256) -> U256 {
	let mut amount_in: U256 = initial_amount_in;
	let mut amount_out: U256 = ZERO;
	for step in self.steps.iter() {
	    let zf1: bool = step.token_in == step.pool.token0();
	    match step.pool {
		Pool::V2(pool) => {
		    amount_out = pool.get_amount_out(zf1, amount_in).await;
		    if amount_out == ZERO {
			return ZERO;
		    }
		    amount_in = amount_out;
		}
		Pool::V3(_) => {
		    amount_out = ZERO;
		    return ZERO;
		}
	    }
	}
	return amount_out;
    }
    
    #[allow(unused_assignments)]           //amount_in, amount_out
    pub async fn maximize_profit(&self) -> (U256, U256) {

	let ten_15 = U256::from_dec_str("1000000000000000").unwrap();
	let ten_16 = U256::from_dec_str("10000000000000000").unwrap();
	let ten_17 = U256::from_dec_str("100000000000000000").unwrap();
	let ten_18 = U256::from_dec_str("1000000000000000000").unwrap();
	let ten_19 = U256::from_dec_str("10000000000000000000").unwrap();
	let ten_20 = U256::from_dec_str("100000000000000000000").unwrap();
	let mut most = ZERO;
	let initial_amounts = [ten_15, ten_16, ten_17, ten_18, ten_19, ten_20];
	for (i, amount) in initial_amounts.iter().enumerate() {
	    let amount_out = self.swap_along_path(*amount).await;

	    if amount_out == ZERO {
		return (ZERO, ZERO);
	    } else if &amount_out > &most {
		most = amount_out;
	    } else if &amount_out < &most {
		return (initial_amounts[i - 1], most);
	    }
	}
	return (initial_amounts[5], most);
    }	
	// let mut amount_out_last: U256 = ZERO;
	// let mut amount_in_last: U256 = ZERO;
	// let tolerance: U256 = U256{0:[1000,0,0,0]};
	// let mut delta = U256::from_dec_str("1000000000000000").unwrap();

	// let mut amount_in = initital_amount_in;
	// loop {
	// let amount_out = self.swap_along_path(amount_in).await;

	// if amount_out > amount_out_last {
	//     let diff = amount_out.checked_sub(amount_out_last).unwrap();
	//     if diff < tolerance {
	// 	return (amount_in, amount_out);
	//     } else {
	// 	amount_in_last = amount_in;
	// 	amount_in = amount_in.checked_add(delta).unwrap();
	//     }
	//     amount_out_last = amount_out;
	// } else {
	//     amount_in = amount_in_last;
	//     match delta.checked_div(U256{0:[2,0,0,0]}) {
	// 	Some(r) => {delta = r}
	// 	None => {return (ZERO, ZERO);}
	//     }
	// }
	// }
	// loop {

	//     let mut amount_out: U256 = ZERO;

	//     for step in self.steps.iter() {
	// 	let zf1: bool = step.token_in == step.pool.token0();
	// 	match step.pool {
	// 	    Pool::V2(pool) => {
	// 		amount_out = pool.get_amount_out(zf1, amount_in).await;
	// 		if amount_out == ZERO {
	// 		    return (ZERO, ZERO);
	// 		}
	// 		amount_in = amount_out;
	// 	    }
	// 	    Pool::V3(_) => {
	// 		amount_out = ZERO;
	// 		return (ZERO, ZERO);
	// 	    }
		    
	// 	}
		
	//     }
	    

	//     if amount_out > amount_out_last {
	// 	let diff = amount_out.checked_sub(amount_out_last).unwrap();
	// 	if diff < tolerance {
	// 	    return (amount_in, amount_out);
	// 	}
	// 	amount_out_last = amount_out
	//     } else {
	// 	return (amount_in_last, amount_out_last);
	//     }
	    
	//     amount_in_last = amount_in;
	//     amount_out_last = amount_out;
	//     amount_in = amount_in.checked_add(delta).unwrap();
	    
	// } 
    //}

    pub async fn check_path(&mut self) {
	for step in self.steps.iter() {
	    let r = match step.pool {
		Pool::V2(pool) => {
		    pool.get_amount_out(true, U256{0:[1000000,0,0,0]}).await
		}
		Pool::V3(_) => {ZERO}
	    };

	    if r == ZERO {
		self.good = false;
		break;
	    }
	    
	}
    }
}

#[derive(Clone)]
pub struct Path {
    steps: Vec<PathStep>
}

impl Path {
    fn contains(&self, pool: &usize) -> bool {
	let mut r = false;
	for step in self.steps.iter() {
	    if &step.pool == pool {
		r = true;
		break;
	    }
	}
	return r;
    }

    fn used_token(&self, token: &usize) -> bool {
	let mut r = false;
	for step in self.steps.iter() {
	    if &step.token_out == token {
		r = true;
		break;
	    }
	}
	return r;
    }

}

#[derive(Clone)]
struct PathStep {
    pool: usize,
    token_in: usize,
    token_out: usize,
}

#[allow(unused_assignments)]
pub async fn find_path(graph: Arc<Graph<Pool, Address, usize>>, start: &Address, finish: &Address) -> Option<Vec<Path>> {

    let (stack_push, stack_pop) = unbounded();
    let (found_paths, path_receiver) = unbounded();

    let target_index: usize;
    if let Some(t) = graph.tokens.get(finish) {
	target_index = *t;
    } else {
	return None;
    }
    
    let token_index: usize;
    if let Some(t) = graph.tokens.get(start) {
	token_index = *t;
    } else {
	return None;
    }

    let pool_indices: Vec<usize>;
    if let Some(p) = graph.ttp.get(&token_index) {
	pool_indices = p.clone();
    } else {
	return None;
    }


    const THREAD_COUNT: usize = 1024;
    let mut handels = Vec::with_capacity(THREAD_COUNT);


    let mut count = 0;
    
    for pool_index in pool_indices.iter() {
	let step = PathStep{pool: *pool_index, token_in: token_index, token_out: *graph.token_out(*pool_index, token_index)};
	if step.token_out == target_index {
	    found_paths.send(Path{steps: vec![step]}).unwrap();
	} else {
	    let mut steps: Vec<PathStep> = Vec::with_capacity(MAXLEN);
	    steps.push(step);
	    stack_push.send(Path{steps}).unwrap();
	}

	if count < THREAD_COUNT {
	    let s_pop = stack_pop.clone();
	    let s_push = stack_push.clone();
	    let f_paths = found_paths.clone();
	    let g = Arc::clone(&graph);
	    let h = tokio::spawn(async move {search(g, s_pop, s_push, f_paths, target_index)});
	    handels.push(h);
	    count += 1;
	}

    }

    if count < THREAD_COUNT {
	let s_pop = stack_pop.clone();
	let s_push = stack_push.clone();
	let f_paths = found_paths.clone();
	let g = Arc::clone(&graph);
	let h = tokio::spawn(async move {search(g, s_pop, s_push, f_paths, target_index)});
	handels.push(h);
	count += 1;
    }
	

    for h in handels {
	let _ = tokio::join!(h);
    }



  //  println!("{}", path_receiver.len());
    
    let mut fp: Vec<Path> = Vec::with_capacity(path_receiver.len());
    loop {
	if path_receiver.len() == 0 {
	    break;
	} else {
	    let p = path_receiver.recv().unwrap();
	    fp.push(p);
	}
    }
    //println!("{}", fp.len());
    return Some(fp);
}



fn search<'a>(graph: Arc<Graph<Pool, Address, usize>>, stack_pop: Receiver<Path>, stack_push: Sender<Path>, found_paths: Sender<Path>, target_index: usize) {
//    println!("hello");
    loop {
//	println!("hi");
	
	let path: Path;
	match  stack_pop.try_recv() {
	    Ok(p) => {path = p}
	    Err(_) => {break}
	}
	let last = path.steps.last().unwrap();

	let pools: &Vec<usize>;
	if let Some(p) = graph.ttp.get(&last.token_out) {
	    pools = p;
	} else {
	    break;
	}
	for pool in pools.iter() {
	    if !path.contains(pool) {
		let token_out = graph.token_out(*pool, last.token_out);
		if !path.used_token(token_out) {
		    let mut p = path.clone();
		    let step = PathStep{pool: *pool, token_in: last.token_out, token_out: *token_out};
		    p.steps.push(step);
		    if p.steps.last().unwrap().token_out == target_index {
			found_paths.send(p).unwrap();
		    } else {
			if p.steps.len() < MAXLEN {
			    stack_push.send(p).unwrap();
			}
		    }
		}
		
	    }
	}
    }
//    println!("goodbye");
}
