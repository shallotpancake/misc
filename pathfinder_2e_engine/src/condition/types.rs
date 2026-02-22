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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensoryChannel {
    Vision,
    Hearing,
    Scent,
    Tremorsense,
}
