module 0x1::bench_module {
    use sui::object::{Self, ID, UID};
    use sui::transfer;
    use sui::tx_context::{Self, TxContext};
    use sui::balance::{Self, Balance};
    use sui::coin::{Self, Coin};
    use sui::table::{Self, Table};
    use sui::vec_map::{Self, VecMap};
    use sui::vec_set::{Self, VecSet};

    // Large number of structs for testing
    struct DataObject1 has key { id: UID, value: u64 }
    struct DataObject2 has key { id: UID, value: u64 }
    struct DataObject3 has key { id: UID, value: u64 }
    struct DataObject4 has key { id: UID, value: u64 }
    struct DataObject5 has key { id: UID, value: u64 }

    // Capability structs
    struct AdminCap has key { id: UID }
    struct ModifyCap has key { id: UID }
    struct TransferCap has key { id: UID }

    // Functions for testing performance
    public fun create_object1(_cap: &AdminCap, ctx: &mut TxContext): DataObject1 {
        DataObject1 { id: object::new(ctx), value: 0 }
    }

    public fun modify_object1(_cap: &ModifyCap, obj: &mut DataObject1, value: u64) {
        obj.value = value;
    }

    public fun transfer_object1(_cap: &TransferCap, obj: DataObject1, recipient: address) {
        transfer::transfer(obj, recipient);
    }

    // Complex functions with multiple operations
    public fun batch_create(
        admin_cap: &AdminCap,
        count: u64,
        ctx: &mut TxContext
    ): vector<DataObject1> {
        let i = 0;
        let objects = vector::empty();
        while (i < count) {
            vector::push_back(&mut objects, create_object1(admin_cap, ctx));
            i = i + 1;
        };
        objects
    }

    public fun batch_modify(
        modify_cap: &ModifyCap,
        objects: &mut vector<DataObject1>,
        values: vector<u64>
    ) {
        let i = 0;
        let len = vector::length(objects);
        while (i < len) {
            let obj = vector::borrow_mut(objects, i);
            let value = vector::borrow(&values, i);
            modify_object1(modify_cap, obj, *value);
            i = i + 1;
        }
    }

    public fun batch_transfer(
        transfer_cap: &TransferCap,
        objects: vector<DataObject1>,
        recipients: vector<address>
    ) {
        let i = 0;
        let len = vector::length(&objects);
        while (i < len) {
            let obj = vector::pop_back(&mut objects);
            let recipient = vector::borrow(&recipients, i);
            transfer_object1(transfer_cap, obj, *recipient);
            i = i + 1;
        };
        vector::destroy_empty(objects);
    }

    // Functions with complex logic and multiple paths
    public fun complex_operation(
        admin_cap: &AdminCap,
        modify_cap: &ModifyCap,
        transfer_cap: &TransferCap,
        count: u64,
        threshold: u64,
        recipient: address,
        ctx: &mut TxContext
    ) {
        let objects = batch_create(admin_cap, count, ctx);
        let values = vector::empty();
        let i = 0;
        while (i < count) {
            let value = if (i < threshold) {
                i * 100
            } else {
                i * 200
            };
            vector::push_back(&mut values, value);
            i = i + 1;
        };
        batch_modify(modify_cap, &mut objects, values);
        
        let recipients = vector::empty();
        i = 0;
        while (i < count) {
            vector::push_back(&mut recipients, recipient);
            i = i + 1;
        };
        batch_transfer(transfer_cap, objects, recipients);
    }
} 