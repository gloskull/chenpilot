#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Env, Address, BytesN};

// How many ledgers to wait before upgrade can be applied (~1 hour at 5s/ledger)
const TIMELOCK_LEDGERS: u32 = 720;

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    PendingUpgrade,
}

#[contracttype]
#[derive(Clone)]
pub struct PendingUpgrade {
    pub new_wasm_hash: BytesN<32>,
    pub unlock_ledger: u32,
}

#[contract]
pub struct CoreVaultContract;

#[contractimpl]
impl CoreVaultContract {
    /// Initialize the contract with an admin
    pub fn init(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    /// Propose an upgrade — starts the time-lock countdown
    pub fn propose_upgrade(env: Env, new_wasm_hash: BytesN<32>) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        let unlock_ledger = env.ledger().sequence() + TIMELOCK_LEDGERS;
        let pending = PendingUpgrade { new_wasm_hash, unlock_ledger };
        env.storage().instance().set(&DataKey::PendingUpgrade, &pending);
    }

    /// Cancel a pending upgrade (admin only)
    pub fn cancel_upgrade(env: Env) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        env.storage().instance().remove(&DataKey::PendingUpgrade);
    }

    /// Apply the upgrade after the time-lock has expired
    pub fn apply_upgrade(env: Env) {
        let pending: PendingUpgrade = env
            .storage()
            .instance()
            .get(&DataKey::PendingUpgrade)
            .expect("no pending upgrade");

        if env.ledger().sequence() < pending.unlock_ledger {
            panic!("time-lock not expired");
        }

        env.storage().instance().remove(&DataKey::PendingUpgrade);
        env.deployer().update_current_contract_wasm(pending.new_wasm_hash);
    }

    /// Transfer admin to a new address
    pub fn transfer_admin(env: Env, new_admin: Address) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &new_admin);
    }

    /// Returns the ledger at which the pending upgrade unlocks, or 0 if none
    pub fn upgrade_unlock_ledger(env: Env) -> u32 {
        env.storage()
            .instance()
            .get::<DataKey, PendingUpgrade>(&DataKey::PendingUpgrade)
            .map(|p| p.unlock_ledger)
            .unwrap_or(0)
    }
}

mod test;
