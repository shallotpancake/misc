//! Integration tests that validate the core architectural principle:
//! mechanics (spacetime) are independent of entities (matter).
//!
//! These tests demonstrate that:
//! 1. The mechanics layer resolves checks without knowing what entity is involved
//! 2. Modifier stacking follows PF2e rules regardless of source
//! 3. Conditions produce effects through the mechanics layer, not entity logic
//! 4. The spatial topology exists independently of entities
//! 5. The action economy applies uniformly to all entities

use pathfinder_mechanics::action::{ActionCost, ActionPool};
use pathfinder_mechanics::condition::Conditions;
use pathfinder_mechanics::condition::rules::ConditionRules;
use pathfinder_mechanics::condition::types::*;
use pathfinder_mechanics::entity::ability::{AbilityScores, AbilityType};
use pathfinder_mechanics::entity::creature::CreatureBundle;
use pathfinder_mechanics::entity::proficiency::Proficiency;
use pathfinder_mechanics::mechanics::check::{resolve_check, CheckContext, DegreeOfSuccess};
use pathfinder_mechanics::mechanics::modifier::{Modifier, ModifierStack, ModifierType};
use pathfinder_mechanics::mechanics::traits::GameTrait;
use pathfinder_mechanics::spatial::grid2d::Grid2D;
use pathfinder_mechanics::spatial::position::Position;
use pathfinder_mechanics::spatial::terrain::Terrain;
use pathfinder_mechanics::spatial::topology::Topology;

/// The mechanics layer resolves a check the same way whether it's a
/// goblin attacking a dragon or a god attacking a peasant. The "law"
/// doesn't care about the identity of the participants.
#[test]
fn mechanics_resolve_independently_of_entity_identity() {
    let context = CheckContext {
        natural_roll: 15,
        modifiers: ModifierStack::new()
            .with(Modifier::new(4, ModifierType::Ability, "strength"))
            .with(Modifier::new(8, ModifierType::Proficiency, "master at level 2")),
        dc: 25,
        degree_shifts: 0,
    };

    let result = resolve_check(&context);
    // 15 + 4 + 8 = 27 vs DC 25 → Success (not crit: 27 < 35)
    assert_eq!(result.total, 27);
    assert_eq!(result.final_degree, DegreeOfSuccess::Success);
}

/// Demonstrate that conditions produce modifiers through the rules system,
/// not through entity-specific logic. A frightened goblin and a frightened
/// dragon get the exact same penalty.
#[test]
fn conditions_apply_through_rules_not_entities() {
    let effects = ConditionRules::effects(ConditionType::Frightened, ConditionSeverity::Value(3));

    // The rules say: frightened N → status penalty of -N to everything
    assert_eq!(effects.len(), 1);
    match &effects[0] {
        ConditionEffect::ApplyModifier(target, modifier) => {
            assert_eq!(*target, ConditionModifierTarget::All);
            assert_eq!(modifier.value, -3);
            assert_eq!(modifier.modifier_type, ModifierType::Status);
        }
        _ => panic!("Expected ApplyModifier from frightened condition"),
    }

    // Now feed that modifier into a check — the mechanics layer doesn't
    // know or care that this modifier came from "frightened"
    let result = resolve_check(&CheckContext {
        natural_roll: 12,
        modifiers: ModifierStack::new()
            .with(Modifier::new(10, ModifierType::Untyped, "base"))
            .with(modifier_from_condition(ConditionType::Frightened, 3)),
        dc: 20,
        degree_shifts: 0,
    });
    // 12 + 10 - 3 = 19 vs DC 20 → Failure
    assert_eq!(result.total, 19);
    assert_eq!(result.final_degree, DegreeOfSuccess::Failure);
}

/// Helper: extract a modifier from condition rules
fn modifier_from_condition(condition: ConditionType, severity: u32) -> Modifier {
    let effects = ConditionRules::effects(condition, ConditionSeverity::Value(severity));
    for effect in effects {
        if let ConditionEffect::ApplyModifier(_, modifier) = effect {
            return modifier;
        }
    }
    panic!("No modifier found for condition");
}

