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

            ConditionType::Drained => vec![
                ConditionEffect::ApplyModifier(
                    ConditionModifierTarget::Specific("constitution-based".into()),
                    Modifier::new(-val, ModifierType::Status, "drained"),
                ),
                ConditionEffect::ReduceMaxHp,
            ],

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
            ConditionType::Stunned => vec![ConditionEffect::ConsumeActions(severity.value())],

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
                ConditionEffect::ApplyCondition(ConditionType::Blinded),
                ConditionEffect::ApplyCondition(ConditionType::Prone),
                ConditionEffect::FlatFooted,
                ConditionEffect::ApplyModifier(
                    ConditionModifierTarget::ArmorClass,
                    Modifier::new(-4, ModifierType::Status, "unconscious"),
                ),
            ],

            ConditionType::Immobilized => vec![ConditionEffect::PreventMovement],

            ConditionType::Grabbed => vec![
                ConditionEffect::PreventMovement,
                ConditionEffect::FlatFooted,
                ConditionEffect::ApplyModifier(
                    ConditionModifierTarget::ArmorClass,
                    Modifier::new(-2, ModifierType::Circumstance, "flat-footed (grabbed)"),
                ),
            ],

            ConditionType::Restrained => vec![
                ConditionEffect::PreventMovement,
                ConditionEffect::FlatFooted,
                ConditionEffect::ApplyModifier(
                    ConditionModifierTarget::ArmorClass,
                    Modifier::new(-2, ModifierType::Circumstance, "flat-footed (restrained)"),
                ),
            ],

            ConditionType::Fleeing => vec![ConditionEffect::ForcedMovement],

            // Concealed: DC 5 flat check for attacker to target this creature
            ConditionType::Concealed => vec![ConditionEffect::RequireFlatCheck(5)],

            // Confused: random targeting, can't use reactions, must Strike
            ConditionType::Confused => vec![
                ConditionEffect::RandomTargeting,
                ConditionEffect::PreventReactions,
                ConditionEffect::FlatFooted,
                ConditionEffect::ApplyModifier(
                    ConditionModifierTarget::ArmorClass,
                    Modifier::new(-2, ModifierType::Circumstance, "flat-footed (confused)"),
                ),
            ],

            // Dazzled: all creatures are concealed to you (DC 5 flat check)
            ConditionType::Dazzled => vec![ConditionEffect::RequireFlatCheck(5)],

            // Doomed: reduces your maximum dying value
            ConditionType::Doomed => vec![ConditionEffect::ReduceMaxDying(severity.value())],

            // Dying: must attempt recovery checks, incapacitated, unconscious
            ConditionType::Dying => vec![
                ConditionEffect::RecoveryCheckRequired,
                ConditionEffect::Incapacitated,
                ConditionEffect::ApplyCondition(ConditionType::Unconscious),
            ],

            // Encumbered: clumsy 1, -10 ft speed penalty
            ConditionType::Encumbered => vec![
                ConditionEffect::ReduceSpeed(10),
                ConditionEffect::ApplyCondition(ConditionType::Clumsy),
            ],

            // Fascinated: -2 status penalty to Perception and skill checks,
            // can't use concentrate actions on other subjects
            ConditionType::Fascinated => vec![
                ConditionEffect::ApplyModifier(
                    ConditionModifierTarget::Perception,
                    Modifier::new(-2, ModifierType::Status, "fascinated"),
                ),
                ConditionEffect::ApplyModifier(
                    ConditionModifierTarget::SkillChecks,
                    Modifier::new(-2, ModifierType::Status, "fascinated"),
                ),
                ConditionEffect::RestrictConcentrate,
            ],

            // Fatigued: -1 status penalty to AC and saving throws
            ConditionType::Fatigued => vec![
                ConditionEffect::ApplyModifier(
                    ConditionModifierTarget::ArmorClass,
                    Modifier::new(-1, ModifierType::Status, "fatigued"),
                ),
                ConditionEffect::ApplyModifier(
                    ConditionModifierTarget::SavingThrows,
                    Modifier::new(-1, ModifierType::Status, "fatigued"),
                ),
            ],

            // Hidden: DC 11 flat check to target, +2 circumstance to AC
            ConditionType::Hidden => vec![
                ConditionEffect::RequireFlatCheck(11),
                ConditionEffect::ApplyModifier(
                    ConditionModifierTarget::ArmorClass,
                    Modifier::new(2, ModifierType::Circumstance, "hidden"),
                ),
            ],

            // Invisible: DC 11 flat check to target, +2 circumstance to AC
            ConditionType::Invisible => vec![
                ConditionEffect::RequireFlatCheck(11),
                ConditionEffect::ApplyModifier(
                    ConditionModifierTarget::ArmorClass,
                    Modifier::new(2, ModifierType::Circumstance, "invisible"),
                ),
            ],

            // Petrified: paralyzed effects + resistance to all damage
            ConditionType::Petrified => vec![
                ConditionEffect::Incapacitated,
                ConditionEffect::FlatFooted,
                ConditionEffect::ApplyModifier(
                    ConditionModifierTarget::ArmorClass,
                    Modifier::new(-2, ModifierType::Circumstance, "flat-footed (petrified)"),
                ),
                ConditionEffect::DamageResistance,
            ],

            // Wounded: increases dying value when gaining the dying condition
            ConditionType::Wounded => {
                vec![ConditionEffect::IncreaseDyingValue(severity.value())]
            }
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
        assert!(
            matches!(&effects[0], ConditionEffect::ReduceActions(1)),
            "Slowed should use ReduceActions, got {:?}",
            effects[0]
        );
    }

    #[test]
    fn stunned_consumes_actions() {
        let effects =
            ConditionRules::effects(ConditionType::Stunned, ConditionSeverity::Value(2));
        assert_eq!(effects.len(), 1);
        assert!(
            matches!(&effects[0], ConditionEffect::ConsumeActions(2)),
            "Stunned should use ConsumeActions, got {:?}",
            effects[0]
        );
    }

    #[test]
    fn immobilized_does_not_produce_flat_footed() {
        let effects =
            ConditionRules::effects(ConditionType::Immobilized, ConditionSeverity::Active);
        assert!(
            !effects
                .iter()
                .any(|e| matches!(e, ConditionEffect::FlatFooted)),
            "Immobilized should NOT produce FlatFooted"
        );
    }

    #[test]
    fn grabbed_produces_flat_footed() {
        let effects = ConditionRules::effects(ConditionType::Grabbed, ConditionSeverity::Active);
        assert!(
            effects
                .iter()
                .any(|e| matches!(e, ConditionEffect::FlatFooted)),
            "Grabbed should produce FlatFooted"
        );
    }

    #[test]
    fn unconscious_cascades_to_blinded_and_prone() {
        let effects =
            ConditionRules::effects(ConditionType::Unconscious, ConditionSeverity::Active);
        assert!(
            effects.iter().any(
                |e| matches!(e, ConditionEffect::ApplyCondition(ConditionType::Blinded))
            ),
            "Unconscious should cascade to Blinded via ApplyCondition"
        );
        assert!(
            effects
                .iter()
                .any(|e| matches!(e, ConditionEffect::ApplyCondition(ConditionType::Prone))),
            "Unconscious should cascade to Prone via ApplyCondition"
        );
    }

    #[test]
    fn drained_includes_reduce_max_hp() {
        let effects =
            ConditionRules::effects(ConditionType::Drained, ConditionSeverity::Value(1));
        assert!(
            effects
                .iter()
                .any(|e| matches!(e, ConditionEffect::ReduceMaxHp)),
            "Drained should include ReduceMaxHp"
        );
    }

    #[test]
    fn prone_gives_flat_footed_and_attack_penalty() {
        let effects = ConditionRules::effects(ConditionType::Prone, ConditionSeverity::Active);
        assert!(effects.len() >= 2);
    }

    #[test]
    fn dying_produces_recovery_check_and_incapacitation() {
        let effects = ConditionRules::effects(ConditionType::Dying, ConditionSeverity::Value(1));
        assert!(
            effects
                .iter()
                .any(|e| matches!(e, ConditionEffect::RecoveryCheckRequired)),
            "Dying should require recovery checks"
        );
        assert!(
            effects
                .iter()
                .any(|e| matches!(e, ConditionEffect::Incapacitated)),
            "Dying should produce Incapacitated"
        );
        assert!(
            effects.iter().any(
                |e| matches!(e, ConditionEffect::ApplyCondition(ConditionType::Unconscious))
            ),
            "Dying should apply Unconscious condition"
        );
    }

    #[test]
    fn concealed_requires_flat_check_dc5() {
        let effects =
            ConditionRules::effects(ConditionType::Concealed, ConditionSeverity::Active);
        assert!(
            effects
                .iter()
                .any(|e| matches!(e, ConditionEffect::RequireFlatCheck(5))),
            "Concealed should require DC 5 flat check"
        );
    }

    #[test]
    fn fatigued_penalizes_ac_and_saves() {
        let effects =
            ConditionRules::effects(ConditionType::Fatigued, ConditionSeverity::Active);
        assert_eq!(effects.len(), 2);

        let has_ac_penalty = effects.iter().any(|e| match e {
            ConditionEffect::ApplyModifier(ConditionModifierTarget::ArmorClass, m) => {
                m.value == -1 && m.modifier_type == ModifierType::Status
            }
            _ => false,
        });
        assert!(has_ac_penalty, "Fatigued should give -1 status penalty to AC");

        let has_save_penalty = effects.iter().any(|e| match e {
            ConditionEffect::ApplyModifier(ConditionModifierTarget::SavingThrows, m) => {
                m.value == -1 && m.modifier_type == ModifierType::Status
            }
            _ => false,
        });
        assert!(
            has_save_penalty,
            "Fatigued should give -1 status penalty to saving throws"
        );
    }

    #[test]
    fn encumbered_reduces_speed_and_applies_clumsy() {
        let effects =
            ConditionRules::effects(ConditionType::Encumbered, ConditionSeverity::Active);
        assert!(
            effects
                .iter()
                .any(|e| matches!(e, ConditionEffect::ReduceSpeed(10))),
            "Encumbered should reduce speed by 10"
        );
        assert!(
            effects.iter().any(
                |e| matches!(e, ConditionEffect::ApplyCondition(ConditionType::Clumsy))
            ),
            "Encumbered should apply Clumsy condition"
        );
    }

    #[test]
    fn invisible_requires_flat_check_dc11_and_ac_bonus() {
        let effects =
            ConditionRules::effects(ConditionType::Invisible, ConditionSeverity::Active);
        assert!(
            effects
                .iter()
                .any(|e| matches!(e, ConditionEffect::RequireFlatCheck(11))),
            "Invisible should require DC 11 flat check"
        );

        let has_ac_bonus = effects.iter().any(|e| match e {
            ConditionEffect::ApplyModifier(ConditionModifierTarget::ArmorClass, m) => {
                m.value == 2 && m.modifier_type == ModifierType::Circumstance
            }
            _ => false,
        });
        assert!(
            has_ac_bonus,
            "Invisible should give +2 circumstance bonus to AC"
        );
    }

    #[test]
    fn wounded_increases_dying_value() {
        let effects =
            ConditionRules::effects(ConditionType::Wounded, ConditionSeverity::Value(2));
        assert!(
            effects
                .iter()
                .any(|e| matches!(e, ConditionEffect::IncreaseDyingValue(2))),
            "Wounded 2 should increase dying value by 2"
        );
    }

    #[test]
    fn doomed_reduces_max_dying() {
        let effects =
            ConditionRules::effects(ConditionType::Doomed, ConditionSeverity::Value(1));
        assert!(
            effects
                .iter()
                .any(|e| matches!(e, ConditionEffect::ReduceMaxDying(1))),
            "Doomed 1 should reduce max dying by 1"
        );

        let effects_2 =
            ConditionRules::effects(ConditionType::Doomed, ConditionSeverity::Value(3));
        assert!(
            effects_2
                .iter()
                .any(|e| matches!(e, ConditionEffect::ReduceMaxDying(3))),
            "Doomed 3 should reduce max dying by 3"
        );
    }

    #[test]
    fn fascinated_penalizes_perception_and_skills() {
        let effects =
            ConditionRules::effects(ConditionType::Fascinated, ConditionSeverity::Active);

        let has_perception_penalty = effects.iter().any(|e| match e {
            ConditionEffect::ApplyModifier(ConditionModifierTarget::Perception, m) => {
                m.value == -2 && m.modifier_type == ModifierType::Status
            }
            _ => false,
        });
        assert!(
            has_perception_penalty,
            "Fascinated should give -2 status penalty to Perception"
        );

        let has_skill_penalty = effects.iter().any(|e| match e {
            ConditionEffect::ApplyModifier(ConditionModifierTarget::SkillChecks, m) => {
                m.value == -2 && m.modifier_type == ModifierType::Status
            }
            _ => false,
        });
        assert!(
            has_skill_penalty,
            "Fascinated should give -2 status penalty to skill checks"
        );

        assert!(
            effects
                .iter()
                .any(|e| matches!(e, ConditionEffect::RestrictConcentrate)),
            "Fascinated should restrict concentrate actions"
        );
    }

    #[test]
    fn confused_prevents_reactions_and_has_random_targeting() {
        let effects =
            ConditionRules::effects(ConditionType::Confused, ConditionSeverity::Active);
        assert!(
            effects
                .iter()
                .any(|e| matches!(e, ConditionEffect::RandomTargeting)),
            "Confused should require random targeting"
        );
        assert!(
            effects
                .iter()
                .any(|e| matches!(e, ConditionEffect::PreventReactions)),
            "Confused should prevent reactions"
        );
        assert!(
            effects
                .iter()
                .any(|e| matches!(e, ConditionEffect::FlatFooted)),
            "Confused should make you flat-footed"
        );
    }

    #[test]
    fn petrified_has_paralysis_effects_and_damage_resistance() {
        let effects =
            ConditionRules::effects(ConditionType::Petrified, ConditionSeverity::Active);
        assert!(
            effects
                .iter()
                .any(|e| matches!(e, ConditionEffect::Incapacitated)),
            "Petrified should produce Incapacitated"
        );
        assert!(
            effects
                .iter()
                .any(|e| matches!(e, ConditionEffect::FlatFooted)),
            "Petrified should produce FlatFooted"
        );
        assert!(
            effects
                .iter()
                .any(|e| matches!(e, ConditionEffect::DamageResistance)),
            "Petrified should produce DamageResistance"
        );
    }

    #[test]
    fn hidden_requires_flat_check_dc11_and_ac_bonus() {
        let effects =
            ConditionRules::effects(ConditionType::Hidden, ConditionSeverity::Active);
        assert!(
            effects
                .iter()
                .any(|e| matches!(e, ConditionEffect::RequireFlatCheck(11))),
            "Hidden should require DC 11 flat check"
        );

        let has_ac_bonus = effects.iter().any(|e| match e {
            ConditionEffect::ApplyModifier(ConditionModifierTarget::ArmorClass, m) => {
                m.value == 2 && m.modifier_type == ModifierType::Circumstance
            }
            _ => false,
        });
        assert!(
            has_ac_bonus,
            "Hidden should give +2 circumstance bonus to AC"
        );
    }

    #[test]
    fn dazzled_requires_flat_check_dc5() {
        let effects =
            ConditionRules::effects(ConditionType::Dazzled, ConditionSeverity::Active);
        assert!(
            effects
                .iter()
                .any(|e| matches!(e, ConditionEffect::RequireFlatCheck(5))),
            "Dazzled should require DC 5 flat check"
        );
    }

    #[test]
    fn no_conditions_return_empty_effects() {
        // Verify every condition type now returns non-empty effects
        let all_conditions = [
            ConditionType::Blinded,
            ConditionType::Clumsy,
            ConditionType::Concealed,
            ConditionType::Confused,
            ConditionType::Dazzled,
            ConditionType::Deafened,
            ConditionType::Doomed,
            ConditionType::Drained,
            ConditionType::Dying,
            ConditionType::Encumbered,
            ConditionType::Enfeebled,
            ConditionType::Fascinated,
            ConditionType::Fatigued,
            ConditionType::FlatFooted,
            ConditionType::Fleeing,
            ConditionType::Frightened,
            ConditionType::Grabbed,
            ConditionType::Hidden,
            ConditionType::Immobilized,
            ConditionType::Invisible,
            ConditionType::Paralyzed,
            ConditionType::Petrified,
            ConditionType::Prone,
            ConditionType::Quickened,
            ConditionType::Restrained,
            ConditionType::Sickened,
            ConditionType::Slowed,
            ConditionType::Stunned,
            ConditionType::Stupefied,
            ConditionType::Unconscious,
            ConditionType::Wounded,
        ];

        for condition in &all_conditions {
            let effects = ConditionRules::effects(*condition, ConditionSeverity::Value(1));
            assert!(
                !effects.is_empty(),
                "{:?} should produce at least one effect",
                condition
            );
        }
    }
}
