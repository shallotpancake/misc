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
}

impl MultipleAttackPenalty {
    pub fn new() -> Self {
        Self {
            attacks_made: 0,
        }
    }

    pub fn current_penalty(&self, agile: bool) -> i32 {
        match self.attacks_made {
            0 => 0,
            1 => {
                if agile {
                    -4
                } else {
                    -5
                }
            }
            _ => {
                if agile {
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
                TraitCategory::Press if self.map.attacks_made == 0 => {
                    return ActionFeasibility::Blocked(
                        "Press actions require a prior attack this turn".into(),
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
                if !traits.iter().any(|t| t.category == TraitCategory::Open) {
                    self.non_open_action_taken = true;
                }
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

    pub fn reset_for_new_turn(&mut self) {
        self.actions_spent = 0;
        self.actions_lost = 0;
        self.actions_gained = 0;
        self.map = MultipleAttackPenalty::new();
        self.flourish_used = false;
        self.non_open_action_taken = false;
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
        assert_eq!(map.current_penalty(false), 0);

        map.record_attack();
        assert_eq!(map.current_penalty(false), -5);

        map.record_attack();
        assert_eq!(map.current_penalty(false), -10);
    }

    #[test]
    fn agile_map() {
        let mut map = MultipleAttackPenalty::new();
        assert_eq!(map.current_penalty(true), 0);
        map.record_attack();
        assert_eq!(map.current_penalty(true), -4);
        map.record_attack();
        assert_eq!(map.current_penalty(true), -8);
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

    #[test]
    fn press_requires_prior_attack() {
        let mut pool = ActionPool::new_turn();
        let press = vec![GameTrait::press()];
        let attack = vec![GameTrait::attack()];

        // Press should be blocked before any attack
        assert!(!pool
            .can_perform(ActionCost::Actions(1), &press)
            .is_available());

        // After an attack, press should be available
        pool.spend(ActionCost::Actions(1), &attack);
        assert!(pool
            .can_perform(ActionCost::Actions(1), &press)
            .is_available());
    }

    #[test]
    fn open_allows_multiple_before_non_open() {
        let mut pool = ActionPool::new_turn();
        let open = vec![GameTrait::open()];

        // First open action should succeed
        assert!(pool
            .can_perform(ActionCost::Actions(1), &open)
            .is_available());
        pool.spend(ActionCost::Actions(1), &open);

        // Second open action should also succeed (no non-open action taken yet)
        assert!(pool
            .can_perform(ActionCost::Actions(1), &open)
            .is_available());
        pool.spend(ActionCost::Actions(1), &open);

        // Now spend a non-open action
        pool.spend(ActionCost::Actions(1), &[]);

        // Open action should now be blocked
        // (no remaining actions anyway, but test the flag)
        let mut pool2 = ActionPool::new_turn();
        let open = vec![GameTrait::open()];
        pool2.spend(ActionCost::Actions(1), &open);
        pool2.spend(ActionCost::Actions(1), &[]); // non-open
        assert!(!pool2
            .can_perform(ActionCost::Actions(1), &open)
            .is_available());
    }

    #[test]
    fn reset_for_new_turn() {
        let mut pool = ActionPool::new_turn();
        let attack = vec![GameTrait::attack()];
        let flourish = vec![GameTrait::flourish()];

        // Simulate some activity
        pool.spend(ActionCost::Actions(1), &attack);
        pool.spend(ActionCost::Actions(1), &flourish);
        pool.actions_lost = 1;
        pool.actions_gained = 1;

        assert_eq!(pool.actions_spent, 2);
        assert!(pool.flourish_used);
        assert!(pool.non_open_action_taken);
        assert_eq!(pool.map.attacks_made, 1);

        // Reset for new turn
        pool.reset_for_new_turn();

        assert_eq!(pool.actions_spent, 0);
        assert_eq!(pool.actions_lost, 0);
        assert_eq!(pool.actions_gained, 0);
        assert_eq!(pool.map.attacks_made, 0);
        assert!(!pool.flourish_used);
        assert!(!pool.non_open_action_taken);

        // base_actions and reaction_used should be preserved
        assert_eq!(pool.base_actions, 3);
    }
}
