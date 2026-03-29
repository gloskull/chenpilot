#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, vec, Env, Address, Vec};

/// One leg of a swap route.
#[contracttype]
#[derive(Clone)]
pub struct Hop {
    pub pool: Address,
    pub amount_in: i128,
    pub min_amount_out: i128,
}

/// Result of a single hop execution.
#[contracttype]
#[derive(Clone)]
pub struct HopResult {
    pub pool: Address,
    pub amount_out: i128,
}

/// Persistent pool state — price ratio stored as (numer, denom).
#[contracttype]
#[derive(Clone)]
pub enum PoolKey {
    Price(Address),
    Reserve(Address),
}

#[contract]
pub struct MultiHopSwap;

#[contractimpl]
impl MultiHopSwap {
    /// Seed a pool with an initial price ratio so swaps have something to read.
    pub fn seed_pool(env: Env, pool: Address, numer: i128, denom: i128) {
        env.storage().persistent().set(&PoolKey::Price(pool.clone()), &(numer, denom));
        env.storage().persistent().set(&PoolKey::Reserve(pool), &(numer * 1_000_i128));
    }

    /// Execute a multi-hop swap across `hops` pools.
    /// Each hop: reads pool price, computes output, writes updated reserve, emits event.
    /// Returns the final amount out.
    pub fn swap(env: Env, caller: Address, hops: Vec<Hop>) -> Vec<HopResult> {
        caller.require_auth();

        let mut results = vec![&env];
        let mut running_amount = 0_i128;

        for hop in hops.iter() {
            let (numer, denom): (i128, i128) = env
                .storage()
                .persistent()
                .get(&PoolKey::Price(hop.pool.clone()))
                .unwrap_or((1, 1));

            let reserve: i128 = env
                .storage()
                .persistent()
                .get(&PoolKey::Reserve(hop.pool.clone()))
                .unwrap_or(1_000_000);

            // constant-product style output: out = in * numer / denom
            let amount_out = hop.amount_in * numer / denom;

            if amount_out < hop.min_amount_out {
                panic!("slippage exceeded");
            }

            // update reserve
            let new_reserve = reserve - amount_out;
            env.storage()
                .persistent()
                .set(&PoolKey::Reserve(hop.pool.clone()), &new_reserve);

            env.events().publish(
                (symbol_short!("hop"), hop.pool.clone()),
                (hop.amount_in, amount_out),
            );

            running_amount += amount_out;
            results.push_back(HopResult { pool: hop.pool.clone(), amount_out });
        }

        // one final storage write to record cumulative output
        env.storage()
            .instance()
            .set(&symbol_short!("last_out"), &running_amount);

        results
    }
}

mod test;
