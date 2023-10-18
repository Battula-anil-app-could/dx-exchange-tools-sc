use common_structs::Epoch;

use crate::external_sc_interactions::proxy_dex_actions::AddLiquidityProxyResult;

dharitri_sc::imports!();

pub struct AddLiquidityArguments<M: ManagedTypeApi> {
    pub payment: DctTokenPayment<M>,
    pub swap_min_amount_out: BigUint<M>,
    pub lock_epochs: Epoch,
    pub add_liq_first_token_min_amount: BigUint<M>,
    pub add_liq_second_token_min_amount: BigUint<M>,
}

#[dharitri_sc::module]
pub trait CreatePairPosModule:
    crate::external_sc_interactions::moax_wrapper_actions::MoaxWrapperActionsModule
    + crate::external_sc_interactions::energy_factory_actions::EnergyFactoryActionsModule
    + crate::external_sc_interactions::pair_actions::PairActionsModule
    + crate::external_sc_interactions::proxy_dex_actions::ProxyDexActionsModule
    + energy_query::EnergyQueryModule
    + utils::UtilsModule
{
    /// lock_epochs must be one of the values allowed by energy_factory
    #[payable("*")]
    #[endpoint(createPairPosFromSingleToken)]
    fn create_pair_pos_from_single_token_endpoint(
        &self,
        swap_min_amount_out: BigUint,
        lock_epochs: Epoch,
        add_liq_first_token_min_amount: BigUint,
        add_liq_second_token_min_amount: BigUint,
    ) -> AddLiquidityProxyResult<Self::Api> {
        let payment = self.call_value().moax_or_single_dct();
        let payment_dct = self.get_dct_payment(payment);
        let args = AddLiquidityArguments {
            payment: payment_dct,
            swap_min_amount_out,
            lock_epochs,
            add_liq_first_token_min_amount,
            add_liq_second_token_min_amount,
        };

        let add_liq_result = self.create_pair_pos_from_single_token(args);

        let mut output_payments =
            ManagedVec::from_single_item(add_liq_result.wrapped_lp_token.clone());
        if add_liq_result.locked_token_leftover.amount > 0 {
            output_payments.push(add_liq_result.locked_token_leftover.clone());
        }
        if add_liq_result.wmoax_leftover.amount > 0 {
            output_payments.push(add_liq_result.wmoax_leftover.clone());
        }

        let caller = self.blockchain().get_caller();
        self.send().direct_multi(&caller, &output_payments);

        add_liq_result
    }

    fn get_dct_payment(&self, payment: MoaxOrDctTokenPayment) -> DctTokenPayment {
        let wmoax_token_id = self.wmoax_token_id().get();
        if payment.token_identifier.is_moax() {
            self.call_wrap_moax(payment.amount)
        } else if payment.token_identifier == MoaxOrDctTokenIdentifier::dct(wmoax_token_id) {
            payment.unwrap_dct()
        } else {
            sc_panic!("Invalid payment");
        }
    }

    fn create_pair_pos_from_single_token(
        &self,
        args: AddLiquidityArguments<Self::Api>,
    ) -> AddLiquidityProxyResult<Self::Api> {
        let half_wmoax_payment = DctTokenPayment::new(
            args.payment.token_identifier.clone(),
            0,
            args.payment.amount.clone() / 2u32,
        );
        let remaining_wmoax = DctTokenPayment::new(
            args.payment.token_identifier.clone(),
            0,
            &args.payment.amount - &half_wmoax_payment.amount,
        );

        let mex_token_id = self.get_base_token_id();
        let mex_tokens =
            self.call_pair_swap(half_wmoax_payment, mex_token_id, args.swap_min_amount_out);

        let caller = self.blockchain().get_caller();
        let locked_tokens = self.call_lock_virtual(mex_tokens, args.lock_epochs, caller);

        let mut proxy_payments = ManagedVec::new();
        proxy_payments.push(remaining_wmoax);
        proxy_payments.push(locked_tokens);

        let pair_address = self.mex_wmoax_pair_address().get();
        self.call_add_liquidity_proxy(
            proxy_payments,
            pair_address,
            args.add_liq_first_token_min_amount,
            args.add_liq_second_token_min_amount,
        )
    }
}