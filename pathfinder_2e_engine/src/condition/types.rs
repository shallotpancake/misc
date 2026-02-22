use crate::mechanics::modifier::Modifier;

/// All PF2e conditions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConditionType {
    Blinded,
    Clumsy,
    Concealed,
    Confused,
    Dazzled,
    Deafened,
    Doomed,
    Drained,
    Dying,
    Encumbered,
    Enfeebled,
    Fascinated,
    Fatigued,
    FlatFooted,
    Fleeing,
    Frightened,
    Grabbed,
    Hidden,
    Immobilized,
    Invisible,
    Paralyzed,
    Petrified,
    Prone,
    Quickened,
    Restrained,
    Sickened,
    Slowed,
    Stunned,
    Stupefied,
    Unconscious,
    Wounded,
}

/// Severity of a condition: binary or valued.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionSeverity {
    Active,
    Value(u32),
}

impl ConditionSeverity {
    pub fn value(&self) -> u32 {
        match self {
            Self::Active => 1,
            Self::Value(v) => *v,
        }
    }
}

/// What a condition's modifier applies to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConditionModifierTarget {
    All,
    ArmorClass,
    AttackRolls,
    SavingThrows,
    SkillChecks,
    Perception,
    StrengthBased,
    DexterityBased,
    SpellBased,
    Specific(String),
}

/// Effects produced by conditions.
#[derive(Debug, Clone)]
pub enum ConditionEffect {
    ApplyModifier(ConditionModifierTarget, Modifier),
    ReduceActions(u32),
    GrantActions(u32),
    /// Like ReduceActions but the condition value decreases by the number of actions lost.
    ConsumeActions(u32),
    Incapacitated,
    ForcedMovement,
    PreventMovement,
    FlatFooted,
    SensoryBlock(SensoryChannel),
    ReduceMaxHp,
    ApplyCondition(ConditionType),
    /// Attacker must pass a flat check of this DC to target the creature.
    RequireFlatCheck(u32),
    /// Reduce movement speed by this many feet.
    ReduceSpeed(u32),
    /// Must select targets randomly (e.g., Confused).
    RandomTargeting,
    /// Cannot use reactions.
    PreventReactions,
    /// Reduce the maximum dying value by this amount (e.g., Doomed).
    ReduceMaxDying(u32),
    /// Must make recovery flat checks each round (e.g., Dying).
    RecoveryCheckRequired,
    /// Concentrate actions are restricted to specific targets (e.g., Fascinated).
    RestrictConcentrate,
    /// Increases the dying value gained when gaining the dying condition (e.g., Wounded).
    IncreaseDyingValue(u32),
    /// Resistance to all damage (e.g., Petrified).
    DamageResistance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensoryChannel {
    Vision,
    Hearing,
    Scent,
    Tremorsense,
}
