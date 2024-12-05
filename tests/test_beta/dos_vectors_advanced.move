module 0x1::dos_vectors_advanced {
    use sui::object::{Self, UID};
    use sui::tx_context::{Self, TxContext};
    use std::vector;

    struct ProcessingPool has key {
        id: UID,
        items: vector<Item>,
        processed: u64
    }

    struct Item has store {
        data: vector<u8>,
        status: u8
    }

    // VULNERABILITY: Nested loops with external calls
    public fun batch_process(
        pool: &mut ProcessingPool,
        batches: vector<vector<Item>>
    ) {
        let i = 0;
        let len = vector::length(&batches);
        
        while (i < len) {  // Outer loop
            let batch = vector::borrow(&batches, i);
            let j = 0;
            let batch_len = vector::length(batch);
            
            while (j < batch_len) {  // Inner loop
                let item = vector::borrow(batch, j);
                external_module::process_item(item);  // External call in nested loop
                j = j + 1;
            };
            
            i = i + 1;
        }
    }

    // VULNERABILITY: Unbounded loop with multiple external calls
    public fun process_with_validation(
        pool: &mut ProcessingPool,
        items: vector<Item>
    ) {
        let i = 0;
        let len = vector::length(&items);
        
        while (i < len) {
            let item = vector::borrow(&items, i);
            
            // Multiple external calls in loop
            external_module::validate(item);
            external_module::process_item(item);
            external_module::update_status(item);
            
            i = i + 1;
        }
    }

    // VULNERABILITY: Dynamic loop bound from external call
    public fun process_dynamic(
        pool: &mut ProcessingPool,
        ctx: &mut TxContext
    ) {
        let count = external_module::get_pending_count(ctx);  // Dynamic bound
        let i = 0;
        
        while (i < count) {  // Loop bound depends on external call
            let item = external_module::get_item(i, ctx);
            external_module::process_item(&item);
            i = i + 1;
        }
    }
} 