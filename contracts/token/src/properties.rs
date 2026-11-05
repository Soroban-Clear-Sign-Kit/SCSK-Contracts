#![cfg(test)]

use super::*;
use ::proptest::prelude::*;
use soroban_sdk::{
    testutils::{Address as _, EnvTestConfig},
    Env, String,
};

fn setup_token(env: &mut Env) -> (TokenClient<'_>, Address, [Address; 3]) {
    env.set_config(EnvTestConfig {
        capture_snapshot_at_drop: false,
    });
    env.mock_all_auths();
    let contract_id = env.register(Token, ());
    let client = TokenClient::new(env, &contract_id);
    let admin = Address::generate(env);
    client.initialize(
        &admin,
        &7,
        &String::from_str(env, "Token"),
        &String::from_str(env, "TKN"),
    );

    let users = [
        Address::generate(env),
        Address::generate(env),
        Address::generate(env),
    ];

    (client, admin, users)
}

#[derive(Clone, Debug)]
enum TokenAction {
    Transfer {
        from_idx: usize,
        to_idx: usize,
        amount: i128,
    },
    TransferFrom {
        spender_idx: usize,
        from_idx: usize,
        to_idx: usize,
        amount: i128,
    },
    Burn {
        from_idx: usize,
        amount: i128,
    },
    BurnFrom {
        spender_idx: usize,
        from_idx: usize,
        amount: i128,
    },
}

