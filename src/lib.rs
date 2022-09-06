pub mod univ2pool;
pub mod univ3pool;
pub mod poolgraph;
pub mod constants;
pub mod pool;
pub mod calculator;

#[cfg(test)]
mod tests {
    use crate::{univ2pool::{UniV2Pool, FlashBotsUniV2Query}, constants::SUSHI_FAC};


    #[tokio::test]
    async fn univ2calc() {
	use ethers::{
	    providers::{Provider, Http},
	    types::{Address, U256},
	    contract::abigen,
	};
	use std::sync::Arc;

	abigen!(UniV2Router, "etherscan:0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D");
	
	let provider = Provider::<Http>::try_from("https://mainnet.infura.io/v3/cb7a603124c4411ba12877599e494814").unwrap();
	let client = Arc::new(provider);

	let uni_v2_router = "0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D".parse::<Address>().unwrap();
	let router = Arc::new(UniV2Router::new(uni_v2_router, Arc::clone(&client)));

	let query_addr = "0x5EF1009b9FCD4fec3094a5564047e190D72Bd511".parse::<Address>().unwrap();
	let query = Arc::new(FlashBotsUniV2Query::new(query_addr, Arc::clone(&client)));
	let amount = U256::from_dec_str("1000000000").unwrap();

	let pools = UniV2Pool::from_flash(SUSHI_FAC, Arc::clone(&query)).await;
	
	let r1 = pools[0].get_amount_out(true, amount).await;
	//reserveIn = reserve1
	//reserveOut = reserve0
	let r2 = router.get_amount_out(amount, pools[0].token1.reserves, pools[0].token0.reserves).call().await.unwrap();
	assert_eq!(r1, r2);

	let r3 = pools[0].get_amount_out(false, amount).await;
	let r4 = router.get_amount_out(amount, pools[0].token0.reserves, pools[0].token1.reserves).call().await.unwrap();
	assert_eq!(r3, r4);

	// let r5 = pools[0].get_amount_in(true, amount).await;
	// let r6 = router.get_amount_in(amount, pools[0].token1.reserves, pools[0].token0.reserves).call().await.unwrap();
	// assert_eq!(r5, r6);

	let r7 = pools[0].get_amount_in(false, amount).await;
	let r8 = router.get_amount_in(amount, pools[0].token0.reserves, pools[0].token1.reserves).call().await.unwrap();
	assert_eq!(r7, r8);

	
	
	
    }
}
