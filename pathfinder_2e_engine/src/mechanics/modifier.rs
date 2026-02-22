/// Modifier type determines stacking rules.
/// In PF2e, only untyped bonuses/penalties stack freely. Typed bonuses
/// keep only the highest; typed penalties keep only the lowest (worst).
/// This is a law of the mechanics universe — no entity can override it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModifierType {
    Untyped,
    Ability,
    Proficiency,
    Item,
    Status,
    Circumstance,
}

/// A single modifier: value + type + source label.
/// Positive = bonus, negative = penalty.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Modifier {
    pub value: i32,
    pub modifier_type: ModifierType,
    pub source: String,
}

impl Modifier {
    pub fn new(value: i32, modifier_type: ModifierType, source: impl Into<String>) -> Self {
        Self {
            value,
            modifier_type,
            source: source.into(),
        }
    }

    pub fn is_bonus(&self) -> bool {
        self.value > 0
    }

    pub fn is_penalty(&self) -> bool {
        self.value < 0
    }
}

/// A collection of modifiers resolved according to PF2e stacking rules.
#[derive(Debug, Clone, Default)]
pub struct ModifierStack {
    modifiers: Vec<Modifier>,
}

impl ModifierStack {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, modifier: Modifier) {
        self.modifiers.push(modifier);
    }

    pub fn with(mut self, modifier: Modifier) -> Self {
        self.add(modifier);
        self
    }

    /// Resolve the stack into a total according to PF2e rules:
    /// - Untyped: all stack
    /// - Typed bonuses: highest of each type
    /// - Typed penalties: lowest (most negative) of each type
    pub fn resolve(&self) -> ResolvedModifiers {
        let mut total = 0i32;
        let mut applied = Vec::new();
        let mut suppressed = Vec::new();

        let typed_categories = [
            ModifierType::Ability,
            ModifierType::Proficiency,
            ModifierType::Item,
            ModifierType::Status,
            ModifierType::Circumstance,
        ];

        // Untyped always stack
        for m in self
            .modifiers
            .iter()
            .filter(|m| m.modifier_type == ModifierType::Untyped)
        {
            total += m.value;
            applied.push(m.clone());
        }

        // For each typed category, best bonus + worst penalty
        for mod_type in &typed_categories {
            let typed: Vec<&Modifier> = self
                .modifiers
                .iter()
                .filter(|m| m.modifier_type == *mod_type)
                .collect();

            if typed.is_empty() {
                continue;
            }

            let bonuses: Vec<&&Modifier> = typed.iter().filter(|m| m.is_bonus()).collect();
            if !bonuses.is_empty() {
                let best_idx = bonuses
                    .iter()
                    .enumerate()
                    .max_by_key(|(_, m)| m.value)
                    .map(|(i, _)| i)
                    .unwrap();
                for (i, &b) in bonuses.iter().enumerate() {
                    if i == best_idx {
                        total += b.value;
                        applied.push((*b).clone());
                    } else {
                        suppressed.push((*b).clone());
                    }
                }
            }

            let penalties: Vec<&&Modifier> = typed.iter().filter(|m| m.is_penalty()).collect();
            if !penalties.is_empty() {
                let worst_idx = penalties
                    .iter()
                    .enumerate()
                    .min_by_key(|(_, m)| m.value)
                    .map(|(i, _)| i)
                    .unwrap();
                for (i, &p) in penalties.iter().enumerate() {
                    if i == worst_idx {
                        total += p.value;
                        applied.push((*p).clone());
                    } else {
                        suppressed.push((*p).clone());
                    }
                }
            }
        }

        ResolvedModifiers {
            total,
            applied,
            suppressed,
        }
    }
}

/// Result of resolving a modifier stack.
#[derive(Debug, Clone)]
pub struct ResolvedModifiers {
    pub total: i32,
    pub applied: Vec<Modifier>,
    pub suppressed: Vec<Modifier>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn untyped_modifiers_stack() {
        let stack = ModifierStack::new()
            .with(Modifier::new(2, ModifierType::Untyped, "aid"))
            .with(Modifier::new(1, ModifierType::Untyped, "heroism"));
        let resolved = stack.resolve();
        assert_eq!(resolved.total, 3);
        assert_eq!(resolved.applied.len(), 2);
    }

    #[test]
    fn typed_bonuses_take_highest() {
        let stack = ModifierStack::new()
            .with(Modifier::new(2, ModifierType::Status, "bless"))
            .with(Modifier::new(3, ModifierType::Status, "heroism"));
        let resolved = stack.resolve();
        assert_eq!(resolved.total, 3);
        assert_eq!(resolved.applied.len(), 1);
        assert_eq!(resolved.suppressed.len(), 1);
        assert_eq!(resolved.applied[0].source, "heroism");
    }

    #[test]
    fn typed_penalties_take_worst() {
        let stack = ModifierStack::new()
            .with(Modifier::new(-1, ModifierType::Status, "frightened"))
            .with(Modifier::new(-3, ModifierType::Status, "sickened"));
        let resolved = stack.resolve();
        assert_eq!(resolved.total, -3);
        assert_eq!(resolved.applied.len(), 1);
        assert_eq!(resolved.applied[0].source, "sickened");
    }

    #[test]
    fn mixed_types_stack_independently() {
        let stack = ModifierStack::new()
            .with(Modifier::new(2, ModifierType::Status, "bless"))
            .with(Modifier::new(1, ModifierType::Circumstance, "flanking"))
            .with(Modifier::new(1, ModifierType::Item, "handwraps"))
            .with(Modifier::new(-2, ModifierType::Status, "frightened"));
        let resolved = stack.resolve();
        // +2 status, -2 status, +1 circ, +1 item = 2
        assert_eq!(resolved.total, 2);
    }
}
