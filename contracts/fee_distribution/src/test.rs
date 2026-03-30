#![cfg(test)]

use super::*;
use soroban_sdk::testutils::{Address as _};
use soroban_sdk::{token, Address, Env};

fn create_token<'a>(env: &Env, admin: &Address) -> (Address, token::Client<'a>, token::StellarAssetClient<'a>) {
    let contract_id = env.register_stellar_asset_contract(admin.clone());
    let token = token::Client::new(env, &contract_id);
    let stellar_asset_client = token::StellarAssetClient::new(env, &contract_id);
    (contract_id, token, stellar_asset_client)
}

#[test]
fn test_initialization() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let treasury = Address::generate(&env);
    let ai_agent_pool = Address::generate(&env);
    let lp_pool = Address::generate(&env);

    let contract_id = env.register_contract(None, FeeDistributionContract);
    let client = FeeDistributionContractClient::new(&env, &contract_id);

    client.initialize(&admin, &treasury, &ai_agent_pool, &lp_pool, &3000, &2000);

    let config = client.get_config();
    assert_eq!(config.admin, admin);
    assert_eq!(config.treasury, treasury);
    assert_eq!(config.ai_agent_pool, ai_agent_pool);
    assert_eq!(config.lp_pool, lp_pool);
    assert_eq!(config.treasury_bps, 3000);
    assert_eq!(config.ai_agent_bps, 2000);
}

#[test]
fn test_distribute_full_precision() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let treasury = Address::generate(&env);
    let ai_agent_pool = Address::generate(&env);
    let lp_pool = Address::generate(&env);
    let fee_source = Address::generate(&env);

    let (token_addr, token_client, stellar_asset) = create_token(&env, &admin);

    let contract_id = env.register_contract(None, FeeDistributionContract);
    let client = FeeDistributionContractClient::new(&env, &contract_id);

    // 30% Treasury, 20% AI, remainder (50%) LP
    client.initialize(&admin, &treasury, &ai_agent_pool, &lp_pool, &3000, &2000);

    let amount = 10000;
    stellar_asset.mint(&fee_source, &amount);

    client.distribute(&token_addr, &fee_source, &amount);

    assert_eq!(token_client.balance(&treasury), 3000);
    assert_eq!(token_client.balance(&ai_agent_pool), 2000);
    assert_eq!(token_client.balance(&lp_pool), 5000);
}

#[test]
fn test_rounding_away_from_zero() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let treasury = Address::generate(&env);
    let ai_agent_pool = Address::generate(&env);
    let lp_pool = Address::generate(&env);
    let fee_source = Address::generate(&env);

    let (token_addr, token_client, stellar_asset) = create_token(&env, &admin);

    let contract_id = env.register_contract(None, FeeDistributionContract);
    let client = FeeDistributionContractClient::new(&env, &contract_id);

    client.initialize(&admin, &treasury, &ai_agent_pool, &lp_pool, &3333, &3333);

    // 10 units distributed.
    // Treasury: 10 * 3333 / 10000 = 3.333 -> 3
    // AI: 10 * 3333 / 10000 = 3.333 -> 3
    // LP: 10 - 3 - 3 = 4 (gets the rounding remainder)
    let amount = 10;
    stellar_asset.mint(&fee_source, &amount);

    client.distribute(&token_addr, &fee_source, &amount);

    assert_eq!(token_client.balance(&treasury), 3);
    assert_eq!(token_client.balance(&ai_agent_pool), 3);
    assert_eq!(token_client.balance(&lp_pool), 4);
}

#[test]
fn test_low_decimal_dust_recovery() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let treasury = Address::generate(&env);
    let ai_agent_pool = Address::generate(&env);
    let lp_pool = Address::generate(&env);
    let fee_source = Address::generate(&env);

    let (token_addr, token_client, stellar_asset) = create_token(&env, &admin);

    let contract_id = env.register_contract(None, FeeDistributionContract);
    let client = FeeDistributionContractClient::new(&env, &contract_id);

    client.initialize(&admin, &treasury, &ai_agent_pool, &lp_pool, &2500, &2500);

    // Distribute 3 units.
    // Treasury: 3 * 2500 / 10000 = 0.75 -> 0
    // AI: 3 * 2500 / 10000 = 0.75 -> 0
    // LP: 3 - 0 - 0 = 3
    let amount = 3;
    stellar_asset.mint(&fee_source, &amount);

    client.distribute(&token_addr, &fee_source, &amount);

    assert_eq!(token_client.balance(&treasury), 0);
    assert_eq!(token_client.balance(&ai_agent_pool), 0);
    assert_eq!(token_client.balance(&lp_pool), 3);
}

#[test]
#[should_panic(expected = "Already initialized")]
fn test_double_initialization_fails() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, FeeDistributionContract);
    let client = FeeDistributionContractClient::new(&env, &contract_id);

    client.initialize(&admin, &admin, &admin, &admin, &0, &0);
    client.initialize(&admin, &admin, &admin, &admin, &0, &0);
}

#[test]
fn test_update_config_authorized() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let new_admin = Address::generate(&env);
    let treasury = Address::generate(&env);
    let contract_id = env.register_contract(None, FeeDistributionContract);
    let client = FeeDistributionContractClient::new(&env, &contract_id);

    client.initialize(&admin, &treasury, &treasury, &treasury, &1000, &1000);

    let mut config = client.get_config();
    config.admin = new_admin.clone();
    
    client.update_config(&config);
    assert_eq!(client.get_config().admin, new_admin);
}

#[test]
#[should_panic]
fn test_invalid_bps_initialization() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, FeeDistributionContract);
    let client = FeeDistributionContractClient::new(&env, &contract_id);

    client.initialize(&admin, &admin, &admin, &admin, &6000, &5000); // 11000 bps > 10000
}
