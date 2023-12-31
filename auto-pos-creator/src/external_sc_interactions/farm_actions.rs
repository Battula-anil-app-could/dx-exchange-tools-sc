use farm::{
    base_functions::{ExitFarmResultType, ExitFarmResultWrapper},
    EnterFarmResultType,
};

dharitri_sc::imports!();

// pub struct ExitFarmResultType<M: ManagedTypeApi>

#[dharitri_sc::module]
pub trait FarmActionsModule {
    fn call_enter_farm(
        &self,
        farm_address: ManagedAddress,
        user: ManagedAddress,
        farming_tokens: DctTokenPayment,
    ) -> DctTokenPayment {
        let raw_results: EnterFarmResultType<Self::Api> = self
            .farm_proxy(farm_address)
            .enter_farm_endpoint(user)
            .with_dct_transfer(farming_tokens)
            .execute_on_dest_context();

        // no rewards on simple enter
        let (new_farm_token, _) = raw_results.into_tuple();
        new_farm_token
    }

    fn call_exit_farm(
        &self,
        farm_address: ManagedAddress,
        user: ManagedAddress,
        farm_tokens: DctTokenPayment,
    ) -> ExitFarmResultWrapper<Self::Api> {
        let raw_results: ExitFarmResultType<Self::Api> = self
            .farm_proxy(farm_address)
            .exit_farm_endpoint(farm_tokens.amount.clone(), user)
            .with_dct_transfer(farm_tokens)
            .execute_on_dest_context();
        let (farming_tokens, rewards) = raw_results.into_tuple();

        ExitFarmResultWrapper {
            farming_tokens,
            rewards,
        }
    }

    #[proxy]
    fn farm_proxy(&self, sc_address: ManagedAddress) -> farm_with_locked_rewards::Proxy<Self::Api>;
}
