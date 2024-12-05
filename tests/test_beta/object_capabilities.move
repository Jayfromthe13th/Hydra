module 0x1::object_capabilities {
    use sui::object::{Self, UID};
    use sui::tx_context::{Self, TxContext};
    use sui::transfer;
    use sui::coin::{Self, Coin};
    use sui::balance::{Self, Balance};
    use sui::clock::{Self, Clock};

    struct Treasury has key {
        id: UID,
        balance: Balance<SUI>,
        admin: address,
        total_withdrawn: u64
    }

    struct AdminCap has key {
        id: UID,
        treasury_id: ID,
        permissions: u8
    }

    struct DelegateCap has key {
        id: UID,
        treasury_id: ID,
        permissions: u8,
        expiry: u64,
        max_amount: u64
    }

    // VULNERABILITY: Missing capability check
    public fun withdraw(
        treasury: &mut Treasury,
        amount: u64,
        ctx: &mut TxContext
    ): Coin<SUI> {
        // Missing: assert!(tx_context::sender(ctx) == treasury.admin)
        let coin = coin::from_balance(
            balance::split(&mut treasury.balance, amount),
            ctx
        );
        treasury.total_withdrawn = treasury.total_withdrawn + amount;
        coin
    }

    // VULNERABILITY: Improper capability delegation
    public fun delegate_access(
        _admin_cap: &AdminCap,
        treasury_id: ID,
        permissions: u8,
        expiry: u64,
        max_amount: u64,
        ctx: &mut TxContext
    ): DelegateCap {
        // Missing: verify admin_cap matches treasury_id
        // Missing: verify permissions are subset of admin permissions
        DelegateCap {
            id: object::new(ctx),
            treasury_id,
            permissions,
            expiry,
            max_amount
        }
    }

    // VULNERABILITY: No expiry check on delegated capability
    public fun use_delegated_cap(
        treasury: &mut Treasury,
        delegate_cap: &DelegateCap,
        amount: u64,
        clock: &Clock,
        ctx: &mut TxContext
    ): Coin<SUI> {
        // Missing: assert!(clock::timestamp_ms(clock) < delegate_cap.expiry)
        // Missing: verify delegate_cap.treasury_id matches treasury
        // Missing: assert!(amount <= delegate_cap.max_amount)
        withdraw(treasury, amount, ctx)
    }

    // VULNERABILITY: Unsafe capability transfer
    public fun transfer_admin_cap(
        admin_cap: AdminCap,
        new_admin: address,
        _ctx: &mut TxContext
    ) {
        // Missing: verify new_admin is valid
        // Missing: update treasury admin field
        transfer::transfer(admin_cap, new_admin);
    }
}