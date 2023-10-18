dharitri_sc::imports!();

mod moax_wrapper_proxy {
    dharitri_sc::imports!();

    #[dharitri_sc::proxy]
    pub trait MoaxWrapperProxy {
        #[payable("MOAX")]
        #[endpoint(wrapMoax)]
        fn wrap_moax(&self) -> DctTokenPayment;
    }
}

#[dharitri_sc::module]
pub trait MoaxWrapperActionsModule {
    fn call_wrap_moax(&self, moax_amount: BigUint) -> DctTokenPayment {
        let wrapper_sc_address = self.moax_wrapper_sc_address().get();
        self.moax_wrapper_proxy(wrapper_sc_address)
            .wrap_moax()
            .with_moax_transfer(moax_amount)
            .execute_on_dest_context()
    }

    #[storage_mapper("wmoaxTokenId")]
    fn wmoax_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[storage_mapper("moaxWrapperScAddress")]
    fn moax_wrapper_sc_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[proxy]
    fn moax_wrapper_proxy(
        &self,
        sc_address: ManagedAddress,
    ) -> moax_wrapper_proxy::Proxy<Self::Api>;
}
