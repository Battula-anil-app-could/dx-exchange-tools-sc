use pair::AddLiquidityResultType;

dharitri_sc::imports!();

pub struct PairAddLiqResult<M: ManagedTypeApi> {
    pub lp_tokens: DctTokenPayment<M>,
    pub first_tokens_remaining: DctTokenPayment<M>,
    pub second_tokens_remaining: DctTokenPayment<M>,
}

#[dharitri_sc::module]
pub trait PairActionsModule {
    fn call_pair_swap(
        &self,
        input_tokens: DctTokenPayment,
        requested_token_id: TokenIdentifier,
        min_amount_out: BigUint,
    ) -> DctTokenPayment {
        let pair_address = self.mex_wmoax_pair_address().get();
        self.pair_proxy(pair_address)
            .swap_tokens_fixed_input(requested_token_id, min_amount_out)
            .with_dct_transfer(input_tokens)
            .execute_on_dest_context()
    }

    fn call_pair_add_liquidity(
        &self,
        pair_address: ManagedAddress,
        first_tokens: DctTokenPayment,
        second_tokens: DctTokenPayment,
        first_token_min_amount_out: BigUint,
        second_token_min_amount_out: BigUint,
    ) -> PairAddLiqResult<Self::Api> {
        let first_token_full_amount = first_tokens.amount.clone();
        let second_token_full_amount = second_tokens.amount.clone();
        let raw_results: AddLiquidityResultType<Self::Api> = self
            .pair_proxy(pair_address)
            .add_liquidity(first_token_min_amount_out, second_token_min_amount_out)
            .with_dct_transfer(first_tokens)
            .with_dct_transfer(second_tokens)
            .execute_on_dest_context();

        let (lp_tokens, first_tokens_used, second_tokens_used) = raw_results.into_tuple();
        let first_tokens_remaining_amount = first_token_full_amount - first_tokens_used.amount;
        let second_tokens_remaining_amount = second_token_full_amount - second_tokens_used.amount;

        let first_tokens_remaining = DctTokenPayment::new(
            first_tokens_used.token_identifier,
            0,
            first_tokens_remaining_amount,
        );
        let second_tokens_remaining = DctTokenPayment::new(
            second_tokens_used.token_identifier,
            0,
            second_tokens_remaining_amount,
        );

        PairAddLiqResult {
            lp_tokens,
            first_tokens_remaining,
            second_tokens_remaining,
        }
    }

    #[storage_mapper("mexWmoaxPairAddress")]
    fn mex_wmoax_pair_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[proxy]
    fn pair_proxy(&self, sc_address: ManagedAddress) -> pair::Proxy<Self::Api>;
}
