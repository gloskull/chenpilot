#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Env, Address, BytesN};

// ~1 hour at 5s/ledger
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

// ── Standardized event data structs ──────────────────────────────────────────
// Every event follows: topics = (symbol, contract_id), data = typed struct.
// This lets the SDK/backend decode state purely from the event stream.

#[contracttype]
#[derive(Clone)]
pub struct EvtInit {
    pub admin: Address,
}

#[contracttype]
#[derive(Clone)]
pub struct EvtUpgradeProposed {
    pub admin: Address,
    pub new_wasm_hash: BytesN<32>,
    pub unlock_ledger: u32,
}

#[contracttype]
#[derive(Clone)]
pub struct EvtUpgradeCancelled {
    pub admin: Address,
}

#[contracttype]
#[derive(Clone)]
pub struct EvtUpgradeApplied {
    pub new_wasm_hash: BytesN<32>,
}

#[contracttype]
#[derive(Clone)]
pub struct EvtAdminTransferred {
    pub old_admin: Address,
    pub new_admin: Address,
}

// ─────────────────────────────────────────────────────────────────────────────

#[contract]
pub struct CoreVaultContract;

#[contractimpl]
impl CoreVaultContract {
    pub fn init(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);

        env.events().publish(
            (symbol_short!("init"), env.current_contract_address()),
            EvtInit { admin },
        );
    }

    pub fn propose_upgrade(env: Env, new_wasm_hash: BytesN<32>) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        let unlock_ledger = env.ledger().sequence() + TIMELOCK_LEDGERS;
        env.storage().instance().set(
            &DataKey::PendingUpgrade,
            &PendingUpgrade { new_wasm_hash: new_wasm_hash.clone(), unlock_ledger },
        );

        env.events().publish(
            (symbol_short!("upg_prop"), env.current_contract_address()),
            EvtUpgradeProposed { admin, new_wasm_hash, unlock_ledger },
        );
    }

    pub fn cancel_upgrade(env: Env) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        env.storage().instance().remove(&DataKey::PendingUpgrade);

        env.events().publish(
            (symbol_short!("upg_cncl"), env.current_contract_address()),
            EvtUpgradeCancelled { admin },
        );
    }

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

        // emit before wasm swap so the event lands in this execution context
        env.events().publish(
            (symbol_short!("upg_done"), env.current_contract_address()),
            EvtUpgradeApplied { new_wasm_hash: pending.new_wasm_hash.clone() },
        );

        env.deployer().update_current_contract_wasm(pending.new_wasm_hash);
    }

    pub fn transfer_admin(env: Env, new_admin: Address) {
        let old_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        old_admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &new_admin);

        env.events().publish(
            (symbol_short!("adm_xfer"), env.current_contract_address()),
            EvtAdminTransferred { old_admin, new_admin },
        );
    }

    pub fn upgrade_unlock_ledger(env: Env) -> u32 {
        env.storage()
            .instance()
            .get::<DataKey, PendingUpgrade>(&DataKey::PendingUpgrade)
            .map(|p| p.unlock_ledger)
            .unwrap_or(0)
    }
}

mod test;
