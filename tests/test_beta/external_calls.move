module 0x1::external_calls {
    use sui::object::{Self, UID};
    use sui::tx_context::{Self, TxContext};
    use std::vector;

    struct Pool has key {
        id: UID,
        items: vector<u64>
    }

    // VULNERABILITY: External call with vector operation
    public fun process_items(pool: &mut Pool, external_items: vector<u64>) {
        let i = 0;
        let len = vector::length(&external_items);
        
        // Loop over external input
        while (i < len) {
            let item = vector::borrow(&external_items, i);
            vector::push_back(&mut pool.items, *item);
            external_module::process(*item);  // External call in loop
            i = i + 1;
        }
    }

    // VULNERABILITY: Nested loops with external calls
    public fun batch_process(pool: &mut Pool, batch: vector<vector<u64>>) {
        let i = 0;
        while (i < vector::length(&batch)) {
            let inner = vector::borrow(&batch, i);
            process_items(pool, *inner); // Nested processing
            external_module::process_batch(inner);  // External call
            i = i + 1;
        }
    }

    // VULNERABILITY: Unbounded external iteration
    public fun iterate_external<T: key>(items: vector<T>) {
        let len = vector::length(&items);
        let i = 0;
        while (i < len) {
            external_module::process_item(i);  // External call in loop
            i = i + 1;
        }
    }
} 