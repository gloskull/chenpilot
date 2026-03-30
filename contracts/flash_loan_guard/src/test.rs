#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    contract, contractimpl, Address, Env,
};

// ---------------------------------------------------------------------------
// Mock oracle
// ---------------------------------------------------------------------------
#[contract]
pub struct MockOracle;

#[contractimpl]
impl MockOracle {
    pub fn get_price(env: Env, _asset: Address) -> i128 {
        env.storage().instance().get(&0u32).unwrap_or(100_000_000i128)
    }

    pub fn set_price(env: Env, price: i128) {
        env.storage().instance().set(&0u32, &price);
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn setup(env: &Env) -> (FlashLoanGuardContractClient, MockOracleClient, Address) {
    let admin = Address::generate(env);
    let asset = Address::generate(env);

    let oracle_id = env.register(MockOracle, ());
    let oracle = MockOracleClient::new(env, &oracle_id);
    oracle.set_price(&100_000_000i128); // $1.00

    let contract_id = env.register(FlashLoanGuardContract, ());
    let client = FlashLoanGuardContractClient::new(env, &contract_id);

    client.initialize(&Config {
        admin,
        oracle: oracle_id,
        guarded_asset: asset.clone(),
        max_intra_ledger_deviation_bps: 200, // 2%
        min_ledger_gap: 1,
    });

    (client, oracle, asset)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn test_record_and_safe_price() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_sequence_number(100);

    let (client, _oracle, _asset) = setup(&env);

    client.record_snapshot();
    let snap = client.get_snapshot().unwrap();
    assert_eq!(snap.price, 100_000_000);
    assert_eq!(snap.ledger, 100);

    // Advance ledger, price unchanged → safe
    env.ledger().set_sequence_number(101);
    let price = client.assert_price_safe();
    assert_eq!(price, 100_000_000);
}

#[test]
#[should_panic]
fn test_no_snapshot_blocks_operation() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_sequence_number(100);

    let (client, _oracle, _asset) = setup(&env);

    // No snapshot recorded yet — must panic
    client.assert_price_safe();
}

#[test]
#[should_panic]
fn test_same_ledger_snapshot_blocked() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_sequence_number(100);

    let (client, _oracle, _asset) = setup(&env);

    // Record snapshot at ledger 100
    client.record_snapshot();

    // Try to use it in the SAME ledger — flash-loan attack pattern
    client.assert_price_safe();
}

#[test]
#[should_panic]
fn test_price_spike_blocked() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_sequence_number(100);

    let (client, oracle, _asset) = setup(&env);

    client.record_snapshot(); // snapshot at $1.00, ledger 100

    // Advance ledger, then spike price by 10% (exceeds 2% threshold)
    env.ledger().set_sequence_number(101);
    oracle.set_price(&110_000_000i128); // $1.10 → 10% deviation

    // Must panic: deviation exceeds max_intra_ledger_deviation_bps
    client.assert_price_safe();
}

#[test]
fn test_small_price_move_allowed() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_sequence_number(100);

    let (client, oracle, _asset) = setup(&env);

    client.record_snapshot(); // snapshot at $1.00

    // Advance ledger, move price by 1% (within 2% threshold)
    env.ledger().set_sequence_number(101);
    oracle.set_price(&101_000_000i128); // $1.01 → 1% deviation

    let price = client.assert_price_safe();
    assert_eq!(price, 101_000_000);
}

#[test]
#[should_panic]
fn test_min_ledger_gap_enforced() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_sequence_number(100);

    let (client, _oracle, _asset) = setup(&env);

    client.record_snapshot(); // ledger 100

    // Try to record again at ledger 100 (gap = 0, min = 1) — must panic
    client.record_snapshot();
}

#[test]
fn test_snapshot_can_be_updated_after_gap() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_sequence_number(100);

    let (client, oracle, _asset) = setup(&env);

    client.record_snapshot(); // ledger 100, price $1.00

    // Advance past min_ledger_gap
    env.ledger().set_sequence_number(101);
    oracle.set_price(&101_000_000i128);
    client.record_snapshot(); // should succeed

    let snap = client.get_snapshot().unwrap();
    assert_eq!(snap.price, 101_000_000);
    assert_eq!(snap.ledger, 101);
}

#[test]
#[should_panic]
fn test_price_drop_blocked() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_sequence_number(100);

    let (client, oracle, _asset) = setup(&env);

    client.record_snapshot(); // $1.00

    env.ledger().set_sequence_number(101);
    oracle.set_price(&85_000_000i128); // $0.85 → 15% drop — must panic

    client.assert_price_safe();
}

#[test]
#[should_panic]
fn test_double_initialize() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, oracle_client, asset) = setup(&env);
    let admin = Address::generate(&env);
    let oracle_id = env.register(MockOracle, ());
    client.initialize(&Config {
        admin,
        oracle: oracle_id,
        guarded_asset: asset,
        max_intra_ledger_deviation_bps: 200,
        min_ledger_gap: 1,
    });
}
