module test::reference_safety {
    use sui::object::{Self, UID};
    use sui::tx_context::TxContext;

    struct Data has key {
        id: UID,
        value: u64,
        ref_count: u64
    }

    // Should detect - reference escapes
    public fun unsafe_ref(data: &mut Data): &mut u64 {
        &mut data.value
    }

    // Should detect - stored reference
    struct RefHolder {
        value_ref: &mut u64
    }

    // Should detect - nested reference escape
    public fun nested_ref_escape(data: &mut Data): RefHolder {
        RefHolder {
            value_ref: &mut data.value
        }
    }

    // Should detect - reference array escape
    public fun array_ref_escape(data: &mut Data): vector<&mut u64> {
        let refs = vector::empty();
        vector::push_back(&mut refs, &mut data.value);
        refs
    }

    // Should pass - scoped reference
    public fun safe_ref(data: &mut Data) {
        let value = &mut data.value;
        *value = 100;
    }

    // Should pass - proper reference cleanup
    public fun safe_ref_cleanup(data: &mut Data) {
        let value = &mut data.value;
        *value = 100;
        data.ref_count = data.ref_count + 1;
    }
}
