
export let view_methods_lib_contract = [
	// Get number of tokens for a swap.
	['price_near_to_token_in'],    
	['price_near_to_token_out'], 

	['price_token_to_near_in'],   
	['price_token_to_near_out'],  

	['price_token_to_token_in'],  
	['price_token_to_token_out'],  
	
	// Pool Info
	['pool_info'],
	['get_pool'],
	['calc_out_amount'],
	['calc_in_amount']

	];

// TODO
// Gas estimation
// Approval transaction methods

export let change_methods_lib_contract = [
	// pool
	['create_pool'],
	['set_pool'],
	
	['add_liquidity'],
	['remove_liquidity'],

	// Swap near to token
	['swap_near_to_reserve_exact_in'],
	['swap_near_to_reserve_exact_in_xfr'],
	['swap_near_to_reserve_exact_out'],
	['swap_near_to_reserve_exact_out_xfr'],

	// Swap token to near
	['swap_reserve_to_near_exact_in'],
	['swap_reserve_to_near_exact_in_xfr'],
	['swap_reserve_to_near_exact_out'],
	['swap_reserve_to_near_exact_out_xfr'],

	// Swap token to token
	['swap_tokens_exact_in'],
	['swap_tokens_exact_in_xfr'],
	['swap_tokens_exact_out'],
	['swap_tokens_exact_out_xfr'],

	];
