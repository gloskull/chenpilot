#![cfg(test)]

extern crate std;
use std::println;

use super::*;
use soroban_sdk::{testutils::Address as _, vec, Env, Address};

const CPU_LIMIT: u64 = 100_000_000;
const MEM_LIMIT: u64 = 40 * 1024 * 1024;

fn setup(env: &Env) -> MultiHopSwapClient<'_> {
    let id = env.register(MultiHopSwap, ());
    MultiHopSwapClient::new(env, &id)
}

fn seed_pools(env: &Env, client: &MultiHopSwapClient<'_>, n: u32) -> soroban_sdk::Vec<Address> {
    let mut pools = vec![env];
    for _ in 0..n {
        let pool = Address::generate(env);
        client.seed_pool(&pool, &2, &1);
        pools.push_back(pool);
    }
    pools
}

fn build_hops(env: &Env, pools: &soroban_sdk::Vec<Address>) -> soroban_sdk::Vec<Hop> {
    let mut hops = vec![env];
    for pool in pools.iter() {
        hops.push_back(Hop { pool, amount_in: 100, min_amount_out: 1 });
    }
    hops
}

/// Run n hops under unlimited budget, return (cpu, mem).
fn measure(n: u32) -> (u64, u64) {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();
    let client = setup(&env);
    let pools = seed_pools(&env, &client, n);
    let hops = build_hops(&env, &pools);
    let caller = Address::generate(&env);
    client.swap(&caller, &hops);
    (
        env.cost_estimate().budget().cpu_instruction_cost(),
        env.cost_estimate().budget().memory_bytes_cost(),
    )
}

#[test]
fn test_single_hop_cost() {
    let (cpu, mem) = measure(1);
    println!("[1 hop] cpu={cpu}  mem={mem}");
    assert!(cpu > 0);
    assert!(cpu < CPU_LIMIT / 10, "single hop used {cpu}, expected < {}", CPU_LIMIT / 10);
    assert!(mem < MEM_LIMIT);
}

#[test]
fn test_hop_cost_scales() {
    let (cpu1, _) = measure(1);
    let (cpu5, _) = measure(5);
    let (cpu10, _) = measure(10);
    println!("[1 hop]  cpu={cpu1}");
    println!("[5 hops] cpu={cpu5}");
    println!("[10 hops] cpu={cpu10}");
    assert!(cpu5 > cpu1);
    assert!(cpu10 > cpu5);
}

#[test]
fn test_find_gas_breaking_point() {
    let mut lo: u32 = 1;
    let mut hi: u32 = 512;

    let (cpu_hi, _) = measure(hi);
    println!("[{hi} hops] cpu={cpu_hi}  limit={CPU_LIMIT}");

    if cpu_hi < CPU_LIMIT {
        println!("512 hops still under limit — raise hi");
        return;
    }

    while lo + 1 < hi {
        let mid = (lo + hi) / 2;
        let (cpu, mem) = measure(mid);
        println!("[{mid} hops] cpu={cpu}  mem={mem}");
        if cpu < CPU_LIMIT { lo = mid; } else { hi = mid; }
    }

    let (cpu_safe, mem_safe) = measure(lo);
    let (cpu_break, mem_break) = measure(hi);

    println!("SAFE:   {lo} hops  cpu={cpu_safe}  mem={mem_safe}");
    println!("BREAKS: {hi} hops  cpu={cpu_break}  mem={mem_break}");
    println!("CPU limit: {CPU_LIMIT}");

    assert!(cpu_safe < CPU_LIMIT);
    assert!(cpu_break >= CPU_LIMIT);
}

#[test]
fn test_memory_at_50_hops() {
    let (cpu, mem) = measure(50);
    let cpu_pct = cpu * 100 / CPU_LIMIT;
    let mem_pct = mem * 100 / MEM_LIMIT;
    println!("[50 hops] cpu={cpu} ({cpu_pct}%)  mem={mem} ({mem_pct}%)");
    assert!(mem <= MEM_LIMIT, "50 hops exceeded memory limit");
}

#[test]
#[should_panic(expected = "slippage exceeded")]
fn test_slippage_guard() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();
    let client = setup(&env);
    let pool = Address::generate(&env);
    client.seed_pool(&pool, &1, &2); // out < in
    let caller = Address::generate(&env);
    client.swap(&caller, &vec![&env, Hop { pool, amount_in: 100, min_amount_out: 999 }]);
}
