module test::consensus_safety {
    use sui::object::{Self, UID};
    use sui::tx_context::TxContext;

    struct SharedCounter has key {
        id: UID,
        value: u64,
        last_updated: u64,
        min_interval: u64
    }

    // Should detect - missing consensus
    public fun increment(counter: &mut SharedCounter) {
        // Missing: consensus::verify()
        counter.value = counter.value + 1;
    }

    // Should detect - missing timestamp check
    public fun update_value(counter: &mut SharedCounter, new_value: u64, clock: &Clock) {
        // Missing: assert!(clock::timestamp_ms(clock) >= counter.last_updated + counter.min_interval)
        counter.value = new_value;
    }

    // Should detect - race condition
    public fun compound_operation(counter: &mut SharedCounter) {
        // Missing: consensus::verify()
        let old_value = counter.value;
        counter.value = old_value * 2;
    }

    // Should pass - proper consensus and timing
    public fun safe_increment(counter: &mut SharedCounter, clock: &Clock) {
        consensus::verify();
        assert!(clock::timestamp_ms(clock) >= counter.last_updated + counter.min_interval, 0);
        counter.value = counter.value + 1;
        counter.last_updated = clock::timestamp_ms(clock);
    }
}
