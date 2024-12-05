// arithmetic_vulnerabilities.move
module 0x1::arithmetic_vulns {
    use sui::object::{Self, UID};
    use sui::tx_context::{Self, TxContext};
    use sui::balance::{Self, Balance};
    use sui::coin::{Self, Coin};
    use std::vector;

    struct Pool has key {
        id: UID,
        balance: Balance<SUI>,
        total_shares: u64,
        last_update: u64
    }

    // VULNERABILITY: Unchecked arithmetic operation
    public fun calculate_shares(
        amount: u64,
        total_supply: u64,
        pool_balance: u64
    ): u64 {
        // Missing overflow check
        (amount * total_supply) / pool_balance  // Potential overflow before division
    }

    // VULNERABILITY: Multiple unchecked operations
    public fun mint_shares(
        pool: &mut Pool,
        coin: Coin<SUI>,
        ctx: &mut TxContext
    ): u64 {
        let amount = coin::value(&coin);
        let shares = calculate_shares(
            amount,
            pool.total_shares,
            balance::value(&pool.balance)
        );
        
        // Missing checks for:
        // - Division by zero
        // - Overflow in multiplication
        // - Underflow in subtraction
        pool.total_shares = pool.total_shares + shares;
        
        shares
    }

    // VULNERABILITY: Timestamp manipulation
    public fun update_pool_state(
        pool: &mut Pool,
        new_timestamp: u64,
        ctx: &mut TxContext
    ) {
        // Missing: verify timestamp is greater than last_update
        // Missing: verify timestamp is not in future
        pool.last_update = new_timestamp;
    }

    // VULNERABILITY: Vector length overflow
    public fun batch_mint(
        pool: &mut Pool,
        amounts: vector<u64>,
        ctx: &mut TxContext
    ): u64 {
        let i = 0;
        let total_shares = 0;
        let len = vector::length(&amounts);

        while (i < len) {
            let amount = *vector::borrow(&amounts, i);
            // Unchecked addition could overflow
            total_shares = total_shares + calculate_shares(
                amount,
                pool.total_shares,
                balance::value(&pool.balance)
            );
            i = i + 1;
        };

        // Missing overflow check
        pool.total_shares = pool.total_shares + total_shares;
        total_shares
    }
}