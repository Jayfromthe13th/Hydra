// timestamp_manipulation.move
module 0x1::timestamp_manipulation {
    use sui::object::{Self, UID};
    use sui::clock::{Self, Clock};
    use sui::tx_context::{Self, TxContext};
    use sui::transfer;

    // From paper section 4.4: "Shared objects require consensus on state updates"
    struct Leaderboard has key {
        id: UID,
        end_timestamp_ms: u64,
        creator: address,
        min_duration: u64,
        max_duration: u64,
        locked: bool
    }

    struct AdminCap has key {
        id: UID,
        leaderboard_id: UID
    }

    public fun init(ctx: &mut TxContext) {
        transfer::transfer(
            AdminCap { id: object::new(ctx), leaderboard_id: object::new(ctx) },
            tx_context::sender(ctx)
        )
    }

    // VULNERABILITY: Timestamp manipulation without consensus
    public fun update_end_timestamp(
        leaderboard: &mut Leaderboard,
        end_timestamp_ms: u64,
        _ctx: &TxContext 
    ) {
        // Missing: consensus::assert_synchronized(leaderboard)
        // Missing: assert!(!leaderboard.locked)
        assert!(
            end_timestamp_ms > leaderboard.end_timestamp_ms,
            1
        );
        leaderboard.end_timestamp_ms = end_timestamp_ms;
    }

    // VULNERABILITY: Missing time bounds validation
    public fun extend_duration(
        leaderboard: &mut Leaderboard,
        extension_ms: u64,
        _clock: &Clock
    ) {
        // Missing: assert!(extension_ms >= leaderboard.min_duration)
        // Missing: assert!(extension_ms <= leaderboard.max_duration)
        // Missing: assert!(clock::timestamp_ms(clock) < leaderboard.end_timestamp_ms)
        leaderboard.end_timestamp_ms = leaderboard.end_timestamp_ms + extension_ms;
    }

    // VULNERABILITY: Missing clock validation on initialization
    public fun create_leaderboard(
        end_timestamp_ms: u64,
        _clock: &Clock,
        ctx: &mut TxContext
    ) {
        // Missing: assert!(clock::timestamp_ms(clock) < end_timestamp_ms)
        // Missing: validate_timestamp_bounds(end_timestamp_ms)
        transfer::share_object(Leaderboard {
            id: object::new(ctx),
            end_timestamp_ms,
            creator: tx_context::sender(ctx),
            min_duration: 3600000, // 1 hour in ms
            max_duration: 86400000, // 24 hours in ms
            locked: false
        })
    }
}