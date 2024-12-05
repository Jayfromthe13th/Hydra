module 0x1::object_safety {
    use sui::object::{Self, UID};
    use sui::transfer;
    use sui::tx_context::{Self, TxContext};

    struct GameItem has key {
        id: UID,
        power: u64,
        owner: address,
        is_initialized: bool
    }

    struct AdminCap has key {
        id: UID
    }

    // Object safety issue: Unsafe object construction - missing initialization
    public fun create_uninitialized_item(ctx: &mut TxContext): GameItem {
        GameItem {
            id: object::new(ctx),
            power: 0,
            owner: @0x0,  // Invalid default owner
            is_initialized: false  // Object not properly initialized
        }
    }

    // Object safety issue: Invalid transfer guard - missing ownership check
    public fun unsafe_transfer(item: GameItem, recipient: address) {
        // Missing ownership validation
        // Missing initialization check
        // Missing recipient validation
        transfer::transfer(item, recipient);
    }

    // Object safety issue: Missing ownership check before modification
    public fun unsafe_power_update(item: &mut GameItem, new_power: u64) {
        // Missing ownership validation
        // Missing initialization check
        item.power = new_power;
    }

    // Object safety issue: Object might be used before initialization
    public fun use_item(item: &GameItem): u64 {
        // Missing initialization check
        item.power
    }
} 