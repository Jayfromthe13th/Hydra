// missing_ownership.move
module 0x1::missing_ownership {
    use sui::object::{Self, UID, ID};
    use sui::balance::{Self, Balance};
    use sui::coin::{Self, Coin};
    use sui::tx_context::{Self, TxContext};

    struct ProjectOwnerCap has key {
        id: UID,
        project_id: ID
    }

    struct Project<phantom T> has key {
        id: UID,
        balance: Balance<T>,
        owner: address
    }

    // VULNERABILITY: From paper section 4.4 - Missing owner field validation
    public fun transfer_ownership(
        project: &mut Project<T>,
        new_owner: address,
        ctx: &mut TxContext
    ) {
        // Missing: assert!(tx_context::sender(ctx) == project.owner)
        project.owner = new_owner;
    }

    // VULNERABILITY: From paper - Improper object wrapping
    public fun wrap_project<T>(
        project: Project<T>,
        wrapper: &mut WrappedProject<T>
    ) {
        // Missing: verify_wrapping_conditions(&project)
        wrapper.project = Some(project);
    }

    // VULNERABILITY: From paper - Unauthorized unwrapping
    public fun unwrap_project<T>(
        wrapper: &mut WrappedProject<T>,
        ctx: &mut TxContext
    ): Project<T> {
        // Missing: assert!(wrapper.owner == tx_context::sender(ctx))
        let project = wrapper.project.extract();
        project
    }
}