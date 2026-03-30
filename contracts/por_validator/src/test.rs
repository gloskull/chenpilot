#![cfg(test)]

use super::*;
use soroban_sdk::testutils::{Address as _};
use soroban_sdk::{token, Address, Env, contractimpl};

// --- Mock Oracle Implementation ---
pub struct MockOracle;

#[contract]
impl MockOracle {
    pub fn get_reserve_data(env: Env) -> ReserveData {
        env.storage().instance().get(&symbol_short!("res")).unwrap()
    }
    
    pub fn set_reserve_data(env: Env, data: ReserveData) {
        env.storage().instance().set(&symbol_short!("res"), &data);
    }
}

// --- Helper Functions ---
fn create_token<'a>(env: &Env, admin: &Address) -> (Address, token::Client<'a>, token::StellarAssetClient<'a>) {
    let contract_id = env.register_stellar_asset_contract(admin.clone());
    let token = token::Client::new(env, &contract_id);
    let stellar_asset_client = token::StellarAssetClient::new(env, &contract_id);
    (contract_id, token, stellar_asset_client)
}

#[test]
fn test_por_validation_success() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let (wbtc_addr, _token_client, stellar_asset) = create_token(&env, &admin);
    
    let oracle_id = env.register_contract(None, MockOracle);
    let oracle_client = MockOracleClient::new(&env, &oracle_id);
    
    // Set 1,000,000 units in oracle
    oracle_client.set_reserve_data(&ReserveData { balance: 1_000_000, timestamp: 12345 });
    
    // Mint 1,000,000 Wrapped BTC (Perfect balance)
    stellar_asset.mint(&admin, &1_000_000);
    
    let contract_id = env.register_contract(None, PoRValidatorContract);
    let client = PoRValidatorContractClient::new(&env, &contract_id);
    
    // 50 bps tolerance (0.5%)
    client.initialize(&admin, &wbtc_addr, &oracle_id, &50);
    
    client.verify_reserves();
    assert_eq!(client.is_valid(), true);
}

#[test]
fn test_por_validation_alert_on_discrepancy() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let (wbtc_addr, _token_client, stellar_asset) = create_token(&env, &admin);
    
    let oracle_id = env.register_contract(None, MockOracle);
    let oracle_client = MockOracleClient::new(&env, &oracle_id);
    
    // Set 1,000,000 units in oracle
    oracle_client.set_reserve_data(&ReserveData { balance: 1_000_000, timestamp: 12345 });
    
    // Mint 1,100,000 Wrapped BTC (Supply > Reserves + 0.5% tolerance)
    // 1,100,000 > 1,005,000 (allowed)
    stellar_asset.mint(&admin, &1_100_000);
    
    let contract_id = env.register_contract(None, PoRValidatorContract);
    let client = PoRValidatorContractClient::new(&env, &contract_id);
    
    client.initialize(&admin, &wbtc_addr, &oracle_id, &50);
    
    client.verify_reserves();
    assert_eq!(client.is_valid(), false);
}

#[test]
fn test_por_validation_within_tolerance() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let (wbtc_addr, _token_client, stellar_asset) = create_token(&env, &admin);
    
    let oracle_id = env.register_contract(None, MockOracle);
    let oracle_client = MockOracleClient::new(&env, &oracle_id);
    
    // Set 1,000,000 units in oracle
    oracle_client.set_reserve_data(&ReserveData { balance: 1_000_000, timestamp: 12345 });
    
    // Mint 1,004,000 Wrapped BTC (0.4% deviation, within 0.5% tolerance)
    stellar_asset.mint(&admin, &1_004_000);
    
    let contract_id = env.register_contract(None, PoRValidatorContract);
    let client = PoRValidatorContractClient::new(&env, &contract_id);
    
    client.initialize(&admin, &wbtc_addr, &oracle_id, &50);
    
    client.verify_reserves();
    assert_eq!(client.is_valid(), true);
}

#[test]
#[should_panic(expected = "Already initialized")]
fn test_initialization_failure() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, PoRValidatorContract);
    let client = PoRValidatorContractClient::new(&env, &contract_id);

    client.initialize(&admin, &admin, &admin, &50);
    client.initialize(&admin, &admin, &admin, &50);
}

#[test]
fn test_admin_config_update() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let new_oracle = Address::generate(&env);
    let contract_id = env.register_contract(None, PoRValidatorContract);
    let client = PoRValidatorContractClient::new(&env, &contract_id);

    client.initialize(&admin, &admin, &admin, &50);
    
    let mut config = client.get_config();
    config.oracle = new_oracle.clone();
    
    client.update_config(&config);
    assert_eq!(client.get_config().oracle, new_oracle);
}
