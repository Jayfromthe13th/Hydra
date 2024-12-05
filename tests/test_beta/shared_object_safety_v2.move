module 0x1::resource_safety {
    use sui::object::{Self, UID};
    use sui::tx_context::{Self, TxContext};
    use sui::dynamic_field;
    use std::vector;

    struct ResourceContainer has key {
        id: UID,
        resources: vector<Resource>,
        initialized: bool
    }

    struct Resource has store {
        data: vector<u8>,
        cleaned: bool
    }

    struct ResourceKey has copy, drop, store {
        index: u64
    }

    // VULNERABILITY: Resource leak in error path
    public fun initialize_container(
        container: &mut ResourceContainer,
        data: vector<u8>,
        ctx: &mut TxContext
    ) {
        assert!(!container.initialized, 0);
        
        let resource = Resource {
            data,
            cleaned: false
        };
        
        // If this fails, resource is leaked
        vector::push_back(&mut container.resources, resource);
        container.initialized = true;
    }

    // VULNERABILITY: Missing cleanup in dynamic fields
    public fun store_resource(
        container: &mut ResourceContainer,
        index: u64,
        data: vector<u8>
    ) {
        let resource = Resource {
            data,
            cleaned: false
        };
        
        // Missing cleanup of existing resource
        dynamic_field::add(&mut container.id, ResourceKey { index }, resource);
    }

    // VULNERABILITY: Improper resource transfer
    public fun transfer_resource(
        container: &mut ResourceContainer,
        index: u64,
        new_container: &mut ResourceContainer
    ) {
        let resource = vector::remove(&mut container.resources, index);
        // Missing: verify new_container can accept resource
        // Missing: update resource state
        vector::push_back(&mut new_container.resources, resource);
    }

    // VULNERABILITY: Incomplete cleanup
    public fun cleanup_container(container: &mut ResourceContainer) {
        let i = 0;
        let len = vector::length(&container.resources);
        
        while (i < len) {
            let resource = vector::borrow_mut(&mut container.resources, i);
            // Missing: cleanup associated dynamic fields
            // Missing: verify all resources are cleaned
            resource.cleaned = true;
            i = i + 1;
        }
    }
}