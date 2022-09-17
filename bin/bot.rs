#![allow(dead_code, unused_imports, unused_variables, unreachable_code)]
use pathfinder::{
    pool::{Pool, load_pools, save_pools, load_pools_from_save, PoolSave},
    univ3pool::UniV3Calc,
    univ2pool::{UniV2Pool, UniV2Calc, FlashBotsUniV2Query},
    poolgraph::{Graph, SwapPath, find_path, decode_and_test_path},
    calculator::Calculator,
    constants::ZERO,
};
use std::{sync::Arc, time::Instant};
use ethers::{
    prelude::*,
    types::{Address, H160, U256, U64,Transaction, Block, Bytes},
    providers::{Provider, Http, Middleware, StreamExt, call_raw::RawCall},
    contract::ContractFactory,
};

use tokio::time::{sleep, Duration};
use hex_literal::hex;
use crossbeam::channel::unbounded;
use rayon::prelude::*;


#[tokio::main]
async fn main() -> eyre::Result<()> {
    let provider = Provider::<Http>::try_from("https://mainnet.infura.io/v3/cb7a603124c4411ba12877599e494814")?;
    let client = Arc::new(provider);

    
    let query_addr = "0x5EF1009b9FCD4fec3094a5564047e190D72Bd511".parse::<Address>().unwrap();
    let query = Arc::new(FlashBotsUniV2Query::new(query_addr, Arc::clone(&client)));
    
    let (tx, rx) = unbounded();
    let c1 = Arc::clone(&client);
//    let tx1 = tx.clone();

    let weth: H160 = H160(hex!("C02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"));
    let yfi: H160 = H160(hex!("0bc529c00C6401aEF6D220BE8C6Ea1667F6Ad93e"));
    let wbtc: H160 = H160(hex!("2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599"));

    // println!("loading pools!");
    // let block_num = client.get_block_number().await?;
    // let pools = load_pools(Arc::clone(&query)).await.unwrap();
    // PoolSave::save(pools, block_num).await?;
    
    let pool_save: PoolSave = PoolSave::load()?;
    let pools = pool_save.pools;

    let mut my_block_number = pool_save.block;
    let blocks_behind = client.get_block_number().await? - my_block_number;

    let full_update: bool = blocks_behind > U64{0:[100]};
    
  
    println!("making graph");
    let mut graph = Graph::new(pools);

    match full_update {
	true => {
	    println!("full update!");
	    graph.full_update(Arc::clone(&query)).await;
	    println!("saving pools");
	    graph.save_pools(client.get_block_number().await.unwrap()).await.unwrap();
	}
	false => {
	    println!("not full update!");
	    loop {

		if my_block_number != client.get_block_number().await? {
		    my_block_number = my_block_number.overflowing_add(U64::one()).0;	    
		    let b = client.get_block_with_txs(my_block_number).await.unwrap().unwrap();
		    graph.update_pool_data(b, Arc::clone(&query)).await;
		} else {
		    graph.save_pools(my_block_number).await?;
		    break;
		}
	    }
	    
	}
	
    }
    
    tokio::spawn(async move {
	let mut stream = c1.watch_blocks().await.unwrap();
	while let Some(b) = stream.next().await {
	   
	    let block = c1.get_block_with_txs(b).await.unwrap().unwrap(); // one of these unwraps panic
	    tx.send(block).unwrap();
	    
	}
    });
    let graph = graph;
    let mut graph = Arc::new(graph);
    
    let wethweth = find_path(Arc::clone(&graph),&weth, &weth).await.unwrap();
    let wbtcwbtc = find_path(Arc::clone(&graph),&wbtc, &wbtc).await.unwrap();
    let yfiyfi = find_path(Arc::clone(&graph), &yfi, &yfi).await.unwrap();
    let count = wethweth.len() + wbtcwbtc.len() + yfiyfi.len();
    let allpaths = vec![wethweth,wbtcwbtc,yfiyfi];
    loop {    
	println!("awaiting new block");
	let new_block = rx.recv().unwrap();
	let now = Instant::now();
	match Arc::get_mut(&mut graph) {
	    Some(g) => {g.update_pool_data(new_block, Arc::clone(&query)).await;}
	    None => {}
	}    
	let took = now.elapsed();
	println!("info updated took {}ms", took.as_millis());
	let now_all_paths = Instant::now();
	for paths in allpaths.iter() {
	    let now = Instant::now();
	    let swap_paths = decode_and_test_path(Arc::clone(&graph), paths).await;	
	    let took = now.elapsed();
	    println!("path from indices took {}ms", took.as_millis());
	
	//     let now = Instant::now();
	//     for path in swap_paths.iter() {

	// 	let (ai, ao) = path.maximize_profit().await;
	// 	if ao > ai {
	// 	    println!("{:#?}", path);
	// 	    println!("{}", ai);
	// 	    println!("{}", ao);
	// 	}
	//     }
	}
	let took_all_paths = now_all_paths.elapsed();
	println!("all paths took {}ms ", took_all_paths.as_millis());
	println!("{}", count);
	// let took = now.elapsed();
	// println!("took {}ms to find best", took.as_millis());
    }


//     let provider = Provider::<Http>::try_from("https://mainnet.infura.io/v3/cb7a603124c4411ba12877599e494814")?;
//     let client = Arc::new(provider);

//     let query_addr = "0x5EF1009b9FCD4fec3094a5564047e190D72Bd511".parse::<Address>().unwrap();
//     let query = Arc::new(FlashBotsUniV2Query::new(query_addr, Arc::clone(&client)));


//     let weth = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".parse::<Address>().unwrap();
// //    let target = "0xc40d16476380e4037e6b1a2594caf6a6cc8da967".parse::<Address>().unwrap();

//   //  let target_eth = U256::from_dec_str("857438959604129380483")?;

//     let pool_save = PoolSave::load().unwrap();
//     let pools = pool_save.pools;
//     let graph = Graph::new(pools);
    
//     // let mut  graph = Graph::new(pools);
//     // graph.full_update(query).await;
//     // let pools = graph.pools.clone();
//     // PoolSave::save(pools, client.get_block_number().await.unwrap()).await.unwrap();

//     let now = Instant::now();
//     let graph = Arc::new(graph);
//     let paths = find_path(Arc::clone(&graph), &weth, &weth).await.unwrap();
//     let swap_path = graph.path_from_indices(&paths);
//     println!("{}", swap_path.len());
//     let took = now.elapsed();
//     println!("took {}ms", took.as_millis());

//     let amount_in = U256::from_dec_str("1000000000000000").unwrap();

//     //let mut most = ZERO;
//     //let mut best: &SwapPath = &swap_path[0]; 
//     let now = Instant::now();
//     for path in swap_path.iter() {
// 	let (ai, ao) = path.maximize_profit().await;
// 	if ao > ai {
// 	    println!("{:#?}", path);
// 	    println!("{}", ai);
// 	    println!("{}", ao);
// 	}
//     }
    
//     // for path in swap_path.iter() {
//     // 	let amount_out = path.swap_along_path(amount_in).await;
//     // 	if amount_out > most {
//     // 	    most = amount_out;
//     // 	    best = path;
//     // 	}   
//     // }
//     let took = now.elapsed();
//     // println!("{}", amount_in);
//     // println!("{}", most);
//     // println!("{:#?}", best);
//     println!("took {}ms to find best", took.as_millis());
    Ok(())
}
