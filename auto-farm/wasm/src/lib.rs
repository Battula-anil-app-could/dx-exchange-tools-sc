// Code generated by the dharitri-sc multi-contract system. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

// Init:                                 1
// Endpoints:                           29
// Async Callback (empty):               1
// Total number of exported functions:  31

#![no_std]

// Configuration that works with rustc < 1.73.0.
// TODO: Recommended rustc version: 1.73.0 or newer.
#![feature(lang_items)]

dharitri_sc_wasm_adapter::allocator!();
dharitri_sc_wasm_adapter::panic_handler!();

dharitri_sc_wasm_adapter::endpoints! {
    auto_farm
    (
        init => init
        changeProxyClaimAddress => change_proxy_claim_address
        register => register
        withdrawAllAndUnregister => withdraw_all_and_unregister
        depositFarmTokens => deposit_farm_tokens
        withdrawAllFarmTokens => withdraw_all_farm_tokens_endpoint
        withdrawSpecificFarmTokens => withdraw_specific_farm_tokens_endpoint
        getUserFarmTokens => get_user_farm_tokens_view
        depositMetastakingTokens => deposit_metastaking_tokens
        withdrawAllMetastakingTokens => withdraw_all_metastaking_tokens_endpoint
        withdrawSpecificMetastakingTokens => withdraw_specific_metastaking_tokens_endpoint
        getUserMetastakingTokens => get_user_metastaking_tokens_view
        claimAllRewardsAndCompound => claim_all_rewards_and_compound
        userClaimRewards => user_claim_rewards_endpoint
        getUserRewards => get_user_rewards_view
        claimFees => claim_fees
        getFeePercentage => fee_percentage
        getAccumulatedFees => accumulated_fees
        setEnergyFactoryAddress => set_energy_factory_address
        getEnergyFactoryAddress => energy_factory_address
        addFarms => add_farms
        removeFarms => remove_farms
        getFarmForFarmToken => get_farm_for_farm_token_view
        getFarmForFarmingToken => get_farm_for_farming_token_view
        getFarmConfig => get_farm_config
        addMetastakingScs => add_metastaking_scs
        removeMetastakingScs => remove_metastaking_scs
        getMetastakingForDualYieldToken => get_metastaking_for_dual_yield_token_view
        getMetastakingForLpFarmToken => get_metastaking_for_lp_farm_token
        getMetastakingConfig => get_metastaking_config
    )
}

dharitri_sc_wasm_adapter::async_callback_empty! {}