use bevy::prelude::*;
use std::collections::HashMap;

/// PF2e proficiency ranks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Proficiency {
    Untrained,
    Trained,
    Expert,
    Master,
    Legendary,
}

impl Proficiency {
    /// The proficiency bonus = rank value + level (if trained or better).
    /// Untrained adds 0. This is a rule of the mechanics universe.
    pub fn bonus(&self, level: i32) -> i32 {
        match self {
            Proficiency::Untrained => 0,
            Proficiency::Trained => level + 2,
            Proficiency::Expert => level + 4,
            Proficiency::Master => level + 6,
            Proficiency::Legendary => level + 8,
        }
    }
}

/// Component: an entity's skill proficiencies.
#[derive(Component, Debug, Clone, Default)]
pub struct SkillProficiencies {
    pub skills: HashMap<String, Proficiency>,
}

impl SkillProficiencies {
    pub fn get(&self, skill: &str) -> Proficiency {
        self.skills
            .get(skill)
            .copied()
            .unwrap_or(Proficiency::Untrained)
    }

    pub fn set(&mut self, skill: impl Into<String>, rank: Proficiency) {
        self.skills.insert(skill.into(), rank);
    }
}

/// Component: saving throw proficiencies.
#[derive(Component, Debug, Clone)]
pub struct SavingThrows {
    pub fortitude: Proficiency,
    pub reflex: Proficiency,
    pub will: Proficiency,
}

impl Default for SavingThrows {
    fn default() -> Self {
        Self {
            fortitude: Proficiency::Untrained,
            reflex: Proficiency::Untrained,
            will: Proficiency::Untrained,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proficiency_bonuses() {
        let level = 5;
        assert_eq!(Proficiency::Untrained.bonus(level), 0);
        assert_eq!(Proficiency::Trained.bonus(level), 7);
        assert_eq!(Proficiency::Expert.bonus(level), 9);
        assert_eq!(Proficiency::Master.bonus(level), 11);
        assert_eq!(Proficiency::Legendary.bonus(level), 13);
    }
}
