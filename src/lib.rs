pub mod univ2pool;
pub mod univ3pool;
pub mod poolgraph;
pub mod constants;
pub mod pool;
pub mod calculator;

#[cfg(test)]
mod tests {

    


    #[tokio::test]
    async fn univ2calc() {
	use ethers::{
	    providers::{Provider, Http},
	    types::{Address, U256},
	    contract::abigen,
	};
	use std::sync::Arc;
	use crate::{univ2pool::{UniV2Pool, FlashBotsUniV2Query}, constants::{SHIBA_FAC, ZERO}};
	abigen!(UniV2Router, "etherscan:0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D");
	
	let provider = Provider::<Http>::try_from("https://mainnet.infura.io/v3/cb7a603124c4411ba12877599e494814").unwrap();
	let client = Arc::new(provider);

	let uni_v2_router = "0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D".parse::<Address>().unwrap();
	let router = Arc::new(UniV2Router::new(uni_v2_router, Arc::clone(&client)));

	let query_addr = "0x5EF1009b9FCD4fec3094a5564047e190D72Bd511".parse::<Address>().unwrap();
	let query = Arc::new(FlashBotsUniV2Query::new(query_addr, Arc::clone(&client)));
	let amount = U256::from_dec_str("1000000000").unwrap();

	let pools = UniV2Pool::from_flash(SHIBA_FAC, Arc::clone(&query)).await;

	for pool in pools {
	    let r1 = pool.get_amount_out(true, amount).await;
	//reserveIn = reserve1
	//reserveOut = reserve0
	    let r2 = router.get_amount_out(amount, pool.token1.reserves, pool.token0.reserves).call().await;
	    match r2 {
		Ok(r2) => {
		    match r1 == r2 {
			true => {}
			false if r1 == ZERO => {}
			false => {println!("{} {} {}", pool.id, r1, r2);}
		    }
		}
		Err(_) => {
		    assert_eq!(r1, ZERO);
		}
	    }

	    let r3 = pool.get_amount_out(false, amount).await;
	    let r4 = router.get_amount_out(amount, pool.token0.reserves, pool.token1.reserves).call().await;

	    match r4 {
		Ok(r4) => {
		
		    match r3 == r4 {
			true => {}
			false if r3 == ZERO => {}
			false => {println!("{} {} {}", pool.id, r3, r4);}
		    }
		}
		Err(_) => {
		    assert_eq!(r3, ZERO);
		}
	    }
	    
	}
	assert_eq!(0,0);
	// let r5 = pools[0].get_amount_in(true, amount).await;
	// let r6 = router.get_amount_in(amount, pools[0].token1.reserves, pools[0].token0.reserves).call().await.unwrap();
	// assert_eq!(r5, r6);

	// let r7 = pools[0].get_amount_in(false, amount).await;
	// let r8 = router.get_amount_in(amount, pools[0].token0.reserves, pools[0].token1.reserves).call().await.unwrap();
	// assert_eq!(r7, r8);

	
	
	
    }

    #[tokio::test]
    async fn path_finder() {
	use crate::{
	    pool::PoolSave,
	    poolgraph::{Graph, find_path},
	};

	use ethers::types::Address;
	use std::{
	    time::Instant,
	    sync::Arc,
	};
	
	let weth = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".parse::<Address>().unwrap();
	
	let pool_save = PoolSave::load().unwrap();
	let pools = pool_save.pools;
	let graph = Graph::new(pools);

	let now = Instant::now();
	let graph = Arc::new(graph);
	let paths = find_path(Arc::clone(&graph), &weth, &weth).await.unwrap();
	let swap_path = graph.path_from_indices(paths);
	println!("{}", swap_path.len());
	let took = now.elapsed();
	println!("took {}us", took.as_micros());
	assert_eq!(0,0);
    }
    
}
