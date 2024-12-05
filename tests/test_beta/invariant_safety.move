module test::invariant_safety {
    use sui::object::{Self, UID};
    use sui::table::{Self, Table};
    use sui::tx_context::TxContext;

    struct Pool has key {
        id: UID,
        total_supply: u64,
        balances: Table<address, u64>,
        locked: bool,
        min_deposit: u64,
        max_supply: u64
    }

    // Should detect - invariant violation (overflow)
    public fun mint(pool: &mut Pool, amount: u64, recipient: address) {
        // Missing: assert!(pool.total_supply + amount >= pool.total_supply)
        pool.total_supply = pool.total_supply + amount;
        table::add(&mut pool.balances, recipient, amount);
    }

    // Should detect - state invariant violation
    public fun unsafe_operation(pool: &mut Pool) {
        // Missing: assert!(!pool.locked)
        pool.total_supply = 0;
    }

    // Should detect - bounds violation
    public fun unsafe_deposit(pool: &mut Pool, amount: u64) {
        // Missing: assert!(amount >= pool.min_deposit)
        // Missing: assert!(pool.total_supply + amount <= pool.max_supply)
        pool.total_supply = pool.total_supply + amount;
    }

    // Should pass - maintains all invariants
    public fun safe_operation(pool: &mut Pool, amount: u64) {
        assert!(!pool.locked, 0);
        assert!(amount >= pool.min_deposit, 1);
        assert!(pool.total_supply + amount <= pool.max_supply, 2);
        assert!(pool.total_supply + amount >= pool.total_supply, 3);
        pool.total_supply = pool.total_supply + amount;
    }
}
