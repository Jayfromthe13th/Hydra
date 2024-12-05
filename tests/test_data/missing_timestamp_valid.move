// missing_timestamp_validation.move
module 0x1::missing_timestamp_validation {
    use sui::object::{Self, UID, ID};
    use sui::balance::{Self, Balance};
    use sui::coin::{Self, Coin};
    use sui::tx_context::{Self, TxContext};
    use sui::clock::{Self, Clock};

    struct Leaderboard<phantom T> has key {
        id: UID,
        end_timestamp_ms: u64,
        creator: address
    }

    struct Project<phantom T> has key {
        id: UID,
        balance: Balance<T>,
        leaderboard_id: ID
    }

    // VULNERABILITY: Missing timestamp validation
    public fun create_project<T>(
        leaderboard: &mut Leaderboard<T>,
        deposit: Coin<T>,
        _clock: &Clock, // Clock parameter unused
        ctx: &mut TxContext
    ): Project<T> {
        // Missing: assert!(clock::timestamp_ms(clock) <= leaderboard.end_timestamp_ms)
        
        Project<T> {
            id: object::new(ctx),
            balance: coin::into_balance(deposit),
            leaderboard_id: object::id(leaderboard)
        }
    }

    // VULNERABILITY: Missing time-based validations
    public fun participate<T>(
        project: &mut Project<T>,
        leaderboard: &Leaderboard<T>,
        amount: Coin<T>,
        _clock: &Clock // Clock parameter unused
    ) {
        // Missing: assert!(clock::timestamp_ms(clock) <= leaderboard.end_timestamp_ms)
        balance::join(&mut project.balance, coin::into_balance(amount));
    }

    // VULNERABILITY: No time check on reward distribution
    public fun claim_rewards<T>(
        project: &mut Project<T>,
        leaderboard: &mut Leaderboard<T>,
        _clock: &Clock, // Clock parameter unused
        ctx: &mut TxContext
    ): Coin<T> {
        // Missing: assert!(clock::timestamp_ms(clock) > leaderboard.end_timestamp_ms)
        let amount = balance::value(&project.balance);
        coin::from_balance(balance::split(&mut project.balance, amount), ctx)
    }
}