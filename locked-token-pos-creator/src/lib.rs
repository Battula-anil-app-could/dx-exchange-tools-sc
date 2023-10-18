#![no_std]

dharitri_sc::imports!();

pub mod create_farm_pos;
pub mod create_pair_pos;
pub mod external_sc_interactions;

#[dharitri_sc::contract]
pub trait LockedTokenPosCreatorContract:
    create_pair_pos::CreatePairPosModule
    + create_farm_pos::CreateFarmPosModule
    + external_sc_interactions::moax_wrapper_actions::MoaxWrapperActionsModule
    + external_sc_interactions::energy_factory_actions::EnergyFactoryActionsModule
    + external_sc_interactions::pair_actions::PairActionsModule
    + external_sc_interactions::proxy_dex_actions::ProxyDexActionsModule
    + energy_query::EnergyQueryModule
    + utils::UtilsModule
{
    /// This contract needs the burn role for MEX token
    #[init]
    fn init(
        &self,
        energy_factory_adddress: ManagedAddress,
        moax_wrapper_address: ManagedAddress,
        wmoax_token_id: TokenIdentifier,
        mex_wmoax_pair_address: ManagedAddress,
        mex_wmoax_lp_farm_address: ManagedAddress,
        proxy_dex_address: ManagedAddress,
    ) {
        self.require_sc_address(&moax_wrapper_address);
        self.require_valid_token_id(&wmoax_token_id);
        self.require_sc_address(&mex_wmoax_pair_address);
        self.require_sc_address(&mex_wmoax_lp_farm_address);
        self.require_sc_address(&proxy_dex_address);

        self.moax_wrapper_sc_address().set(moax_wrapper_address);
        self.wmoax_token_id().set(wmoax_token_id);
        self.mex_wmoax_pair_address().set(mex_wmoax_pair_address);
        self.farm_address().set(mex_wmoax_lp_farm_address);
        self.proxy_dex_address().set(proxy_dex_address);

        self.set_energy_factory_address(energy_factory_adddress);
    }

    #[endpoint]
    fn upgrade(&self) {}
}