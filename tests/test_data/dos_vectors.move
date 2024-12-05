module 0x1::dos_vectors {
    use sui::object::{Self, UID};
    use sui::tx_context::{Self, TxContext};
    use std::vector;

    struct DataStore has key {
        id: UID,
        items: vector<vector<u64>>
    }

    // VULNERABILITY: Nested loops with external calls
    public fun process_nested_data(store: &mut DataStore) {
        let i = 0;
        let outer_len = vector::length(&store.items);
        
        while (i < outer_len) {
            let inner = vector::borrow(&store.items, i);
            let j = 0;
            while (j < vector::length(inner)) {
                external_module::process_item(j);  // External call in nested loop
                j = j + 1;
            }
            i = i + 1;
        }
    }

    // VULNERABILITY: Unbounded vector operation
    public fun add_items(store: &mut DataStore, items: vector<u64>) {
        let i = 0;
        while (i < vector::length(&items)) {
            let item = vector::borrow(&items, i);
            vector::push_back(&mut store.items, vector::singleton(*item));
            i = i + 1;
        }
    }

    // VULNERABILITY: Recursive external call
    public fun recursive_process(store: &mut DataStore, depth: u64) {
        if (depth > 0) {
            external_module::heavy_computation();  // External call
            recursive_process(store, depth - 1);  // Recursive call
        }
    }
} 