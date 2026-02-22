//! # Condition Layer (Layer 2)
//!
//! Conditions are mechanical states the field imposes on entities.
//! When an entity has "frightened 2", the mechanics layer knows exactly
//! what modifiers that produces. The entity doesn't decide this.

pub mod rules;
pub mod types;

use bevy::prelude::*;

use crate::EngineSet;
use crate::mechanics::modifier::Modifier;

pub use rules::ConditionRules;
pub use types::*;

pub struct ConditionPlugin;

impl Plugin for ConditionPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ConditionAppliedEvent>()
            .add_message::<ConditionRemovedEvent>()
            .add_message::<ConditionChangedEvent>()
            .add_systems(
                Update,
                end_of_turn_condition_decay.in_set(EngineSet::ApplyEffects),
            );
    }
}

/// Bridge: collect all modifiers that the mechanics layer should apply
/// based on an entity's active conditions. This is the wiring between
/// the condition layer (what states are active) and the mechanics layer
/// (what modifiers apply to checks).
///
/// Consumers building a `CheckContext` call this to automatically include
/// condition-based modifiers without knowing condition internals.
pub fn collect_condition_modifiers(conditions: &Conditions) -> Vec<Modifier> {
    let mut modifiers = Vec::new();
    for active in &conditions.active {
        let effects = ConditionRules::effects(active.condition, active.severity);
        for effect in effects {
            if let ConditionEffect::ApplyModifier(_, modifier) = effect {
                modifiers.push(modifier);
            }
        }
    }
    modifiers
}

/// Bridge: collect only modifiers that apply to a specific target
/// (e.g., only ArmorClass modifiers, or only AttackRolls modifiers).
/// Also includes modifiers targeting `All`.
pub fn collect_targeted_condition_modifiers(
    conditions: &Conditions,
    target: &ConditionModifierTarget,
) -> Vec<Modifier> {
    let mut modifiers = Vec::new();
    for active in &conditions.active {
        let effects = ConditionRules::effects(active.condition, active.severity);
        for effect in effects {
            if let ConditionEffect::ApplyModifier(ref effect_target, modifier) = effect {
                if effect_target == target || *effect_target == ConditionModifierTarget::All {
                    modifiers.push(modifier);
                }
            }
        }
    }
    modifiers
}

/// Component: all active conditions on an entity.
#[derive(Component, Debug, Clone, Default)]
pub struct Conditions {
    pub active: Vec<ActiveCondition>,
}

impl Conditions {
    pub fn has(&self, condition: ConditionType) -> bool {
        self.active.iter().any(|c| c.condition == condition)
    }

    pub fn severity(&self, condition: ConditionType) -> u32 {
        self.active
            .iter()
            .find(|c| c.condition == condition)
            .map(|c| c.severity.value())
            .unwrap_or(0)
    }

    pub fn apply(&mut self, condition: ConditionType, severity: ConditionSeverity) {
        if let Some(existing) = self.active.iter_mut().find(|c| c.condition == condition) {
            if severity.value() > existing.severity.value() {
                existing.severity = severity;
            }
        } else {
            self.active.push(ActiveCondition {
                condition,
                severity,
            });
        }
    }

    pub fn remove(&mut self, condition: ConditionType) {
        self.active.retain(|c| c.condition != condition);
    }

    pub fn reduce(&mut self, condition: ConditionType, amount: u32) {
        if let Some(existing) = self.active.iter_mut().find(|c| c.condition == condition) {
            match existing.severity {
                ConditionSeverity::Active => {
                    self.active.retain(|c| c.condition != condition);
                }
                ConditionSeverity::Value(v) => {
                    if v <= amount {
                        self.active.retain(|c| c.condition != condition);
                    } else {
                        existing.severity = ConditionSeverity::Value(v - amount);
                    }
                }
            }
        }
    }
}

/// An active condition on an entity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveCondition {
    pub condition: ConditionType,
    pub severity: ConditionSeverity,
}

/// Message: a condition was applied to an entity.
#[derive(Message, Debug, Clone)]
pub struct ConditionAppliedEvent {
    pub entity: Entity,
    pub condition: ConditionType,
    pub severity: ConditionSeverity,
}

/// Message: a condition was removed from an entity.
#[derive(Message, Debug, Clone)]
pub struct ConditionRemovedEvent {
    pub entity: Entity,
    pub condition: ConditionType,
}

/// Message: a condition's severity changed.
#[derive(Message, Debug, Clone)]
pub struct ConditionChangedEvent {
    pub entity: Entity,
    pub condition: ConditionType,
    pub old_severity: u32,
    pub new_severity: u32,
}

/// Marker component: signals that this entity's turn just ended.
/// The game layer adds this; the condition system reads it.
#[derive(Component)]
pub struct TurnEnded;

/// System: decay conditions that reduce at end of turn (e.g., frightened).
fn end_of_turn_condition_decay(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Conditions), With<TurnEnded>>,
    mut changed: MessageWriter<ConditionChangedEvent>,
    mut removed: MessageWriter<ConditionRemovedEvent>,
) {
    for (entity, mut conditions) in &mut query {
        let decaying: Vec<ConditionType> = conditions
            .active
            .iter()
            .filter(|c| ConditionRules::decreases_at_end_of_turn(c.condition))
            .map(|c| c.condition)
            .collect();

        for condition in decaying {
            let old = conditions.severity(condition);
            conditions.reduce(condition, 1);
            let new = conditions.severity(condition);

            if new == 0 {
                removed.write(ConditionRemovedEvent { entity, condition });
            } else {
                changed.write(ConditionChangedEvent {
                    entity,
                    condition,
                    old_severity: old,
                    new_severity: new,
                });
            }
        }

        commands.entity(entity).remove::<TurnEnded>();
    }
}
