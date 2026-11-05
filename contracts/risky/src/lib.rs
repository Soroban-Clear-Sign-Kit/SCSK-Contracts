//! [TEST-ONLY] Benign test fixtures demonstrating risky-looking smart contract patterns.
//!
//! WARNING: This contract is strictly for testing, auditing, and clear-sign demonstration
//! in the Soroban Clear-Sign Kit (SCSK). It contains patterns intentionally designed to
//! trigger warning heuristics (e.g. infinite approvals, single-step admin transfers,
//! in-place bytecode upgrades, very large transfers, and protocol bounds stress tests).
//! DO NOT deploy or use these patterns in production contracts without appropriate safeguards!

#![no_std]

use soroban_sdk::{
    Address, Bytes, BytesN, Env, Vec, contract, contractevent, contractimpl, contracttype, token,
};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
}

#[contractevent(data_format = "single-value")]
pub struct AdminChanged {
    #[topic]
    pub old_admin: Address,
    pub new_admin: Address,
}

#[contractevent(data_format = "single-value")]
pub struct ContractUpgraded {
    #[topic]
    pub admin: Address,
    pub new_wasm_hash: BytesN<32>,
}

#[contractevent(data_format = "single-value")]
pub struct UnlimitedApprove {
    #[topic]
    pub from: Address,
    #[topic]
    pub spender: Address,
    pub amount: i128,
}

#[contractevent(data_format = "single-value")]
pub struct FreshSpenderApproved {
    #[topic]
    pub from: Address,
    #[topic]
    pub fresh_spender: Address,
    pub amount: i128,
}

#[contractevent(data_format = "single-value")]
pub struct LargeTransferExecuted {
    #[topic]
    pub from: Address,
    #[topic]
    pub to: Address,
    pub amount: i128,
}

#[contractevent(data_format = "single-value")]
pub struct OversizePayloadProcessed {
    pub payload_size: u32,
}

#[contractevent(data_format = "single-value")]
pub struct DeeplyNestedProcessed {
    pub total_elements: u32,
}

/// [TEST-ONLY] Contract demonstrating high-risk patterns for analyzer and decoder testing.
#[contract]
pub struct RiskyContract;

#[contractimpl]
impl RiskyContract {
    /// Initializes the contract with an administrator address.
    pub fn init(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    /// Returns the current administrator address.
    pub fn get_admin(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("admin not initialized")
    }

    /// [TEST-ONLY] Risky Pattern: Single-step admin transfer without two-step acceptance.
    ///
    /// Transferring administrative ownership in a single step without a pending/accept pattern
    /// risks catastrophic loss of contract control if transferred to an incorrect or malicious address.
    pub fn set_admin(env: Env, new_admin: Address) {
        let admin: Address = Self::get_admin(env.clone());
        admin.require_auth();

        env.storage().instance().set(&DataKey::Admin, &new_admin);

        AdminChanged {
            old_admin: admin,
            new_admin,
        }
        .publish(&env);
    }

    /// [TEST-ONLY] Risky Pattern: In-place contract bytecode replacement.
    ///
    /// Upgrading contract code via `update_current_contract_wasm` completely mutates the contract's
    /// business logic, storage schema, and security invariants in place.
    pub fn upgrade(env: Env, new_wasm_hash: BytesN<32>) {
        let admin: Address = Self::get_admin(env.clone());
        admin.require_auth();

        env.deployer()
            .update_current_contract_wasm(new_wasm_hash.clone());

        ContractUpgraded {
            admin,
            new_wasm_hash,
        }
        .publish(&env);
    }

    /// [TEST-ONLY] Risky Pattern: Unlimited allowance approval (i128::MAX).
    ///
    /// Granting `i128::MAX` allowance gives the spender unlimited authorization to transfer
    /// all current and future tokens from the owner without further owner signatures.
    pub fn unlimited_approve(
        env: Env,
        token: Address,
        from: Address,
        spender: Address,
        expiration_ledger: u32,
    ) {
        from.require_auth();

        token::Client::new(&env, &token).approve(&from, &spender, &i128::MAX, &expiration_ledger);

        UnlimitedApprove {
            from,
            spender,
            amount: i128::MAX,
        }
        .publish(&env);
    }

    /// [TEST-ONLY] Risky Pattern: Allowance approval to a fresh/unverified spender address.
    ///
    /// Approving an allowance to an address with zero prior transaction history is a hallmark
    /// of phishing dApps and newly deployed drainer contracts.
    pub fn approve_fresh_spender(
        env: Env,
        token: Address,
        from: Address,
        fresh_spender: Address,
        amount: i128,
        expiration_ledger: u32,
    ) {
        from.require_auth();

        token::Client::new(&env, &token).approve(
            &from,
            &fresh_spender,
            &amount,
            &expiration_ledger,
        );

        FreshSpenderApproved {
            from,
            fresh_spender,
            amount,
        }
        .publish(&env);
    }

    /// [TEST-ONLY] Risky Pattern: Very large transfer (high value or balance draining).
    ///
    /// Executing massive transfers relative to balances can lead to catastrophic treasury drainage
    /// or unintended slippage/liquidation.
    pub fn large_transfer(env: Env, token: Address, from: Address, to: Address, amount: i128) {
        from.require_auth();

        token::Client::new(&env, &token).transfer(&from, &to, &amount);

        LargeTransferExecuted { from, to, amount }.publish(&env);
    }

    /// [TEST-ONLY] Risky Pattern: Oversize argument payload for bounds testing.
    ///
    /// Stress-tests parser, deserializer, and VM memory bounds while remaining safely within
    /// Soroban protocol limits (e.g. transaction/entry size limits).
    pub fn oversize_payload(env: Env, payload: Bytes) -> u32 {
        let len = payload.len();
        OversizePayloadProcessed { payload_size: len }.publish(&env);
        len
    }

    /// [TEST-ONLY] Risky Pattern: Deeply nested data structure for bounds testing.
    ///
    /// Tests nested collection traversal and recursion depth limits within the Soroban VM.
    pub fn deeply_nested_structure(env: Env, levels: Vec<Vec<Vec<Vec<u32>>>>) -> u32 {
        let mut total = 0u32;
        for l1 in levels.iter() {
            for l2 in l1.iter() {
                for l3 in l2.iter() {
                    for val in l3.iter() {
                        total = total.saturating_add(val);
                    }
                }
            }
        }

        DeeplyNestedProcessed {
            total_elements: total,
        }
        .publish(&env);

        total
    }
}

#[cfg(test)]
mod test;
