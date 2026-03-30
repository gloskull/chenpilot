#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Env, Address, BytesN};

fn setup(env: &Env) -> (Address, CoreVaultContractClient) {
    let admin = Address::generate(env);
    let contract_id = env.register_contract(None, CoreVaultContract);
    let client = CoreVaultContractClient::new(env, &contract_id);
    client.init(&admin);
    (admin, client)
}

fn dummy_hash(env: &Env) -> BytesN<32> {
    BytesN::from_array(env, &[1u8; 32])
}

#[test]
fn test_propose_and_cancel_upgrade() {
    let env = Env::default();
    env.mock_all_auths();
    let (_admin, client) = setup(&env);

    client.propose_upgrade(&dummy_hash(&env));
    assert!(client.upgrade_unlock_ledger() > 0);

    client.cancel_upgrade();
    assert_eq!(client.upgrade_unlock_ledger(), 0);
}

#[test]
#[should_panic(expected = "time-lock not expired")]
fn test_apply_upgrade_before_timelock() {
    let env = Env::default();
    env.mock_all_auths();
    let (_admin, client) = setup(&env);

    client.propose_upgrade(&dummy_hash(&env));
    // ledger sequence hasn't advanced past unlock_ledger
    client.apply_upgrade();
}

#[test]
#[should_panic(expected = "no pending upgrade")]
fn test_apply_upgrade_without_proposal() {
    let env = Env::default();
    env.mock_all_auths();
    let (_admin, client) = setup(&env);
    client.apply_upgrade();
}

#[test]
fn test_transfer_admin() {
    let env = Env::default();
    env.mock_all_auths();
    let (_admin, client) = setup(&env);

    let new_admin = Address::generate(&env);
    client.transfer_admin(&new_admin);

    // new admin can propose an upgrade without panicking
    client.propose_upgrade(&dummy_hash(&env));
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_double_init() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, client) = setup(&env);
    client.init(&admin);
}
