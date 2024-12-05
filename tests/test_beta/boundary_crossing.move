module trusted::core {
    use sui::object::{Self, UID};
    use sui::tx_context::TxContext;

    struct SecretData has key {
        id: UID,
        value: u64,
        owner: address
    }

    struct AdminCap has key {
        id: UID,
        permissions: u8
    }

    // Should detect - untrusted access
    public fun leak_data(data: SecretData) {
        untrusted::module::receive(data)
    }

    // Should detect - improper delegation
    public fun unsafe_delegate(cap: AdminCap) {
        untrusted::module::receive_cap(cap)
    }

    // Should detect - indirect boundary crossing
    public fun indirect_leak(data: SecretData) {
        let wrapper = trusted::helper::wrap(data);
        untrusted::module::receive_wrapped(wrapper)
    }

    // Should detect - capability abuse
    public fun unsafe_cap_upgrade(cap: &mut AdminCap) {
        cap.permissions = 0xFF; // Elevate all permissions
    }

    // Should pass - proper boundary check
    public fun safe_access(data: &SecretData, ctx: &TxContext) {
        assert!(tx_context::sender(ctx) == data.owner, 0);
        // Safe access after verification
    }
}
