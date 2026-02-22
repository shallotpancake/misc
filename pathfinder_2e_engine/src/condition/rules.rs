use crate::mechanics::modifier::{Modifier, ModifierType};

use super::types::*;

/// Pure rules for how conditions produce mechanical effects.
/// Given a condition + severity → effects. No entity knowledge needed.
pub struct ConditionRules;

impl ConditionRules {
    pub fn effects(condition: ConditionType, severity: ConditionSeverity) -> Vec<ConditionEffect> {
        let val = severity.value() as i32;
        match condition {
            ConditionType::Frightened => vec![ConditionEffect::ApplyModifier(
                ConditionModifierTarget::All,
                Modifier::new(-val, ModifierType::Status, "frightened"),
            )],

            ConditionType::Sickened => vec![ConditionEffect::ApplyModifier(
                ConditionModifierTarget::All,
                Modifier::new(-val, ModifierType::Status, "sickened"),
            )],

            ConditionType::Clumsy => vec![
                ConditionEffect::ApplyModifier(
                    ConditionModifierTarget::DexterityBased,
                    Modifier::new(-val, ModifierType::Status, "clumsy"),
                ),
                ConditionEffect::ApplyModifier(
                    ConditionModifierTarget::ArmorClass,
                    Modifier::new(-val, ModifierType::Status, "clumsy"),
                ),
            ],

            ConditionType::Enfeebled => vec![ConditionEffect::ApplyModifier(
                ConditionModifierTarget::StrengthBased,
                Modifier::new(-val, ModifierType::Status, "enfeebled"),
            )],

            ConditionType::Stupefied => vec![ConditionEffect::ApplyModifier(
                ConditionModifierTarget::SpellBased,
                Modifier::new(-val, ModifierType::Status, "stupefied"),
            )],

            ConditionType::Drained => vec![ConditionEffect::ApplyModifier(
                ConditionModifierTarget::Specific("fortitude".into()),
                Modifier::new(-val, ModifierType::Status, "drained"),
            )],

            ConditionType::FlatFooted => vec![
                ConditionEffect::FlatFooted,
                ConditionEffect::ApplyModifier(
                    ConditionModifierTarget::ArmorClass,
                    Modifier::new(-2, ModifierType::Circumstance, "flat-footed"),
                ),
            ],

            ConditionType::Blinded => vec![
                ConditionEffect::SensoryBlock(SensoryChannel::Vision),
                ConditionEffect::FlatFooted,
                ConditionEffect::ApplyModifier(
                    ConditionModifierTarget::ArmorClass,
                    Modifier::new(-2, ModifierType::Circumstance, "flat-footed (blinded)"),
                ),
            ],

            ConditionType::Deafened => {
                vec![ConditionEffect::SensoryBlock(SensoryChannel::Hearing)]
            }

            ConditionType::Prone => vec![
                ConditionEffect::ApplyModifier(
                    ConditionModifierTarget::AttackRolls,
                    Modifier::new(-2, ModifierType::Circumstance, "prone"),
                ),
                ConditionEffect::FlatFooted,
                ConditionEffect::ApplyModifier(
                    ConditionModifierTarget::ArmorClass,
                    Modifier::new(-2, ModifierType::Circumstance, "flat-footed (prone)"),
                ),
            ],

            ConditionType::Slowed => vec![ConditionEffect::ReduceActions(severity.value())],
            ConditionType::Quickened => vec![ConditionEffect::GrantActions(1)],
            ConditionType::Stunned => vec![ConditionEffect::ReduceActions(severity.value())],

            ConditionType::Paralyzed => vec![
                ConditionEffect::Incapacitated,
                ConditionEffect::FlatFooted,
                ConditionEffect::ApplyModifier(
                    ConditionModifierTarget::ArmorClass,
                    Modifier::new(-2, ModifierType::Circumstance, "flat-footed (paralyzed)"),
                ),
            ],

            ConditionType::Unconscious => vec![
                ConditionEffect::Incapacitated,
                ConditionEffect::FlatFooted,
                ConditionEffect::ApplyModifier(
                    ConditionModifierTarget::ArmorClass,
                    Modifier::new(-4, ModifierType::Status, "unconscious"),
                ),
            ],

            ConditionType::Immobilized
            | ConditionType::Restrained
            | ConditionType::Grabbed => vec![
                ConditionEffect::PreventMovement,
                ConditionEffect::FlatFooted,
                ConditionEffect::ApplyModifier(
                    ConditionModifierTarget::ArmorClass,
                    Modifier::new(-2, ModifierType::Circumstance, "flat-footed (immobilized)"),
                ),
            ],

            ConditionType::Fleeing => vec![ConditionEffect::ForcedMovement],

            _ => vec![],
        }
    }

    /// Whether this condition decreases by 1 at end of turn.
    pub fn decreases_at_end_of_turn(condition: ConditionType) -> bool {
        matches!(condition, ConditionType::Frightened)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frightened_produces_status_penalty() {
        let effects =
            ConditionRules::effects(ConditionType::Frightened, ConditionSeverity::Value(2));
        assert_eq!(effects.len(), 1);
        match &effects[0] {
            ConditionEffect::ApplyModifier(target, modifier) => {
                assert_eq!(*target, ConditionModifierTarget::All);
                assert_eq!(modifier.value, -2);
                assert_eq!(modifier.modifier_type, ModifierType::Status);
            }
            _ => panic!("Expected ApplyModifier"),
        }
    }

    #[test]
    fn slowed_reduces_actions() {
        let effects = ConditionRules::effects(ConditionType::Slowed, ConditionSeverity::Value(1));
        assert_eq!(effects.len(), 1);
        matches!(&effects[0], ConditionEffect::ReduceActions(1));
    }

    #[test]
    fn prone_gives_flat_footed_and_attack_penalty() {
        let effects = ConditionRules::effects(ConditionType::Prone, ConditionSeverity::Active);
        assert!(effects.len() >= 2);
    }
}
