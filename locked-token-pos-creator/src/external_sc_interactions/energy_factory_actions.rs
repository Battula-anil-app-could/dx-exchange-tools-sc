use common_structs::Epoch;
use energy_factory::virtual_lock::ProxyTrait as _;

dharitri_sc::imports!();

#[dharitri_sc::module]
pub trait EnergyFactoryActionsModule: energy_query::EnergyQueryModule {
    fn call_lock_virtual(
        &self,
        payment: DctTokenPayment,
        lock_epochs: Epoch,
        user: ManagedAddress,
    ) -> DctTokenPayment {
        let energy_factory_address = self.energy_factory_address().get();
        let own_address = self.blockchain().get_sc_address();
        let locked_tokens: DctTokenPayment = self
            .energy_factory_proxy(energy_factory_address)
            .lock_virtual(
                payment.token_identifier.clone(),
                payment.amount.clone(),
                lock_epochs,
                own_address,
                user,
            )
            .execute_on_dest_context();

        self.send()
            .dct_local_burn(&payment.token_identifier, 0, &payment.amount);

        locked_tokens
    }
}
