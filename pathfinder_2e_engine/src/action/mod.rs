//! # Action Layer (Layer 3)
//!
//! The three-action economy, MAP tracking, and action trait restrictions.
//! These are laws of the field — every entity is subject to them equally.

use bevy::prelude::*;

use crate::mechanics::traits::{GameTrait, TraitCategory};

pub struct ActionPlugin;

impl Plugin for ActionPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ActionPerformedEvent>();
    }
}

/// The cost of an action in PF2e's three-action economy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActionCost {
    Free,
    Reaction,
    Actions(u32),
}

impl ActionCost {
    pub fn actions_required(&self) -> u32 {
        match self {
            Self::Free | Self::Reaction => 0,
            Self::Actions(n) => *n,
        }
    }

    pub fn is_reaction(&self) -> bool {
        matches!(self, Self::Reaction)
    }
}

/// MAP state — a law of the universe: successive attacks suffer penalties.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MultipleAttackPenalty {
    pub attacks_made: u32,
    pub agile: bool,
}

impl MultipleAttackPenalty {
    pub fn new() -> Self {
        Self {
            attacks_made: 0,
            agile: false,
        }
    }

    pub fn current_penalty(&self) -> i32 {
        match self.attacks_made {
            0 => 0,
            1 => {
                if self.agile {
                    -4
                } else {
                    -5
                }
            }
            _ => {
                if self.agile {
                    -8
                } else {
                    -10
                }
            }
        }
    }

    pub fn record_attack(&mut self) {
        self.attacks_made += 1;
    }
}

impl Default for MultipleAttackPenalty {
    fn default() -> Self {
        Self::new()
    }
}

/// Component: tracks available actions for an entity's current turn.
#[derive(Component, Debug, Clone)]
pub struct ActionPool {
    pub base_actions: u32,
    pub actions_lost: u32,
    pub actions_gained: u32,
    pub actions_spent: u32,
    pub reaction_used: bool,
    pub map: MultipleAttackPenalty,
    pub flourish_used: bool,
    pub non_open_action_taken: bool,
}

impl ActionPool {
    pub fn new_turn() -> Self {
        Self {
            base_actions: 3,
            actions_lost: 0,
            actions_gained: 0,
            actions_spent: 0,
            reaction_used: false,
            map: MultipleAttackPenalty::new(),
            flourish_used: false,
            non_open_action_taken: false,
        }
    }

    pub fn total_available(&self) -> u32 {
        (self.base_actions + self.actions_gained).saturating_sub(self.actions_lost)
    }

    pub fn remaining(&self) -> u32 {
        self.total_available().saturating_sub(self.actions_spent)
    }

    pub fn can_perform(&self, cost: ActionCost, traits: &[GameTrait]) -> ActionFeasibility {
        if cost.is_reaction() && self.reaction_used {
            return ActionFeasibility::Blocked("Reaction already used this round".into());
        }

        if cost.actions_required() > self.remaining() {
            return ActionFeasibility::Blocked(format!(
                "Requires {} actions, only {} remaining",
                cost.actions_required(),
                self.remaining()
            ));
        }

        for trait_ in traits {
            match trait_.category {
                TraitCategory::Flourish if self.flourish_used => {
                    return ActionFeasibility::Blocked(
                        "Already used a flourish action this turn".into(),
                    );
                }
                TraitCategory::Open if self.non_open_action_taken => {
                    return ActionFeasibility::Blocked(
                        "Open actions must be used before other actions".into(),
                    );
                }
                _ => {}
            }
        }

        ActionFeasibility::Available
    }

    pub fn spend(&mut self, cost: ActionCost, traits: &[GameTrait]) {
        match cost {
            ActionCost::Free => {}
            ActionCost::Reaction => {
                self.reaction_used = true;
            }
            ActionCost::Actions(n) => {
                self.actions_spent += n;
                self.non_open_action_taken = true;
            }
        }

        for trait_ in traits {
            match trait_.category {
                TraitCategory::Attack => {
                    self.map.record_attack();
                }
                TraitCategory::Flourish => {
                    self.flourish_used = true;
                }
                _ => {}
            }
        }
    }

    pub fn new_round(&mut self) {
        self.reaction_used = false;
    }
}

/// Result of checking action feasibility.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionFeasibility {
    Available,
    Blocked(String),
}

impl ActionFeasibility {
    pub fn is_available(&self) -> bool {
        matches!(self, Self::Available)
    }
}

/// Message: an action was performed.
#[derive(Message, Debug, Clone)]
pub struct ActionPerformedEvent {
    pub actor: Entity,
    pub action_name: String,
    pub cost: ActionCost,
    pub traits: Vec<GameTrait>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn three_action_economy() {
        let mut pool = ActionPool::new_turn();
        assert_eq!(pool.remaining(), 3);

        pool.spend(ActionCost::Actions(1), &[]);
        assert_eq!(pool.remaining(), 2);

        pool.spend(ActionCost::Actions(2), &[]);
        assert_eq!(pool.remaining(), 0);
    }

    #[test]
    fn slowed_reduces_actions() {
        let mut pool = ActionPool::new_turn();
        pool.actions_lost = 1;
        assert_eq!(pool.total_available(), 2);
    }

    #[test]
    fn map_progression() {
        let mut map = MultipleAttackPenalty::new();
        assert_eq!(map.current_penalty(), 0);

        map.record_attack();
        assert_eq!(map.current_penalty(), -5);

        map.record_attack();
        assert_eq!(map.current_penalty(), -10);
    }

    #[test]
    fn agile_map() {
        let mut map = MultipleAttackPenalty {
            attacks_made: 0,
            agile: true,
        };
        map.record_attack();
        assert_eq!(map.current_penalty(), -4);
        map.record_attack();
        assert_eq!(map.current_penalty(), -8);
    }

    #[test]
    fn reaction_tracking() {
        let mut pool = ActionPool::new_turn();
        assert!(pool
            .can_perform(ActionCost::Reaction, &[])
            .is_available());

        pool.spend(ActionCost::Reaction, &[]);
        assert!(!pool
            .can_perform(ActionCost::Reaction, &[])
            .is_available());

        pool.new_round();
        assert!(pool
            .can_perform(ActionCost::Reaction, &[])
            .is_available());
    }

    #[test]
    fn flourish_restriction() {
        let mut pool = ActionPool::new_turn();
        let flourish = vec![GameTrait::flourish()];

        assert!(pool
            .can_perform(ActionCost::Actions(1), &flourish)
            .is_available());
        pool.spend(ActionCost::Actions(1), &flourish);
        assert!(!pool
            .can_perform(ActionCost::Actions(1), &flourish)
            .is_available());
    }
}
