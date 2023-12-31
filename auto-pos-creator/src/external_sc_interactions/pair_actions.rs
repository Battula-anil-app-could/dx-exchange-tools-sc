use pair::{AddLiquidityResultType, RemoveLiquidityResultType};

dharitri_sc::imports!();

pub const MIN_AMOUNT_OUT: u32 = 1;

pub struct PairAddLiqResult<M: ManagedTypeApi> {
    pub lp_tokens: DctTokenPayment<M>,
    pub first_tokens_remaining: DctTokenPayment<M>,
    pub second_tokens_remaining: DctTokenPayment<M>,
}

pub struct PairRemoveLiqResult<M: ManagedTypeApi> {
    pub first_tokens: DctTokenPayment<M>,
    pub second_tokens: DctTokenPayment<M>,
}

pub type PairTokenPayments<M> = PairRemoveLiqResult<M>;

#[dharitri_sc::module]
pub trait PairActionsModule:
    crate::configs::pairs_config::PairsConfigModule + utils::UtilsModule
{
    fn perform_tokens_swap(
        &self,
        from_tokens: TokenIdentifier,
        from_amount: BigUint,
        to_tokens: TokenIdentifier,
    ) -> DctTokenPayment {
        if from_tokens == to_tokens {
            return DctTokenPayment::new(from_tokens, 0, from_amount);
        }

        let pair_address = self
            .get_pair_address_for_tokens(&from_tokens, &to_tokens)
            .unwrap_address();
        let payment = DctTokenPayment::new(from_tokens, 0, from_amount);

        self.call_pair_swap(pair_address, payment, to_tokens)
    }

    fn call_pair_swap(
        &self,
        pair_address: ManagedAddress,
        input_tokens: DctTokenPayment,
        requested_token_id: TokenIdentifier,
    ) -> DctTokenPayment {
        self.pair_proxy(pair_address)
            .swap_tokens_fixed_input(requested_token_id, MIN_AMOUNT_OUT)
            .with_dct_transfer(input_tokens)
            .execute_on_dest_context()
    }

    fn call_pair_add_liquidity(
        &self,
        pair_address: ManagedAddress,
        first_tokens: DctTokenPayment,
        second_tokens: DctTokenPayment,
    ) -> PairAddLiqResult<Self::Api> {
        let first_token_full_amount = first_tokens.amount.clone();
        let second_token_full_amount = second_tokens.amount.clone();
        let raw_results: AddLiquidityResultType<Self::Api> = self
            .pair_proxy(pair_address)
            .add_liquidity(MIN_AMOUNT_OUT, MIN_AMOUNT_OUT)
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

    fn call_pair_remove_liquidity(
        &self,
        pair_address: ManagedAddress,
        lp_tokens: DctTokenPayment,
    ) -> PairRemoveLiqResult<Self::Api> {
        let raw_results: RemoveLiquidityResultType<Self::Api> = self
            .pair_proxy(pair_address)
            .remove_liquidity(MIN_AMOUNT_OUT, MIN_AMOUNT_OUT)
            .with_dct_transfer(lp_tokens)
            .execute_on_dest_context();
        let (first_tokens, second_tokens) = raw_results.into_tuple();

        PairRemoveLiqResult {
            first_tokens,
            second_tokens,
        }
    }

    #[proxy]
    fn pair_proxy(&self, sc_address: ManagedAddress) -> pair::Proxy<Self::Api>;
}