fn action_strategy() -> impl Strategy<Value = TokenAction> {
    prop_oneof![
        (0..3usize, 0..3usize, 0..=20_000_i128).prop_map(|(from_idx, to_idx, amount)| {
            TokenAction::Transfer {
                from_idx,
                to_idx,
                amount,
            }
        }),
        (0..3usize, 0..3usize, 0..3usize, 0..=20_000_i128).prop_map(
            |(spender_idx, from_idx, to_idx, amount)| {
                TokenAction::TransferFrom {
                    spender_idx,
                    from_idx,
                    to_idx,
                    amount,
                }
            }
        ),
        (0..3usize, 0..=20_000_i128)
            .prop_map(|(from_idx, amount)| TokenAction::Burn { from_idx, amount }),
        (0..3usize, 0..3usize, 0..=20_000_i128).prop_map(|(spender_idx, from_idx, amount)| {
            TokenAction::BurnFrom {
                spender_idx,
                from_idx,
                amount,
            }
        }),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// Property 1: Total supply is conserved across random transfer, burn, and transfer_from sequences.
    #[test]
    fn prop_total_supply_conserved(
        initial_balances in prop::array::uniform3(0..=10_000_i128),
        actions in prop::collection::vec(action_strategy(), 1..=10),
    ) {
        let mut env = Env::default();
        let (client, _admin, users) = setup_token(&mut env);

        let mut initial_supply = 0i128;
        for (i, &bal) in initial_balances.iter().enumerate() {
            if bal > 0 {
                client.mint(&users[i], &bal);
            }
            initial_supply += bal;
        }

        // Setup mutual allowances so transfer_from and burn_from can succeed
        for i in 0..3 {
            for j in 0..3 {
                if i != j {
                    client.approve(&users[i], &users[j], &100_000, &100_000);
                }
            }
        }

        let mut total_burned = 0i128;

        for action in actions {
            match action {
                TokenAction::Transfer { from_idx, to_idx, amount } => {
                    let _ = client.try_transfer(&users[from_idx], &users[to_idx], &amount);
                }
                TokenAction::TransferFrom { spender_idx, from_idx, to_idx, amount } => {
                    let _ = client.try_transfer_from(&users[spender_idx], &users[from_idx], &users[to_idx], &amount);
                }
                TokenAction::Burn { from_idx, amount } => {
                    if client.try_burn(&users[from_idx], &amount).is_ok() {
                        total_burned += amount;
                    }
                }
                TokenAction::BurnFrom { spender_idx, from_idx, amount } => {
                    if client.try_burn_from(&users[spender_idx], &users[from_idx], &amount).is_ok() {
                        total_burned += amount;
                    }
                }
            }
        }

        let final_supply: i128 = users.iter().map(|u| client.balance(u)).sum();
        prop_assert_eq!(final_supply + total_burned, initial_supply);
    }

    /// Property 2: Balances never go negative under any sequence of operations and arbitrary amounts.
    #[test]
    fn prop_balances_never_go_negative(
        initial_balances in prop::array::uniform3(0..=50_000_i128),
        amounts in prop::collection::vec(-50_000..=100_000_i128, 1..=8),
        ops in prop::collection::vec((0..3usize, 0..3usize, 0..5usize), 1..=8),
    ) {
        let mut env = Env::default();
        let (client, _admin, users) = setup_token(&mut env);

        for (i, &bal) in initial_balances.iter().enumerate() {
            if bal > 0 {
                client.mint(&users[i], &bal);
            }
        }

        for i in 0..3 {
            for j in 0..3 {
                if i != j {
                    let _ = client.try_approve(&users[i], &users[j], &50_000, &10_000);
                }
            }
        }

        let num_steps = amounts.len().min(ops.len());
        for k in 0..num_steps {
            let amount = amounts[k];
            let (u1, u2, op_type) = ops[k];
            match op_type {
                0 => { let _ = client.try_transfer(&users[u1], &users[u2], &amount); }
                1 => { let _ = client.try_transfer_from(&users[u1], &users[u2], &users[(u1 + 1) % 3], &amount); }
                2 => { let _ = client.try_burn(&users[u1], &amount); }
                3 => { let _ = client.try_burn_from(&users[u1], &users[u2], &amount); }
                _ => { let _ = client.try_mint(&users[u1], &amount); }
            }

            for u in &users {
                prop_assert!(client.balance(u) >= 0);
            }
        }
    }

    /// Property 3: Allowance never increases on transfer_from.
    #[test]
    fn prop_allowance_never_increases_on_transfer_from(
        initial_balance in 0..=50_000_i128,
        initial_allowance in 0..=50_000_i128,
        amount in -10_000..=60_000_i128,
        target_idx in 0..3usize,
    ) {
        let mut env = Env::default();
        let (client, _admin, users) = setup_token(&mut env);

        let owner = &users[0];
        let spender = &users[1];
        let recipient = &users[target_idx];

        if initial_balance > 0 {
            client.mint(owner, &initial_balance);
        }

        client.approve(owner, spender, &initial_allowance, &10_000);
        let before_allowance = client.allowance(owner, spender);
        prop_assert_eq!(before_allowance, initial_allowance);

        let res = client.try_transfer_from(spender, owner, recipient, &amount);
        let after_allowance = client.allowance(owner, spender);

        // Invariant 1: Allowance never increases
        prop_assert!(after_allowance <= before_allowance);

        // Invariant 2: If call succeeded, allowance decreased by exactly amount
        if res.is_ok() {
            prop_assert_eq!(after_allowance, before_allowance - amount);
        } else {
            // Invariant 3: If call failed, allowance remains unchanged
            prop_assert_eq!(after_allowance, before_allowance);
        }
    }

    /// Property 4: Failed calls leave state completely unchanged (atomic rollback).
    #[test]
    fn prop_failed_calls_leave_state_unchanged(
        initial_balances in prop::array::uniform3(0..=20_000_i128),
        fail_mode in 0..6usize,
        excess_or_neg in -10_000..=-1_i128,
    ) {
        let mut env = Env::default();
        let (client, _admin, users) = setup_token(&mut env);

        for (i, &bal) in initial_balances.iter().enumerate() {
            if bal > 0 {
                client.mint(&users[i], &bal);
            }
        }

        client.approve(&users[0], &users[1], &5_000, &10_000);
        client.approve(&users[1], &users[2], &5_000, &10_000);
        client.approve(&users[2], &users[0], &5_000, &10_000);

        // Snapshot all balances and allowances before the failing call
        let snap_b0 = client.balance(&users[0]);
        let snap_b1 = client.balance(&users[1]);
        let snap_b2 = client.balance(&users[2]);

        let snap_a01 = client.allowance(&users[0], &users[1]);
        let snap_a12 = client.allowance(&users[1], &users[2]);
        let snap_a20 = client.allowance(&users[2], &users[0]);

        let res = match fail_mode {
            0 => {
                // Transfer exceeding balance
                let over_amount = snap_b0 + 1000;
                client.try_transfer(&users[0], &users[1], &over_amount)
            }
            1 => {
                // Negative transfer amount
                client.try_transfer(&users[0], &users[1], &excess_or_neg)
            }
            2 => {
                // TransferFrom exceeding allowance
                let over_allowance = snap_a01 + 1000;
                client.try_transfer_from(&users[1], &users[0], &users[2], &over_allowance)
            }
            3 => {
                // Negative burn amount
                client.try_burn(&users[0], &excess_or_neg)
            }
            4 => {
                // Burn exceeding balance
                let over_burn = snap_b0 + 1000;
                client.try_burn(&users[0], &over_burn)
            }
            _ => {
                // Negative approve amount
                client.try_approve(&users[0], &users[1], &excess_or_neg, &10_000)
            }
        };

        // Assert that the operation indeed failed
        prop_assert!(res.is_err());

        // Assert all balances remain strictly identical
        prop_assert_eq!(client.balance(&users[0]), snap_b0);
        prop_assert_eq!(client.balance(&users[1]), snap_b1);
        prop_assert_eq!(client.balance(&users[2]), snap_b2);

        // Assert all allowances remain strictly identical
        prop_assert_eq!(client.allowance(&users[0], &users[1]), snap_a01);
        prop_assert_eq!(client.allowance(&users[1], &users[2]), snap_a12);
        prop_assert_eq!(client.allowance(&users[2], &users[0]), snap_a20);
    }
}
