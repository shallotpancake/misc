use bevy::prelude::*;

use super::modifier::ModifierStack;

/// The four degrees of success — a fundamental law of the mechanics universe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DegreeOfSuccess {
    CriticalFailure,
    Failure,
    Success,
    CriticalSuccess,
}

impl DegreeOfSuccess {
    pub fn improve(self) -> Self {
        match self {
            Self::CriticalFailure => Self::Failure,
            Self::Failure => Self::Success,
            Self::Success => Self::CriticalSuccess,
            Self::CriticalSuccess => Self::CriticalSuccess,
        }
    }

    pub fn worsen(self) -> Self {
        match self {
            Self::CriticalFailure => Self::CriticalFailure,
            Self::Failure => Self::CriticalFailure,
            Self::Success => Self::Failure,
            Self::CriticalSuccess => Self::Success,
        }
    }

    pub fn shift(self, steps: i32) -> Self {
        let mut result = self;
        if steps > 0 {
            for _ in 0..steps {
                result = result.improve();
            }
        } else {
            for _ in 0..steps.abs() {
                result = result.worsen();
            }
        }
        result
    }
}

/// Pure input for resolving a check. No entity knowledge — just numbers.
#[derive(Debug, Clone)]
pub struct CheckContext {
    pub natural_roll: i32,
    pub modifiers: ModifierStack,
    pub dc: i32,
    pub degree_shifts: i32,
}

/// Full result of resolving a check.
#[derive(Debug, Clone)]
pub struct CheckResult {
    pub natural_roll: i32,
    pub total: i32,
    pub dc: i32,
    pub base_degree: DegreeOfSuccess,
    pub final_degree: DegreeOfSuccess,
}

/// Resolve a check according to PF2e rules. Pure function — maps inputs to outputs.
///
/// 1. total = natural_roll + resolved modifiers
/// 2. Compare total to DC for base degree
/// 3. Natural 20 improves, natural 1 worsens
/// 4. Apply additional degree shifts
pub fn resolve_check(ctx: &CheckContext) -> CheckResult {
    let resolved = ctx.modifiers.resolve();
    let total = ctx.natural_roll + resolved.total;

    let base_degree = if total >= ctx.dc + 10 {
        DegreeOfSuccess::CriticalSuccess
    } else if total >= ctx.dc {
        DegreeOfSuccess::Success
    } else if total <= ctx.dc - 10 {
        DegreeOfSuccess::CriticalFailure
    } else {
        DegreeOfSuccess::Failure
    };

    let nat_shifted = if ctx.natural_roll == 20 {
        base_degree.improve()
    } else if ctx.natural_roll == 1 {
        base_degree.worsen()
    } else {
        base_degree
    };

    let final_degree = nat_shifted.shift(ctx.degree_shifts);

    CheckResult {
        natural_roll: ctx.natural_roll,
        total,
        dc: ctx.dc,
        base_degree,
        final_degree,
    }
}

// --- Bevy integration: messages + system ---

/// Message sent to request a check be resolved by the mechanics layer.
#[derive(Message, Debug, Clone)]
pub struct CheckRequestedEvent {
    /// The entity performing the check (for routing the result back).
    pub actor: Entity,
    /// Optional target entity.
    pub target: Option<Entity>,
    /// Human-readable label ("Perception check", "Strike vs AC", etc.)
    pub label: String,
    pub context: CheckContext,
}

/// Message sent when a check has been resolved.
#[derive(Message, Debug, Clone)]
pub struct CheckResolvedEvent {
    pub actor: Entity,
    pub target: Option<Entity>,
    pub label: String,
    pub result: CheckResult,
}

/// Bevy system: listens for check requests, resolves them, emits results.
/// This IS the law of gravity — it operates uniformly on every check request.
pub fn resolve_checks_system(
    mut requests: MessageReader<CheckRequestedEvent>,
    mut results: MessageWriter<CheckResolvedEvent>,
) {
    for request in requests.read() {
        let result = resolve_check(&request.context);
        results.write(CheckResolvedEvent {
            actor: request.actor,
            target: request.target,
            label: request.label.clone(),
            result,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mechanics::modifier::{Modifier, ModifierType};

    fn simple_check(natural_roll: i32, modifier: i32, dc: i32) -> CheckResult {
        let modifiers =
            ModifierStack::new().with(Modifier::new(modifier, ModifierType::Untyped, "total"));
        resolve_check(&CheckContext {
            natural_roll,
            modifiers,
            dc,
            degree_shifts: 0,
        })
    }

    #[test]
    fn basic_success() {
        let result = simple_check(14, 5, 18);
        assert_eq!(result.total, 19);
        assert_eq!(result.final_degree, DegreeOfSuccess::Success);
    }

    #[test]
    fn basic_failure() {
        // 10 + 3 = 13 vs DC 20 → Failure (not critical: 13 > 10)
        let result = simple_check(10, 3, 20);
        assert_eq!(result.total, 13);
        assert_eq!(result.final_degree, DegreeOfSuccess::Failure);
    }

    #[test]
    fn critical_success_by_ten() {
        let result = simple_check(15, 10, 15);
        assert_eq!(result.total, 25);
        assert_eq!(result.final_degree, DegreeOfSuccess::CriticalSuccess);
    }

    #[test]
    fn critical_failure_by_ten() {
        let result = simple_check(2, 0, 20);
        assert_eq!(result.total, 2);
        assert_eq!(result.final_degree, DegreeOfSuccess::CriticalFailure);
    }

    #[test]
    fn natural_20_improves_degree() {
        let result = simple_check(20, 5, 25);
        assert_eq!(result.base_degree, DegreeOfSuccess::Success);
        assert_eq!(result.final_degree, DegreeOfSuccess::CriticalSuccess);
    }

    #[test]
    fn natural_1_worsens_degree() {
        let result = simple_check(1, 10, 10);
        assert_eq!(result.base_degree, DegreeOfSuccess::Success);
        assert_eq!(result.final_degree, DegreeOfSuccess::Failure);
    }

    #[test]
    fn degree_shifts_apply() {
        let modifiers =
            ModifierStack::new().with(Modifier::new(5, ModifierType::Untyped, "total"));
        let result = resolve_check(&CheckContext {
            natural_roll: 10,
            modifiers,
            dc: 15,
            degree_shifts: 1,
        });
        assert_eq!(result.final_degree, DegreeOfSuccess::CriticalSuccess);
    }
}
