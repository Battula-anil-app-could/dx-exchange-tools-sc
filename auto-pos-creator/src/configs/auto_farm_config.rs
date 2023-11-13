dharitri_sc::imports!();

#[dharitri_sc::module]
pub trait AutoFarmConfigModule {
    #[storage_mapper("autoFarmScAddress")]
    fn auto_farm_sc_address(&self) -> SingleValueMapper<ManagedAddress>;
}
