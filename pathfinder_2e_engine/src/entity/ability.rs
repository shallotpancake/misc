use bevy::prelude::*;

/// The six ability scores.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AbilityType {
    Strength,
    Dexterity,
    Constitution,
    Intelligence,
    Wisdom,
    Charisma,
}

/// Component: an entity's ability scores.
/// These are just data — the mechanics layer computes modifiers from them.
#[derive(Component, Debug, Clone)]
pub struct AbilityScores {
    pub strength: i32,
    pub dexterity: i32,
    pub constitution: i32,
    pub intelligence: i32,
    pub wisdom: i32,
    pub charisma: i32,
}

impl AbilityScores {
    /// PF2e ability modifier = floor((score - 10) / 2).
    /// Rust truncates toward zero, so we need explicit floor for negatives.
    pub fn modifier(&self, ability: AbilityType) -> i32 {
        let diff = self.score(ability) - 10;
        if diff >= 0 {
            diff / 2
        } else {
            (diff - 1) / 2
        }
    }

    pub fn score(&self, ability: AbilityType) -> i32 {
        match ability {
            AbilityType::Strength => self.strength,
            AbilityType::Dexterity => self.dexterity,
            AbilityType::Constitution => self.constitution,
            AbilityType::Intelligence => self.intelligence,
            AbilityType::Wisdom => self.wisdom,
            AbilityType::Charisma => self.charisma,
        }
    }
}

impl Default for AbilityScores {
    fn default() -> Self {
        Self {
            strength: 10,
            dexterity: 10,
            constitution: 10,
            intelligence: 10,
            wisdom: 10,
            charisma: 10,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ability_modifiers() {
        let scores = AbilityScores {
            strength: 18,
            dexterity: 14,
            constitution: 12,
            intelligence: 10,
            wisdom: 8,
            charisma: 7,
        };
        assert_eq!(scores.modifier(AbilityType::Strength), 4);
        assert_eq!(scores.modifier(AbilityType::Dexterity), 2);
        assert_eq!(scores.modifier(AbilityType::Constitution), 1);
        assert_eq!(scores.modifier(AbilityType::Intelligence), 0);
        assert_eq!(scores.modifier(AbilityType::Wisdom), -1);
        assert_eq!(scores.modifier(AbilityType::Charisma), -2);
    }
}
