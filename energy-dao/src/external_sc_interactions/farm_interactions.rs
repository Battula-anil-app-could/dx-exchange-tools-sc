dharitri_sc::imports!();

use crate::{
    common::errors::{
        ERROR_BAD_PAYMENT_TOKENS, ERROR_EXTERNAL_CONTRACT_OUTPUT, ERROR_FARM_DOES_NOT_EXIST,
        ERROR_UNBOND_TOO_SOON,
    },
    external_sc_interactions::farm_config::{UnstakeTokenAttributes, WrappedFarmTokenAttributes},
};
use common_structs::PaymentsVec;
use locked_token_wrapper::wrapped_token;

#[dharitri_sc::module]
pub trait FarmInteractionsModule:
    crate::external_sc_interactions::farm_config::FarmConfigModule
    + crate::external_sc_interactions::farm_actions::FarmActionsModule
    + crate::external_sc_interactions::locked_token_actions::LockedTokenModule
    + crate::external_sc_interactions::fees_collector_interactions::FeesCollectorInteractionsModule
    + lkmex_transfer::energy_transfer::EnergyTransferModule
    + legacy_token_decode_module::LegacyTokenDecodeModule
    + energy_query::EnergyQueryModule
    + token_send::TokenSendModule
    + utils::UtilsModule
    + wrapped_token::WrappedTokenModule
    + simple_lock::token_attributes::TokenAttributesModule
    + dharitri_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    #[payable("*")]
    #[endpoint(enterFarm)]
    fn enter_farm_endpoint(&self, farm_address: ManagedAddress) -> DctTokenPayment {
        let payment = self.call_value().single_dct();
        let mut farm_state_mapper = self.farm_state(&farm_address);
        require!(!farm_state_mapper.is_empty(), ERROR_FARM_DOES_NOT_EXIST);
        let farming_token_id = self.get_farming_token(&farm_address);
        require!(
            farming_token_id == payment.token_identifier,
            ERROR_BAD_PAYMENT_TOKENS
        );

        let farm_state = farm_state_mapper.get();
        let farm_token_id = self.get_farm_token(&farm_address);
        let division_safety_constant = self.get_division_safety_constant(&farm_address);
        let mut enter_farm_payments = ManagedVec::from_single_item(payment);

        let current_farm_position = DctTokenPayment::new(
            farm_token_id,
            farm_state.farm_token_nonce,
            farm_state.farm_staked_value.clone(),
        );
        let initial_total_farm_amount = current_farm_position.amount.clone();
        if initial_total_farm_amount > 0 {
            enter_farm_payments.push(current_farm_position);
        }

        let enter_farm_result = self.call_enter_farm(farm_address, enter_farm_payments);
        let (new_farm_token, farm_rewards) = enter_farm_result.into_tuple();

        require!(
            new_farm_token.amount > initial_total_farm_amount,
            ERROR_EXTERNAL_CONTRACT_OUTPUT
        );

        let user_farm_amount = &new_farm_token.amount - &initial_total_farm_amount;
        self.update_farm_after_claim(
            &farm_state,
            &mut farm_state_mapper,
            new_farm_token,
            farm_rewards,
            &division_safety_constant,
        );

        let caller = self.blockchain().get_caller();
        let new_farm_state = farm_state_mapper.get();
        let user_token_attributes = WrappedFarmTokenAttributes {
            token_rps: new_farm_state.farm_rps,
        };
        let wrapped_farm_token_id = self.wrapped_farm_token_id().get();
        let output_payment = self.mint_tokens(
            wrapped_farm_token_id,
            user_farm_amount,
            &user_token_attributes,
        );
        self.send_payment_non_zero(&caller, &output_payment);

        output_payment
    }

    #[endpoint(claimFarmRewards)]
    fn claim_farm_rewards(&self, farm_address: ManagedAddress) {
        let mut farm_state_mapper = self.farm_state(&farm_address);
        require!(!farm_state_mapper.is_empty(), ERROR_FARM_DOES_NOT_EXIST);
        let farm_state = farm_state_mapper.get();
        let farm_token_id = self.get_farm_token(&farm_address);
        let division_safety_constant = self.get_division_safety_constant(&farm_address);
        let current_farm_position = DctTokenPayment::new(
            farm_token_id,
            farm_state.farm_token_nonce,
            farm_state.farm_staked_value.clone(),
        );

        let claim_rewards_result = self.call_farm_claim(farm_address, current_farm_position);
        let (new_farm_token, farm_rewards) = claim_rewards_result.into_tuple();

        self.update_farm_after_claim(
            &farm_state,
            &mut farm_state_mapper,
            new_farm_token,
            farm_rewards,
            &division_safety_constant,
        );
    }

    #[payable("*")]
    #[endpoint(claimUserRewards)]
    fn claim_user_rewards(&self, farm_address: ManagedAddress) {
        let payment = self.call_value().single_dct();
        let mut farm_state_mapper = self.farm_state(&farm_address);
        require!(!farm_state_mapper.is_empty(), ERROR_FARM_DOES_NOT_EXIST);
        let farm_state = farm_state_mapper.get();
        let division_safety_constant = self.get_division_safety_constant(&farm_address);
        let wrapped_farm_token_id = self.wrapped_farm_token_id().get();
        require!(
            wrapped_farm_token_id == payment.token_identifier,
            ERROR_BAD_PAYMENT_TOKENS
        );
        let user_rewards = self.compute_user_rewards_payment(
            &mut farm_state_mapper,
            &payment,
            &division_safety_constant,
        );
        if user_rewards.amount > 0 {
            farm_state_mapper.update(|config| config.reward_reserve -= &user_rewards.amount);
        }
        let new_attributes = WrappedFarmTokenAttributes {
            token_rps: farm_state.farm_rps,
        };

        self.send().dct_local_burn(
            &payment.token_identifier,
            payment.token_nonce,
            &payment.amount,
        );
        let wrapped_farm_token_id = self.wrapped_farm_token_id().get();
        let new_farm_token =
            self.mint_tokens(wrapped_farm_token_id, payment.amount, &new_attributes);

        let mut user_payments = ManagedVec::from_single_item(new_farm_token);
        if user_rewards.amount > 0 {
            let wrapper_user_rewards = self.wrap_locked_token(user_rewards);
            user_payments.push(wrapper_user_rewards);
        }
        let caller = self.blockchain().get_caller();
        self.send().direct_multi(&caller, &user_payments);
    }

    #[payable("*")]
    #[endpoint(unstakeFarm)]
    fn unstake_farm(&self, farm_address: ManagedAddress) -> DctTokenPayment {
        let payment = self.call_value().single_dct();
        let mut farm_state_mapper = self.farm_state(&farm_address);
        require!(!farm_state_mapper.is_empty(), ERROR_FARM_DOES_NOT_EXIST);
        let farm_state = farm_state_mapper.get();
        let farm_token_id = self.get_farm_token(&farm_address);
        let division_safety_constant = self.get_division_safety_constant(&farm_address);
        let wrapped_farm_token_id = self.wrapped_farm_token_id().get();
        require!(
            wrapped_farm_token_id == payment.token_identifier,
            ERROR_BAD_PAYMENT_TOKENS
        );

        let current_farm_position = DctTokenPayment::new(
            farm_token_id,
            farm_state.farm_token_nonce,
            farm_state.farm_staked_value.clone(),
        );
        let claim_rewards_result = self.call_farm_claim(farm_address, current_farm_position);
        let (new_farm_token, farm_rewards) = claim_rewards_result.into_tuple();
        self.update_farm_after_claim(
            &farm_state,
            &mut farm_state_mapper,
            new_farm_token.clone(),
            farm_rewards,
            &division_safety_constant,
        );

        let user_rewards = self.compute_user_rewards_payment(
            &mut farm_state_mapper,
            &payment,
            &division_safety_constant,
        );

        farm_state_mapper.update(|config| {
            config.farm_staked_value -= &payment.amount;
            config.farm_unstaked_value += &payment.amount;
            config.reward_reserve -= &user_rewards.amount;
        });
        self.send().dct_local_burn(
            &payment.token_identifier,
            payment.token_nonce,
            &payment.amount,
        );
        let current_epoch = self.blockchain().get_block_epoch();
        let unstake_attributes = UnstakeTokenAttributes {
            unstake_epoch: current_epoch,
            token_nonce: new_farm_token.token_nonce,
        };
        let unstake_farm_token_id = self.unstake_farm_token_id().get();
        let unstake_token_payment =
            self.mint_tokens(unstake_farm_token_id, payment.amount, &unstake_attributes);

        let caller = self.blockchain().get_caller();
        self.send_payment_non_zero(&caller, &unstake_token_payment);

        unstake_token_payment
    }

    #[payable("*")]
    #[endpoint(unbondFarm)]
    fn unbond_farm(&self, farm_address: ManagedAddress) -> PaymentsVec<Self::Api> {
        let payment = self.call_value().single_dct();
        let farm_state_mapper = self.farm_state(&farm_address);
        require!(!farm_state_mapper.is_empty(), ERROR_FARM_DOES_NOT_EXIST);
        let unstake_farm_token_id = self.unstake_farm_token_id().get();
        require!(
            unstake_farm_token_id == payment.token_identifier,
            ERROR_BAD_PAYMENT_TOKENS
        );
        let unstake_attributes: UnstakeTokenAttributes =
            self.get_token_attributes(&payment.token_identifier, payment.token_nonce);
        let current_epoch = self.blockchain().get_block_epoch();
        let unbond_period = self.unbond_period().get();
        let unbond_epoch = unstake_attributes.unstake_epoch + unbond_period;
        require!(current_epoch >= unbond_epoch, ERROR_UNBOND_TOO_SOON);

        let farm_token_id = self.get_farm_token(&farm_address);
        let unstake_payment = DctTokenPayment::new(
            farm_token_id,
            unstake_attributes.token_nonce,
            payment.amount.clone(),
        );
        let exit_farm_result = self.call_exit_farm(farm_address, unstake_payment);
        let (mut farming_tokens, locked_rewards_payment, _) = exit_farm_result.into_tuple();

        farm_state_mapper.update(|config| {
            config.farm_unstaked_value -= &payment.amount;
        });

        self.send().dct_local_burn(
            &payment.token_identifier,
            payment.token_nonce,
            &payment.amount,
        );
        self.apply_fee(&mut farming_tokens);
        let mut user_payments = ManagedVec::from_single_item(farming_tokens);
        if locked_rewards_payment.amount > 0 {
            user_payments.push(locked_rewards_payment);
        }
        let caller = self.blockchain().get_caller();
        self.send().direct_multi(&caller, &user_payments);

        user_payments
    }
}
