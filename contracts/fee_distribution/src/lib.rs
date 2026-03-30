#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, token};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Config,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
    pub admin: Address,
    pub treasury: Address,
    pub ai_agent_pool: Address,
    pub lp_pool: Address,
    pub treasury_bps: u32,
    pub ai_agent_bps: u32,
}

#[contract]
pub struct FeeDistributionContract;

#[contractimpl]
impl FeeDistributionContract {
    /// Initializes the fee distribution contract with recipient addresses and basis points.
    /// Basis points (bps) are out of 10,000 (e.g., 100 bps = 1%).
    pub fn initialize(
        env: Env,
        admin: Address,
        treasury: Address,
        ai_agent_pool: Address,
        lp_pool: Address,
        treasury_bps: u32,
        ai_agent_bps: u32,
    ) {
        if env.storage().instance().has(&DataKey::Config) {
            panic!("Already initialized");
        }
        if treasury_bps + ai_agent_bps > 10000 {
            panic!("Invalid basis points: sum exceeds 10000");
        }

        let config = Config {
            admin,
            treasury,
            ai_agent_pool,
            lp_pool,
            treasury_bps,
            ai_agent_bps,
        };
        env.storage().instance().set(&DataKey::Config, &config);
    }

    /// Updates the configuration. Only the admin can call this.
    pub fn update_config(env: Env, config: Config) {
        let current_config: Config = env.storage().instance().get(&DataKey::Config).expect("Not initialized");
        current_config.admin.require_auth();

        if config.treasury_bps + config.ai_agent_bps > 10000 {
            panic!("Invalid basis points: sum exceeds 10000");
        }

        env.storage().instance().set(&DataKey::Config, &config);
    }

    /// Distributes fees from the `from` address to Treasury, AI-Agent Pool, and LPs.
    /// The LP share is calculated as the remainder to ensure totals always reconcile.
    pub fn distribute(env: Env, token_addr: Address, from: Address, amount: i128) {
        if amount <= 0 {
            return;
        }

        let config: Config = env.storage().instance().get(&DataKey::Config).expect("Not initialized");
        
        // Calculate shares using deterministic basis points
        // treasury_share = amount * treasury_bps / 10000
        let treasury_share = amount
            .checked_mul(config.treasury_bps as i128)
            .expect("Multiplication overflow")
            .checked_div(10000)
            .expect("Division by zero");

        // ai_agent_share = amount * ai_agent_bps / 10000
        let ai_agent_share = amount
            .checked_mul(config.ai_agent_bps as i128)
            .expect("Multiplication overflow")
            .checked_div(10000)
            .expect("Division by zero");

        // lp_share = amount - treasury_share - ai_agent_share
        // This ensures every single unit of the fee (including rounding remainders) is distributed.
        let lp_share = amount
            .checked_sub(treasury_share)
            .expect("Subtraction underflow")
            .checked_sub(ai_agent_share)
            .expect("Subtraction underflow");

        let client = token::Client::new(&env, &token_addr);

        if treasury_share > 0 {
            client.transfer_from(&env.current_contract_address(), &from, &config.treasury, &treasury_share);
        }
        if ai_agent_share > 0 {
            client.transfer_from(&env.current_contract_address(), &from, &config.ai_agent_pool, &ai_agent_share);
        }
        if lp_share > 0 {
            client.transfer_from(&env.current_contract_address(), &from, &config.lp_pool, &lp_share);
        }
    }
    
    /// Returns the current configuration.
    pub fn get_config(env: Env) -> Config {
        env.storage().instance().get(&DataKey::Config).expect("Not initialized")
    }
}

mod test;
