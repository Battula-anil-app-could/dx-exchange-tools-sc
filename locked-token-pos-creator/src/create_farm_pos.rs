use common_structs::Epoch;

use crate::create_pair_pos::AddLiquidityArguments;

dharitri_sc::imports!();
dharitri_sc::derive_imports!();

#[derive(TypeAbi, TopEncode, TopDecode)]
pub struct CreateFarmPosResult<M: ManagedTypeApi> {
    pub wrapped_farm_token: DctTokenPayment<M>,
    pub locked_token_leftover: DctTokenPayment<M>,
    pub wmoax_leftover: DctTokenPayment<M>,
}

#[dharitri_sc::module]
pub trait CreateFarmPosModule:
    crate::external_sc_interactions::moax_wrapper_actions::MoaxWrapperActionsModule
    + crate::external_sc_interactions::energy_factory_actions::EnergyFactoryActionsModule
    + crate::external_sc_interactions::pair_actions::PairActionsModule
    + crate::external_sc_interactions::proxy_dex_actions::ProxyDexActionsModule
    + crate::create_pair_pos::CreatePairPosModule
    + energy_query::EnergyQueryModule
    + utils::UtilsModule
{
    #[payable("*")]
    #[endpoint(createFarmPosFromSingleToken)]
    fn create_farm_pos_from_single_token(
        &self,
        swap_min_amount_out: BigUint,
        lock_epochs: Epoch,
        add_liq_first_token_min_amount: BigUint,
        add_liq_second_token_min_amount: BigUint,
    ) -> CreateFarmPosResult<Self::Api> {
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

        let mut output_payments = ManagedVec::new();
        if add_liq_result.locked_token_leftover.amount > 0 {
            output_payments.push(add_liq_result.locked_token_leftover.clone());
        }
        if add_liq_result.wmoax_leftover.amount > 0 {
            output_payments.push(add_liq_result.wmoax_leftover.clone());
        }

        let farm_address = self.farm_address().get();
        let wrapped_farm_token =
            self.call_enter_farm_proxy(add_liq_result.wrapped_lp_token, farm_address);

        output_payments.push(wrapped_farm_token.clone());

        let caller = self.blockchain().get_caller();
        self.send().direct_multi(&caller, &output_payments);

        CreateFarmPosResult {
            wmoax_leftover: add_liq_result.wmoax_leftover,
            locked_token_leftover: add_liq_result.locked_token_leftover,
            wrapped_farm_token,
        }
    }

    #[storage_mapper("wmoaxMexLpFarmAddress")]
    fn farm_address(&self) -> SingleValueMapper<ManagedAddress>;
}