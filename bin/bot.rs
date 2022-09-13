#![allow(dead_code, unused_imports, unused_variables, unreachable_code)]
use pathfinder::{
    pool::{Pool, load_pools, save_pools, load_pools_from_save, PoolSave},
    univ3pool::UniV3Calc,
    univ2pool::{UniV2Pool, UniV2Calc, FlashBotsUniV2Query},
    poolgraph::{Graph, SwapPath},
    calculator::Calculator,
    constants::ZERO,
};
use std::{sync::Arc, time::Instant};
use ethers::{
    types::{Address, U256, U64,Transaction, Block},
    providers::{Provider, Http, Middleware, StreamExt},
};

use tokio::time::{sleep, Duration};

use crossbeam::channel::unbounded;



#[tokio::main]
async fn main() -> eyre::Result<()> {
//     let provider = Provider::<Http>::try_from("https://mainnet.infura.io/v3/cb7a603124c4411ba12877599e494814")?;
//     let client = Arc::new(provider);

//     let query_addr = "0x5EF1009b9FCD4fec3094a5564047e190D72Bd511".parse::<Address>().unwrap();
//     let query = Arc::new(FlashBotsUniV2Query::new(query_addr, Arc::clone(&client)));
    
//     let (tx, rx) = unbounded();
//     let c1 = Arc::clone(&client);
// //    let tx1 = tx.clone();

//     let weth = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".parse::<Address>().unwrap();

//     println!("loading pools!");
// //    let block_num = client.get_block_number().await?;
// //    let pools = load_pools(Arc::clone(&query)).await.unwrap();
// //    PoolSave::save(pools, block_num).await?;
// //    panic!();
//     let pool_save: PoolSave = PoolSave::load()?;
//     let pools = pool_save.pools;

//     let mut my_block_number = pool_save.block;
//     let blocks_behind = client.get_block_number().await? - my_block_number;

//     let full_update: bool = blocks_behind > U64{0:[100]};

//     // if re_save {

//     // } else {

//     // }
    
  
//     println!("making graph");
//     let mut graph = Graph::new(pools);

//     match full_update {
// 	true => {
// 	    println!("full update!");
// 	    graph.full_update(Arc::clone(&query)).await;
// 	    println!("saving pools");
// 	    graph.save_pools(client.get_block_number().await.unwrap()).await.unwrap();
// 	}
// 	false => {
// 	    println!("not full update!");
// 	    loop {

// 		if my_block_number != client.get_block_number().await? {
// 		    my_block_number = my_block_number.overflowing_add(U64::one()).0;	    
// 		    let b = client.get_block_with_txs(my_block_number).await.unwrap().unwrap();
// 		    graph.update_pool_data(b, Arc::clone(&query)).await;
// 		} else {
// 		    graph.save_pools(my_block_number).await?;
// 		    break;
// 		}
// 	    }
	    
// 	}
	
//     }
    
//     tokio::spawn(async move {
// 	let mut stream = c1.watch_blocks().await.unwrap();
// 	while let Some(b) = stream.next().await {
// 	    let block = c1.get_block_with_txs(b).await.unwrap().unwrap();
// 	    tx.send(block).unwrap();
	    
// 	}
//     });
 

    
//     loop {    
// 	println!("awaiting new block");
// 	let new_block = rx.recv().unwrap();
// 	let now = Instant::now();
// 	graph.update_pool_data(new_block, Arc::clone(&query)).await;
// 	let took = now.elapsed();
// 	println!("info updated took {}ms", took.as_millis());
// 	let paths = graph.find_path(&weth, &weth).await.unwrap();
// 	let sp: Vec<SwapPath> = graph.path_from_indices(paths);
// 	println!("{}", sp.len());
// 	let now = Instant::now();
// 	for p in sp.iter() {
	    
// 	    let initital_amount_in = U256::from_dec_str("1000000").unwrap();
// 	    let mut amount_in = initital_amount_in;
// 	    let mut amount_out = ZERO;
// 	    for step in p.steps.iter() {
// 		let zf1: bool = step.token_in == step.pool.token0();
// 		match step.pool {
// 		    Pool::V2(pool) => {
// 			amount_out = pool.get_amount_out(zf1, amount_in).await;
// 			if amount_out == ZERO {
// 			    break;
// 			}
// 			amount_in = amount_out;
// 		    }
// 		    Pool::V3(_) => {
// 			amount_out = ZERO;
// 			break;
// 		    }
		    
// 		}

// 	    }
// 	    let profit = amount_out.saturating_sub(initital_amount_in);
// 	    if profit != ZERO {
// 		println!("{:#?}", p);
// 		println!("{}", profit);
// 		println!("");
// 	    }

	    
	    
// 	}
// 	let took = now.elapsed();
// 	println!("took {}us for all v2", took.as_micros());
// 	println!("");

    //     }
    
    
    let weth = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".parse::<Address>().unwrap();
    
    let pool_save = PoolSave::load().unwrap();
    let pools = pool_save.pools;
    let graph = Graph::new(pools);
    
    let now = Instant::now();
    let graph = Arc::new(graph);
    let paths = graph.find_path(&weth, &weth).await.unwrap();
    let swap_path = graph.path_from_indices(paths);
    println!("{}", swap_path.len());
    let took = now.elapsed();
    println!("took {}us", took.as_millis());
    Ok(())
}
