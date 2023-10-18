#![allow(deprecated)]

mod proxy_dex_test_setup;

use config::ConfigModule;
use locked_token_pos_creator::{
    create_farm_pos::CreateFarmPosModule, create_pair_pos::CreatePairPosModule,
    LockedTokenPosCreatorContract,
};
use dharitri_sc::types::{DctLocalRole, DctTokenPayment};
use dharitri_sc_scenario::{
    managed_address, managed_biguint, managed_token_id, rust_biguint,
    whitebox_legacy::TxTokenTransfer, DebugApi,
};
use num_traits::ToPrimitive;
use proxy_dex::{
    proxy_pair::ProxyPairModule, wrapped_farm_attributes::WrappedFarmTokenAttributes,
    wrapped_lp_attributes::WrappedLpTokenAttributes,
};
use proxy_dex_test_setup::*;
use sc_whitelist_module::SCWhitelistModule;

#[test]
fn setup_test() {
    let _ = ProxySetup::new(
        proxy_dex::contract_obj,
        pair::contract_obj,
        farm_with_locked_rewards::contract_obj,
        energy_factory::contract_obj,
    );
}

#[test]
fn create_pair_and_farm_pos_test() {
    let proxy_dex_setup = ProxySetup::new(
        proxy_dex::contract_obj,
        pair::contract_obj,
        farm_with_locked_rewards::contract_obj,
        energy_factory::contract_obj,
    );

    #[allow(clippy::redundant_clone)] // clippy is dumb
    let b_mock = proxy_dex_setup.b_mock.clone();
    let pos_creator_wrapper = b_mock.borrow_mut().create_sc_account(
        &rust_biguint!(0),
        Some(&proxy_dex_setup.owner),
        locked_token_pos_creator::contract_obj,
        "random path ssss",
    );

    b_mock
        .borrow_mut()
        .execute_tx(
            &proxy_dex_setup.owner,
            &pos_creator_wrapper,
            &rust_biguint!(0),
            |sc| {
                sc.init(
                    managed_address!(proxy_dex_setup.simple_lock_wrapper.address_ref()),
                    managed_address!(proxy_dex_setup.simple_lock_wrapper.address_ref()), // not used
                    managed_token_id!(WMOAX_TOKEN_ID),
                    managed_address!(proxy_dex_setup.pair_wrapper.address_ref()),
                    managed_address!(proxy_dex_setup.farm_locked_wrapper.address_ref()),
                    managed_address!(proxy_dex_setup.proxy_wrapper.address_ref()),
                );
            },
        )
        .assert_ok();

    b_mock.borrow_mut().set_dct_local_roles(
        pos_creator_wrapper.address_ref(),
        MEX_TOKEN_ID,
        &[DctLocalRole::Burn],
    );

    b_mock
        .borrow_mut()
        .execute_tx(
            &proxy_dex_setup.owner,
            &proxy_dex_setup.simple_lock_wrapper,
            &rust_biguint!(0),
            |sc| {
                sc.add_sc_address_to_whitelist(managed_address!(pos_creator_wrapper.address_ref()));
            },
        )
        .assert_ok();

    let first_user = &proxy_dex_setup.first_user;
    let second_user = &proxy_dex_setup.second_user;
    let locked_token_amount = rust_biguint!(1_000_000_000);
    let other_token_amount = rust_biguint!(500_000_000);
    let expected_lp_token_amount = rust_biguint!(497);

    // set the price to 1 MOAX = 2 MEX
    let payments = vec![
        TxTokenTransfer {
            token_identifier: WMOAX_TOKEN_ID.to_vec(),
            nonce: 0,
            value: other_token_amount.clone(),
        },
        TxTokenTransfer {
            token_identifier: LOCKED_TOKEN_ID.to_vec(),
            nonce: 2,
            value: locked_token_amount.clone(),
        },
    ];

    // add initial liquidity
    let pair_addr = proxy_dex_setup.pair_wrapper.address_ref().clone();
    b_mock
        .borrow_mut()
        .execute_dct_multi_transfer(
            second_user,
            &proxy_dex_setup.proxy_wrapper,
            &payments,
            |sc| {
                sc.add_liquidity_proxy(
                    managed_address!(&pair_addr),
                    managed_biguint!(other_token_amount.to_u64().unwrap()),
                    managed_biguint!(locked_token_amount.to_u64().unwrap()),
                );
            },
        )
        .assert_ok();

    b_mock
        .borrow_mut()
        .execute_dct_transfer(
            &proxy_dex_setup.first_user,
            &pos_creator_wrapper,
            WMOAX_TOKEN_ID,
            0,
            &rust_biguint!(1_000),
            |sc| {
                let add_liq_result = sc.create_pair_pos_from_single_token_endpoint(
                    managed_biguint!(1u32),
                    LOCK_OPTIONS[0],
                    managed_biguint!(1u32),
                    managed_biguint!(1u32),
                );
                assert_eq!(add_liq_result.locked_token_leftover.amount, 0u64);
                assert_eq!(add_liq_result.wmoax_leftover.amount, 2u64);
                assert_eq!(add_liq_result.wrapped_lp_token.amount, 497u64);
            },
        )
        .assert_ok();

    proxy_dex_setup.b_mock.borrow().check_nft_balance(
        first_user,
        WRAPPED_LP_TOKEN_ID,
        2,
        &expected_lp_token_amount,
        Some(&WrappedLpTokenAttributes::<DebugApi> {
            locked_tokens: DctTokenPayment {
                token_identifier: managed_token_id!(LOCKED_TOKEN_ID),
                token_nonce: 1,
                amount: managed_biguint!(996u64),
            },
            lp_token_id: managed_token_id!(LP_TOKEN_ID),
            lp_token_amount: managed_biguint!(expected_lp_token_amount.to_u64().unwrap()),
        }),
    );

    // check proxy balance
    proxy_dex_setup.b_mock.borrow().check_dct_balance(
        proxy_dex_setup.proxy_wrapper.address_ref(),
        LP_TOKEN_ID,
        &(expected_lp_token_amount.clone() + 499_999_000u64), // from other user's add initial liquidity
    );

    b_mock
        .borrow_mut()
        .execute_tx(
            &proxy_dex_setup.owner,
            &proxy_dex_setup.farm_locked_wrapper,
            &rust_biguint!(0),
            |sc| {
                sc.farming_token_id().set(&managed_token_id!(LP_TOKEN_ID));

                // set produce rewards to false for easier calculation
                sc.produce_rewards_enabled().set(false);
            },
        )
        .assert_ok();

    b_mock.borrow_mut().set_dct_local_roles(
        proxy_dex_setup.farm_locked_wrapper.address_ref(),
        LP_TOKEN_ID,
        &[DctLocalRole::Burn],
    );

    b_mock
        .borrow_mut()
        .execute_dct_transfer(
            &proxy_dex_setup.first_user,
            &pos_creator_wrapper,
            WMOAX_TOKEN_ID,
            0,
            &rust_biguint!(1_000),
            |sc| {
                let create_farm_pos_result = sc.create_farm_pos_from_single_token(
                    managed_biguint!(1u32),
                    LOCK_OPTIONS[0],
                    managed_biguint!(1u32),
                    managed_biguint!(1u32),
                );
                assert_eq!(create_farm_pos_result.locked_token_leftover.amount, 0u64);
                assert_eq!(create_farm_pos_result.wmoax_leftover.amount, 2u64);
                assert_eq!(create_farm_pos_result.wrapped_farm_token.amount, 497u64);
            },
        )
        .assert_ok();

    // check user balance
    b_mock.borrow().check_nft_balance(
        first_user,
        WRAPPED_FARM_TOKEN_ID,
        1,
        &expected_lp_token_amount,
        Some(&WrappedFarmTokenAttributes::<DebugApi> {
            proxy_farming_token: DctTokenPayment {
                token_identifier: managed_token_id!(WRAPPED_LP_TOKEN_ID),
                token_nonce: 3,
                amount: managed_biguint!(expected_lp_token_amount.to_u64().unwrap()),
            },
            farm_token: DctTokenPayment {
                token_identifier: managed_token_id!(FARM_LOCKED_TOKEN_ID),
                token_nonce: 1,
                amount: managed_biguint!(expected_lp_token_amount.to_u64().unwrap()),
            },
        }),
    );
}