/// The spatial topology computes distances and movement costs without
/// knowing what entity is moving. A pixie and a storm giant follow
/// the same spatial rules.
#[test]
fn spatial_topology_is_entity_agnostic() {
    let mut grid = Grid2D::new(20, 20);

    // Set up some terrain
    grid.set_terrain(Position::new(5, 5), Terrain::Difficult);
    grid.set_terrain(Position::new(10, 10), Terrain::Impassable);

    // Distance calculations don't involve entities at all
    let dist = grid.distance_in_feet(Position::new(0, 0), Position::new(3, 0));
    assert_eq!(dist, 15); // 3 cardinal × 5 feet

    // Terrain effects are properties of the field
    assert_eq!(grid.terrain_at(Position::new(5, 5)), Terrain::Difficult);
    assert_eq!(grid.terrain_at(Position::new(5, 5)).movement_cost(), Some(2));
    assert_eq!(grid.terrain_at(Position::new(10, 10)).movement_cost(), None);

    // Adjacency is a spatial concept, not an entity concept
    assert!(grid.is_adjacent(Position::new(3, 3), Position::new(4, 4)));
    assert!(!grid.is_adjacent(Position::new(0, 0), Position::new(2, 2)));
}

/// Modifier stacking rules apply the same way regardless of source.
/// A status bonus from a spell and a status bonus from a class feature
/// follow the same stacking law.
#[test]
fn modifier_stacking_is_source_agnostic() {
    let stack = ModifierStack::new()
        .with(Modifier::new(2, ModifierType::Status, "bless spell"))
        .with(Modifier::new(3, ModifierType::Status, "inspire courage"))
        .with(Modifier::new(1, ModifierType::Circumstance, "flanking"))
        .with(Modifier::new(-2, ModifierType::Status, "frightened 2"))
        .with(Modifier::new(-1, ModifierType::Circumstance, "wind"));

    let resolved = stack.resolve();
    // Status: best bonus +3, worst penalty -2 → net +1
    // Circumstance: best bonus +1, worst penalty -1 → net 0
    // Total: +1
    assert_eq!(resolved.total, 1);
}

/// The action economy constrains all entities equally.
/// A level 1 fighter and a level 20 wizard both get 3 actions.
#[test]
fn action_economy_applies_uniformly() {
    let mut pool = ActionPool::new_turn();
    assert_eq!(pool.remaining(), 3);

    // First attack: no MAP
    let attack_traits = vec![GameTrait::attack()];
    assert_eq!(pool.map.current_penalty(), 0);
    pool.spend(ActionCost::Actions(1), &attack_traits);

    // Second attack: -5 MAP
    assert_eq!(pool.map.current_penalty(), -5);
    pool.spend(ActionCost::Actions(1), &attack_traits);

    // Third attack: -10 MAP, 1 action remaining
    assert_eq!(pool.map.current_penalty(), -10);
    assert_eq!(pool.remaining(), 1);
}

/// Creature bundles are just data containers — they don't define behavior.
/// All behavior comes from the mechanics systems operating on their components.
#[test]
fn creatures_are_just_component_bundles() {
    let goblin = CreatureBundle::new("Goblin Warrior", 1, 15, 16);
    let dragon = CreatureBundle::new("Ancient Red Dragon", 20, 425, 45);

    // Both creatures' ability scores produce modifiers through the same function
    let goblin_str = AbilityScores {
        strength: 12,
        ..AbilityScores::default()
    };
    let dragon_str = AbilityScores {
        strength: 30,
        ..AbilityScores::default()
    };

    // The modifier function is the same for both — it's a law of the universe
    assert_eq!(goblin_str.modifier(AbilityType::Strength), 1);
    assert_eq!(dragon_str.modifier(AbilityType::Strength), 10);

    // Proficiency bonus follows the same formula
    assert_eq!(Proficiency::Trained.bonus(1), 3); // goblin level
    assert_eq!(Proficiency::Legendary.bonus(20), 28); // dragon level

    // Both start with the same action economy
    assert_eq!(goblin.actions.remaining(), 3);
    assert_eq!(dragon.actions.remaining(), 3);
}

/// Conditions component is just a data container. The rules for what
/// conditions DO are in the ConditionRules system, not in the component.
#[test]
fn condition_component_is_inert_data() {
    let mut conditions = Conditions::default();

    // Applying a condition is just adding data
    conditions.apply(ConditionType::Frightened, ConditionSeverity::Value(2));
    assert!(conditions.has(ConditionType::Frightened));
    assert_eq!(conditions.severity(ConditionType::Frightened), 2);

    // Higher severity overwrites
    conditions.apply(ConditionType::Frightened, ConditionSeverity::Value(4));
    assert_eq!(conditions.severity(ConditionType::Frightened), 4);

    // Reducing severity
    conditions.reduce(ConditionType::Frightened, 1);
    assert_eq!(conditions.severity(ConditionType::Frightened), 3);

    // The component doesn't know what frightened DOES — that's in ConditionRules
    let effects = ConditionRules::effects(ConditionType::Frightened, ConditionSeverity::Value(3));
    assert!(!effects.is_empty());
}
