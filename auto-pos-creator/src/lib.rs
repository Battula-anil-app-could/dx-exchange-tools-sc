#![no_std]

dharitri_sc::imports!();

pub mod common;
pub mod configs;
pub mod external_sc_interactions;
pub mod multi_contract_interactions;

#[dharitri_sc::contract]
pub trait AutoPosCreator:
    auto_farm::whitelists::farms_whitelist::FarmsWhitelistModule
    + auto_farm::whitelists::metastaking_whitelist::MetastakingWhitelistModule
    + auto_farm::external_storage_read::farm_storage_read::FarmStorageReadModule
    + auto_farm::external_storage_read::metastaking_storage_read::MetastakingStorageReadModule
    + utils::UtilsModule
    + configs::pairs_config::PairsConfigModule
    + external_sc_interactions::pair_actions::PairActionsModule
    + external_sc_interactions::farm_actions::FarmActionsModule
    + external_sc_interactions::metastaking_actions::MetastakingActionsModule
    + external_sc_interactions::moax_wrapper_actions::MoaxWrapperActionsModule
    + multi_contract_interactions::create_pos::CreatePosModule
    + multi_contract_interactions::create_pos_endpoints::CreatePosEndpointsModule
    + multi_contract_interactions::exit_pos::ExitPosModule
    + multi_contract_interactions::exit_pos_endpoints::ExitPosEndpointsModule
{
    #[init]
    fn init(&self, moax_wrapper_address: ManagedAddress, wmoax_token_id: TokenIdentifier) {
        self.require_sc_address(&moax_wrapper_address);
        self.require_valid_token_id(&wmoax_token_id);

        self.moax_wrapper_sc_address().set(moax_wrapper_address);
        self.wmoax_token_id().set(wmoax_token_id);
    }

    #[endpoint]
    fn upgrade(&self) {}
}
