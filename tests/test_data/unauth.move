// unauthorized_withdrawal.move
module 0x1::unauth {
    use sui::object::{Self, UID, ID};
    use sui::balance::{Self, Balance};
    use sui::coin::{Self, Coin};
    use sui::tx_context::{Self, TxContext};
    use sui::transfer;

    // From paper section 2.1.1: "Move's type system ensures encapsulation"
    struct Treasury<phantom T> has key {
        id: UID,
        balance: Balance<T>,
        admin: address,
        frozen: bool,
        minimum_operation_amount: u64
    }

    struct TreasuryOperatorCap has key {
        id: UID,
        treasury_id: ID,
        permissions: u8
    }

    const PERMISSION_WITHDRAW: u8 = 1;
    const PERMISSION_FREEZE: u8 = 2;
    const PERMISSION_UPDATE: u8 = 4;

    // VULNERABILITY: Missing capability verification
    public fun withdraw<T>(
        treasury: &mut Treasury<T>,
        amount: u64,
        _cap: &TreasuryOperatorCap,
        ctx: &mut TxContext
    ): Coin<T> {
        // Missing: assert!(!treasury.frozen)
        // Missing: assert!(has_permission(cap, PERMISSION_WITHDRAW))
        // Missing: assert!(amount >= treasury.minimum_operation_amount)
        // Missing: assert!(treasury_id matches cap.treasury_id)
        
        coin::from_balance(
            balance::split(&mut treasury.balance, amount),
            ctx
        )
    }

    // VULNERABILITY: Unauthorized state modification
    public fun update_treasury<T>(
        treasury: &mut Treasury<T>,
        _cap: &TreasuryOperatorCap,
        new_minimum: u64,
        ctx: &mut TxContext
    ) {
        // Missing: assert!(has_permission(cap, PERMISSION_UPDATE))
        // Missing: assert!(tx_context::sender(ctx) == treasury.admin)
        treasury.minimum_operation_amount = new_minimum;
    }

    // VULNERABILITY: Missing multi-capability checks
    public fun freeze_treasury<T>(
        treasury: &mut Treasury<T>,
        _cap: &TreasuryOperatorCap,
        should_freeze: bool
    ) {
        // Missing: assert!(has_permission(cap, PERMISSION_FREEZE))
        // Missing: verify_treasury_state(treasury)
        treasury.frozen = should_freeze;
    }
}