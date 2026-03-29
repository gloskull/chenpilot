#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Events},
    symbol_short, vec, Env, Address, BytesN, IntoVal, FromVal,
};

fn setup(env: &Env) -> (Address, CoreVaultContractClient<'_>) {
    let admin = Address::generate(env);
    let contract_id = env.register(CoreVaultContract, ());
    let client = CoreVaultContractClient::new(env, &contract_id);
    client.init(&admin);
    (admin, client)
}

fn dummy_hash(env: &Env) -> BytesN<32> {
    BytesN::from_array(env, &[1u8; 32])
}

fn topic_vec(env: &Env, sym: soroban_sdk::Symbol, contract_id: &Address) -> soroban_sdk::Vec<soroban_sdk::Val> {
    vec![&env, sym.into_val(env), contract_id.into_val(env)]
}

#[test]
fn test_init_emits_event() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let contract_id = env.register(CoreVaultContract, ());
    let client = CoreVaultContractClient::new(&env, &contract_id);
    client.init(&admin);

    let events = env.events().all();
    let last = events.last().unwrap();
    assert_eq!(last.1, topic_vec(&env, symbol_short!("init"), &contract_id));

    let data = EvtInit::from_val(&env, &last.2);
    assert_eq!(data.admin, admin);
}

#[test]
fn test_propose_upgrade_emits_event() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, client) = setup(&env);
    let contract_id = client.address.clone();
    let hash = dummy_hash(&env);

    client.propose_upgrade(&hash);

    let events = env.events().all();
    let last = events.last().unwrap();
    assert_eq!(last.1, topic_vec(&env, symbol_short!("upg_prop"), &contract_id));

    let data = EvtUpgradeProposed::from_val(&env, &last.2);
    assert_eq!(data.admin, admin);
    assert_eq!(data.new_wasm_hash, hash);
    assert_eq!(data.unlock_ledger, env.ledger().sequence() + TIMELOCK_LEDGERS);
}

#[test]
fn test_cancel_upgrade_emits_event() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, client) = setup(&env);
    let contract_id = client.address.clone();

    client.propose_upgrade(&dummy_hash(&env));
    client.cancel_upgrade();

    let events = env.events().all();
    let last = events.last().unwrap();
    assert_eq!(last.1, topic_vec(&env, symbol_short!("upg_cncl"), &contract_id));

    let data = EvtUpgradeCancelled::from_val(&env, &last.2);
    assert_eq!(data.admin, admin);
}

#[test]
fn test_transfer_admin_emits_event() {
    let env = Env::default();
    env.mock_all_auths();
    let (old_admin, client) = setup(&env);
    let contract_id = client.address.clone();
    let new_admin = Address::generate(&env);

    client.transfer_admin(&new_admin);

    let events = env.events().all();
    let last = events.last().unwrap();
    assert_eq!(last.1, topic_vec(&env, symbol_short!("adm_xfer"), &contract_id));

    let data = EvtAdminTransferred::from_val(&env, &last.2);
    assert_eq!(data.old_admin, old_admin);
    assert_eq!(data.new_admin, new_admin);
}

#[test]
#[should_panic(expected = "time-lock not expired")]
fn test_apply_upgrade_before_timelock() {
    let env = Env::default();
    env.mock_all_auths();
    let (_admin, client) = setup(&env);
    client.propose_upgrade(&dummy_hash(&env));
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
#[should_panic(expected = "already initialized")]
fn test_double_init() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, client) = setup(&env);
    client.init(&admin);
}
