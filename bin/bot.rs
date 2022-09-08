#![allow(dead_code, unused_imports, unused_variables, unreachable_code)]
use pathfinder::{
    pool::{Pool, load_pools, save_pools, load_pools_from_save},
    univ3pool::UniV3Calc,
    univ2pool::{UniV2Pool, UniV2Calc, FlashBotsUniV2Query},
    poolgraph::{Graph, SwapPath},
    calculator::Calculator,
    constants::ZERO,
};
use std::{
    sync::Arc,
    time::Instant,
	
};
use ethers::{
    types::{Address, U256, Transaction, Block},
    providers::{Provider, Http, Middleware, StreamExt},
};

use crossbeam::channel::unbounded;



#[tokio::main]
async fn main() -> eyre::Result<()> {
    //let pools = load_pools().unwrap();

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
    let provider = Provider::<Http>::try_from("https://mainnet.infura.io/v3/cb7a603124c4411ba12877599e494814")?;
    let client = Arc::new(provider);

    let query_addr = "0x5EF1009b9FCD4fec3094a5564047e190D72Bd511".parse::<Address>().unwrap();
    let query = Arc::new(FlashBotsUniV2Query::new(query_addr, Arc::clone(&client)));
    
    let (tx, rx) = unbounded();
    let c1 = Arc::clone(&client);
    tokio::spawn(async move {
	let mut stream = c1.watch_blocks().await.unwrap();
	while let Some(b) = stream.next().await {
	    let block = c1.get_block_with_txs(b).await.unwrap().unwrap();
	    tx.send(block).unwrap();
	    
	}
    });
    
    let weth = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".parse::<Address>().unwrap();
        
    //let pools = load_pools(Arc::clone(&query)).await.unwrap();
    //save_pools(&pools)?;
    println!("loading pools!");
    let pools = load_pools_from_save()?;
    println!("making graph");
    let mut graph = Graph::new(pools);
    println!("awaiting new block");
    loop {    
	let new_block = rx.recv().unwrap();
	let now = Instant::now();
	graph.update_pool_data(new_block, Arc::clone(&query)).await;
	let took = now.elapsed();
	println!("info updated took {}", took.as_millis());
	let paths = graph.find_path(&weth, &weth).await.unwrap();
	let sp: Vec<SwapPath> = graph.path_from_indices(paths);
	println!("{}", sp.len());
	let now = Instant::now();
	for p in sp.iter() {
	    
	    let initital_amount_in = U256::from_dec_str("1000000").unwrap();
	    let mut amount_in = initital_amount_in;
	    let mut amount_out = ZERO;
	    'dance: for step in p.steps.iter() {
		let zf1: bool = step.token_in == step.pool.token0();
		match step.pool {
		    Pool::V2(pool) => {
			amount_out = pool.get_amount_out(zf1, amount_in).await;
			if amount_out == ZERO {
			    break;
			}
			amount_in = amount_out;
		    }
		    Pool::V3(_) => {break 'dance}
		    
		}

	    }
	    let profit = amount_out.saturating_sub(initital_amount_in);
	    if profit != ZERO {
		println!("{:#?}", p);
		println!("{}", profit);
		println!("");
	    }
	    
	}
	let took = now.elapsed();
	println!("took {} for all v2", took.as_micros());
	println!("");
	
	}
	
	

    
    

    

//     let pools: Vec<Pool>;
//     if let Ok(p) = load_pools(query).await {
// 	pools = p;
//     } else {
// 	panic!();
//     }
//     let amount = U256::from_dec_str("1000000000").unwrap();
//     let now = Instant::now();
//     for pool in pools.iter() {
// 	match pool {
// 	    Pool::V2(p) => {
// 		let t = p.get_amount_out(true, amount).await;
// 		let f = p.get_amount_out(false, amount).await;
// //		println!("{} {}", t, f);
// 		let tt = p.get_amount_in(true, t).await;
// 		let ff = p.get_amount_in(false, f).await;
// 		//		
// 		if t != ZERO && f != ZERO && tt != ZERO && ff != ZERO {
// 		    println!("{} {}", t, f);
// 		    println!("{} {}", tt, ff);
// 		    println!("");
// 		} else {
// 		    continue;
// 		}
		
// 	    }
// 	    Pool::V3(_) => {}
// 	}

//     }
//     let took = now.elapsed();
//     println!("{}", took.as_millis());
    // let fac_addrs = vec![shiba_fac];
    
    // let r = UniV2Pool::flash_all_factorys(fac_addrs, Arc::clone(&query)).await;
    // println!("{}", r.len());
    // let sushi_fac = "0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f".parse::<Address>().unwrap();
    // println!("{:#?}", sushi_fac.0);


    

    //let v2_address = "0x9E4c14403d7d9A8A782044E86a93CAE09D7B2ac9";
    //let v3_address = "0xcCB53c9429d32594F404d01fbe9E65ED1DCda8D9";
    
    //let calc = Calculator::new(v2_address, v3_address, Arc::clone(&client));
    //let uni_v2_calc_addr = "0x9E4c14403d7d9A8A782044E86a93CAE09D7B2ac9".parse::<Address>().unwrap();
    //let uni_v3_calc_addr = "0xcCB53c9429d32594F404d01fbe9E65ED1DCda8D9".parse::<Address>().unwrap();
    
    
    //let uni_v2_calc = Arc::new(UniV2Calc::new(uni_v2_calc_addr, Arc::clone(&client)));
    //let uni_v3_calc = Arc::new(UniV3Calc::new(uni_v3_calc_addr, Arc::clone(&client)));

    // let amount = U256::from_dec_str("1000000000000000000").unwrap();
    // let t = Instant::now();
    // let graph = Graph::new(pools.clone());
    // let took = t.elapsed();
    // println!("took {}ms to build graph", took.as_millis());


    // 
    //let yfi = "0x0bc529c00C6401aEF6D220BE8C6Ea1667F6Ad93e".parse::<Address>().unwrap();
    //let dai = "0x6B175474E89094C44Da98b954EedeAC495271d0F".parse::<Address>().unwrap();
    // for i in graph.clone().tokens.keys() {
    // 	for j in graph.clone().tokens.keys() {
    // 	    let t = Instant::now();
    // 	    let paths = graph.find_path(i, j).await.unwrap();
    // 	    let gt = t.elapsed();
    // 	    println!("{} ---> {}", i, j);
    // 	    println!("took {}ms to search finding {} paths", gt.as_millis(), paths.len());
    // 	}
    // }
    // let t = Instant::now();
    // let g = graph.find_path(&weth, &weth).await.unwrap();
    // let gt = t.elapsed();

    // let t = Instant::now();
    // let allpaths = graph.path_from_indices(g);
    // let pt = t.elapsed();
    // println!("{}", allpaths.len());


    // let token_in = pools[0].token0();

    // let r = calc.amount_out_for_an_input(&pools[0], true, amount).await;
    // println!("{}", r);
    
    // let amount = U256::from_dec_str("4091674361126346").unwrap();
    // let r = calc.amount_in_for_an_output(&pools[0], true, amount).await;
    // println!("{}", r);
    
    

    
    // for i in 0..pools.len() {
    // 	let token_in = pools[i].token0();
    // 	let v2 = Arc::clone(&uni_v2_calc);
    // 	let v3 = Arc::clone(&uni_v3_calc);
    // 	//println!("{:#?}", pools[i]);
    // 	let r = pools[i].amount_out(token_in, amount, v2, v3).await;
    // 	println!("{}", r);
    // }

    // /* 
    // for loops through token space. ie start_token == end_token
    //  */
    // for i in 0..allpaths.len() {
    // //for (index, path) in allpaths.iter().enumerate() {
    // 	let path = &allpaths[i];
    // 	let mut amount_in = amount;
    // 	for step in path.steps.iter() {
	    
    // 	    let amount_out = step.pool.amount_out(step.token_in, amount_in, Arc::clone(&uni_v2_calc), Arc::clone(&uni_v3_calc)).await;
    // 	    if amount_out == ZERO {
    // 		break;
    // 	    }
    // 	    amount_in = amount_out;
	    
    // 	}
    // 	let prof = amount_in.saturating_sub(amount);
    // 	println!("{}", i);
    // 	if prof != ZERO {
    // 	    println!("{:#?}", path.steps);
    // 	    println!("{:?}", prof);
    // 	    println!("");
    // 	}
	
    // }

    // println!("{} {}", gt.as_micros(), pt.as_micros());


    //println!("took {}ms to search finding {} paths", gt.as_millis(), g.len());

    
    

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
