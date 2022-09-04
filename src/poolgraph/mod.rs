#![allow(dead_code)]
use ahash::AHashMap;
use crate::{pool::Pool, constants::MAXLEN};
use ethers::types::Address;


#[derive(Clone)]
pub struct Graph<P, A, I> {
    pools: Vec<P>,
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
	    
	Graph{pools, tokens, ttp, itt, ptt}
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

    pub async fn find_path(&self, start: &Address, finish: &Address) -> Option<Vec<Path>> {
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

    pub fn path_from_indices(&self, paths: Vec<Path>) -> Vec<SwapPath> {
	let mut swap_paths: Vec<SwapPath> = Vec::with_capacity(paths.capacity());
	for path in paths {
	    let mut swap_path = SwapPath{steps: Vec::with_capacity(path.steps.capacity())};
	    
	    for step in path.steps {
		let pool = self.pools.get(*step.pool).unwrap();
		let token_in = self.itt.get(step.token_in).unwrap();
		let token_out = self.itt.get(step.token_out).unwrap();
		swap_path.steps.push(SwapStep{pool, token_in, token_out});
	    }
	    swap_paths.push(swap_path);
	}
	return swap_paths;

    }
}
#[derive(Debug)]
struct SwapStep<'a> {
    pool: &'a Pool,
    token_in: &'a Address,
    token_out: &'a Address,
}
#[derive(Debug)]
pub struct SwapPath <'a> {
    steps: Vec<SwapStep<'a>>
}

#[derive(Clone)]
pub struct Path <'a> {
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
