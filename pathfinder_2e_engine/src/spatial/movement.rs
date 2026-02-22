//! # Movement Rules (PF2e)
//!
//! Pure functions encoding how movement works in the Pathfinder 2e system.
//! Like `ConditionRules`, these are laws of the field — given a movement
//! request plus the state of the world, they produce a deterministic result.
//! No entity knowledge needed beyond what is passed in.

use crate::action::ActionCost;
use crate::condition::types::ConditionEffect;
use crate::spatial::position::Position;

/// The type of movement being attempted, per PF2e Core Rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MovementType {
    /// Stride: 1 action, Move trait, provokes reactions, uses full Speed.
    Stride,
    /// Step: 1 action, Move trait, does NOT provoke, exactly 5 feet.
    Step,
    /// Crawl: 1 action, Move trait, 5 feet, requires prone.
    Crawl,
    /// Climb: 1 action, Move trait, requires Athletics check,
    /// uses climb speed or 1/4 base speed.
    Climb,
    /// Swim: 1 action, Move trait, requires Athletics check,
    /// uses swim speed or 1/4 base speed.
    Swim,
    /// Fly: 1 action, Move trait, provokes reactions, uses fly speed.
    Fly,
    /// Burrow: 1 action, Move trait, uses burrow speed.
    Burrow,
}

/// A request to validate a movement. This is a data struct, not a message.
/// Systems build this from ECS data, then pass it to `MovementRules` for
/// pure validation.
#[derive(Debug, Clone)]
pub struct MovementRequest {
    /// The entity attempting to move (opaque id for logging/tracing).
    pub entity: bevy::ecs::entity::Entity,
    /// What kind of movement is being attempted.
    pub movement_type: MovementType,
    /// The path the entity wants to traverse (ordered list of positions).
    pub path: Vec<Position>,
    /// The entity's base speed in feet (before condition modifiers).
    pub speed_in_feet: u32,
}

/// The result of validating a movement request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MovementValidation {
    /// Movement is permitted.
    Valid {
        /// How many feet of movement the path costs.
        cost_in_feet: u32,
        /// Whether this movement provokes reactions (e.g., Attacks of Opportunity).
        provokes: bool,
    },
    /// Movement is blocked.
    Blocked {
        /// Human-readable reason the movement was denied.
        reason: String,
    },
}

/// Pure rules for PF2e movement. Stateless — all inputs are passed in,
/// all outputs are returned. Mirrors the `ConditionRules` pattern.
pub struct MovementRules;

impl MovementRules {
    /// Validate a complete movement request against terrain cost and active conditions.
    ///
    /// Checks applied in order:
    /// 1. `PreventMovement` blocks all movement outright.
    /// 2. `ReduceSpeed` reduces available distance.
    /// 3. `Step` must cost exactly 5 feet of raw terrain cost.
    /// 4. Terrain cost must not exceed available distance.
    pub fn validate_movement(
        request: &MovementRequest,
        terrain_cost: u32,
        conditions: &[ConditionEffect],
    ) -> MovementValidation {
        // 1. PreventMovement blocks everything
        for effect in conditions {
            if matches!(effect, ConditionEffect::PreventMovement) {
                return MovementValidation::Blocked {
                    reason: "Cannot move: movement is prevented by a condition".into(),
                };
            }
        }

        // 2. Calculate effective speed after condition modifiers
        let speed = Self::effective_speed(request.speed_in_feet, conditions);

        // 3. Determine max distance for this movement type
        let max_dist = Self::max_distance(&request.movement_type, speed);

        // 4. Step must be exactly 5 feet of raw terrain cost
        if request.movement_type == MovementType::Step && terrain_cost != 5 {
            return MovementValidation::Blocked {
                reason: format!(
                    "Step must cover exactly 5 feet, but path costs {} feet",
                    terrain_cost
                ),
            };
        }

        // 5. Check whether the terrain cost fits within available distance
        if terrain_cost > max_dist {
            return MovementValidation::Blocked {
                reason: format!(
                    "Movement costs {} feet but only {} feet available (speed {}, type {:?})",
                    terrain_cost, max_dist, speed, request.movement_type
                ),
            };
        }

        MovementValidation::Valid {
            cost_in_feet: terrain_cost,
            provokes: Self::provokes_reactions(&request.movement_type),
        }
    }

    /// Calculate effective speed after applying all `ReduceSpeed` condition effects.
    /// Speed cannot be reduced below 0.
    pub fn effective_speed(base_speed: u32, conditions: &[ConditionEffect]) -> u32 {
        let mut speed = base_speed;
        for effect in conditions {
            if let ConditionEffect::ReduceSpeed(amount) = effect {
                speed = speed.saturating_sub(*amount);
            }
        }
        speed
    }

    /// The action cost of a movement type in PF2e's three-action economy.
    /// All standard movement types cost 1 action.
    pub fn action_cost(movement_type: &MovementType) -> ActionCost {
        match movement_type {
            MovementType::Stride
            | MovementType::Step
            | MovementType::Crawl
            | MovementType::Climb
            | MovementType::Swim
            | MovementType::Fly
            | MovementType::Burrow => ActionCost::Actions(1),
        }
    }

    /// Whether a movement type provokes reactions (e.g., Attack of Opportunity).
    /// Step and Crawl specifically do not provoke.
    pub fn provokes_reactions(movement_type: &MovementType) -> bool {
        match movement_type {
            MovementType::Stride => true,
            MovementType::Step => false,
            MovementType::Crawl => false,
            MovementType::Climb => true,
            MovementType::Swim => true,
            MovementType::Fly => true,
            MovementType::Burrow => true,
        }
    }

    /// Maximum distance (in feet) a movement type allows.
    ///
    /// - Stride / Fly / Burrow: full speed
    /// - Step / Crawl: 5 feet
    /// - Climb / Swim: quarter speed (minimum 5 feet)
    pub fn max_distance(movement_type: &MovementType, speed: u32) -> u32 {
        match movement_type {
            MovementType::Stride | MovementType::Fly | MovementType::Burrow => speed,
            MovementType::Step | MovementType::Crawl => 5,
            MovementType::Climb | MovementType::Swim => (speed / 4).max(5),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: build a minimal entity id for test requests.
    fn test_entity() -> bevy::ecs::entity::Entity {
        // Entity::from_raw is available for testing; we just need a placeholder.
        bevy::ecs::entity::Entity::from_bits(42)
    }

    // ------------------------------------------------------------------
    // Stride with full speed
    // ------------------------------------------------------------------

    #[test]
    fn stride_full_speed_valid() {
        let request = MovementRequest {
            entity: test_entity(),
            movement_type: MovementType::Stride,
            path: vec![
                Position::new(0, 0),
                Position::new(1, 0),
                Position::new(2, 0),
                Position::new(3, 0),
                Position::new(4, 0),
                Position::new(5, 0),
            ],
            speed_in_feet: 25,
        };
        // 5 steps * 5 ft each = 25 ft terrain cost
        let terrain_cost = 25;
        let result = MovementRules::validate_movement(&request, terrain_cost, &[]);
        assert_eq!(
            result,
            MovementValidation::Valid {
                cost_in_feet: 25,
                provokes: true,
            }
        );
    }

    #[test]
    fn stride_exceeding_speed_blocked() {
        let request = MovementRequest {
            entity: test_entity(),
            movement_type: MovementType::Stride,
            path: vec![
                Position::new(0, 0),
                Position::new(1, 0),
                Position::new(2, 0),
                Position::new(3, 0),
                Position::new(4, 0),
                Position::new(5, 0),
                Position::new(6, 0),
            ],
            speed_in_feet: 25,
        };
        // 6 steps * 5 ft = 30 ft, but speed is only 25
        let terrain_cost = 30;
        let result = MovementRules::validate_movement(&request, terrain_cost, &[]);
        assert!(matches!(result, MovementValidation::Blocked { .. }));
    }

    // ------------------------------------------------------------------
    // Step limited to 5 feet
    // ------------------------------------------------------------------

    #[test]
    fn step_exactly_five_feet_valid() {
        let request = MovementRequest {
            entity: test_entity(),
            movement_type: MovementType::Step,
            path: vec![Position::new(0, 0), Position::new(1, 0)],
            speed_in_feet: 25,
        };
        let terrain_cost = 5;
        let result = MovementRules::validate_movement(&request, terrain_cost, &[]);
        assert_eq!(
            result,
            MovementValidation::Valid {
                cost_in_feet: 5,
                provokes: false,
            }
        );
    }

    #[test]
    fn step_more_than_five_feet_blocked() {
        let request = MovementRequest {
            entity: test_entity(),
            movement_type: MovementType::Step,
            path: vec![
                Position::new(0, 0),
                Position::new(1, 0),
                Position::new(2, 0),
            ],
            speed_in_feet: 25,
        };
        // 10 feet of terrain cost — Step requires exactly 5
        let terrain_cost = 10;
        let result = MovementRules::validate_movement(&request, terrain_cost, &[]);
        assert!(matches!(result, MovementValidation::Blocked { .. }));
    }

    // ------------------------------------------------------------------
    // PreventMovement blocks all movement
    // ------------------------------------------------------------------

    #[test]
    fn prevent_movement_blocks_stride() {
        let request = MovementRequest {
            entity: test_entity(),
            movement_type: MovementType::Stride,
            path: vec![Position::new(0, 0), Position::new(1, 0)],
            speed_in_feet: 25,
        };
        let conditions = vec![ConditionEffect::PreventMovement];
        let result = MovementRules::validate_movement(&request, 5, &conditions);
        assert!(matches!(result, MovementValidation::Blocked { .. }));
    }

    #[test]
    fn prevent_movement_blocks_step() {
        let request = MovementRequest {
            entity: test_entity(),
            movement_type: MovementType::Step,
            path: vec![Position::new(0, 0), Position::new(1, 0)],
            speed_in_feet: 25,
        };
        let conditions = vec![ConditionEffect::PreventMovement];
        let result = MovementRules::validate_movement(&request, 5, &conditions);
        assert!(matches!(result, MovementValidation::Blocked { .. }));
    }

    // ------------------------------------------------------------------
    // ReduceSpeed reduces effective speed
    // ------------------------------------------------------------------

    #[test]
    fn reduce_speed_lowers_effective_speed() {
        assert_eq!(
            MovementRules::effective_speed(25, &[ConditionEffect::ReduceSpeed(10)]),
            15
        );
    }

    #[test]
    fn reduce_speed_stacks() {
        let conditions = vec![
            ConditionEffect::ReduceSpeed(5),
            ConditionEffect::ReduceSpeed(10),
        ];
        assert_eq!(MovementRules::effective_speed(30, &conditions), 15);
    }

    #[test]
    fn reduce_speed_cannot_go_below_zero() {
        assert_eq!(
            MovementRules::effective_speed(10, &[ConditionEffect::ReduceSpeed(20)]),
            0
        );
    }

    #[test]
    fn reduce_speed_blocks_stride_that_was_valid() {
        let request = MovementRequest {
            entity: test_entity(),
            movement_type: MovementType::Stride,
            path: vec![
                Position::new(0, 0),
                Position::new(1, 0),
                Position::new(2, 0),
                Position::new(3, 0),
                Position::new(4, 0),
            ],
            speed_in_feet: 25,
        };
        // 4 steps * 5 ft = 20 ft of terrain
        let terrain_cost = 20;
        // Reduce speed by 10 -> effective speed = 15, which < 20
        let conditions = vec![ConditionEffect::ReduceSpeed(10)];
        let result = MovementRules::validate_movement(&request, terrain_cost, &conditions);
        assert!(matches!(result, MovementValidation::Blocked { .. }));
    }

    // ------------------------------------------------------------------
    // Difficult terrain doubles cost
    // ------------------------------------------------------------------

    #[test]
    fn difficult_terrain_doubles_cost_blocks_if_over_speed() {
        let request = MovementRequest {
            entity: test_entity(),
            movement_type: MovementType::Stride,
            path: vec![
                Position::new(0, 0),
                Position::new(1, 0),
                Position::new(2, 0),
                Position::new(3, 0),
            ],
            speed_in_feet: 25,
        };
        // 3 steps through difficult terrain: 3 * 5 * 2 = 30 ft terrain cost
        // (terrain_cost is already computed by Topology::path_cost_in_feet)
        let terrain_cost = 30;
        let result = MovementRules::validate_movement(&request, terrain_cost, &[]);
        assert!(matches!(result, MovementValidation::Blocked { .. }));
    }

    #[test]
    fn difficult_terrain_within_speed_valid() {
        let request = MovementRequest {
            entity: test_entity(),
            movement_type: MovementType::Stride,
            path: vec![
                Position::new(0, 0),
                Position::new(1, 0),
                Position::new(2, 0),
            ],
            speed_in_feet: 25,
        };
        // 2 steps through difficult terrain: 2 * 5 * 2 = 20 ft
        let terrain_cost = 20;
        let result = MovementRules::validate_movement(&request, terrain_cost, &[]);
        assert_eq!(
            result,
            MovementValidation::Valid {
                cost_in_feet: 20,
                provokes: true,
            }
        );
    }

    // ------------------------------------------------------------------
    // Movement type action costs
    // ------------------------------------------------------------------

    #[test]
    fn all_movement_types_cost_one_action() {
        let types = [
            MovementType::Stride,
            MovementType::Step,
            MovementType::Crawl,
            MovementType::Climb,
            MovementType::Swim,
            MovementType::Fly,
            MovementType::Burrow,
        ];
        for mt in &types {
            assert_eq!(
                MovementRules::action_cost(mt),
                ActionCost::Actions(1),
                "{:?} should cost 1 action",
                mt
            );
        }
    }

    // ------------------------------------------------------------------
    // Provokes reactions check
    // ------------------------------------------------------------------

    #[test]
    fn stride_provokes_reactions() {
        assert!(MovementRules::provokes_reactions(&MovementType::Stride));
    }

    #[test]
    fn step_does_not_provoke_reactions() {
        assert!(!MovementRules::provokes_reactions(&MovementType::Step));
    }

    #[test]
    fn crawl_does_not_provoke_reactions() {
        assert!(!MovementRules::provokes_reactions(&MovementType::Crawl));
    }

    #[test]
    fn climb_provokes_reactions() {
        assert!(MovementRules::provokes_reactions(&MovementType::Climb));
    }

    #[test]
    fn swim_provokes_reactions() {
        assert!(MovementRules::provokes_reactions(&MovementType::Swim));
    }

    #[test]
    fn fly_provokes_reactions() {
        assert!(MovementRules::provokes_reactions(&MovementType::Fly));
    }

    #[test]
    fn burrow_provokes_reactions() {
        assert!(MovementRules::provokes_reactions(&MovementType::Burrow));
    }

    // ------------------------------------------------------------------
    // Max distance for each movement type
    // ------------------------------------------------------------------

    #[test]
    fn max_distance_stride_equals_speed() {
        assert_eq!(MovementRules::max_distance(&MovementType::Stride, 30), 30);
    }

    #[test]
    fn max_distance_step_always_five() {
        assert_eq!(MovementRules::max_distance(&MovementType::Step, 30), 5);
        assert_eq!(MovementRules::max_distance(&MovementType::Step, 100), 5);
    }

    #[test]
    fn max_distance_crawl_always_five() {
        assert_eq!(MovementRules::max_distance(&MovementType::Crawl, 30), 5);
    }

    #[test]
    fn max_distance_climb_quarter_speed() {
        // 30 / 4 = 7 (integer division), which is > 5 so no clamping
        assert_eq!(MovementRules::max_distance(&MovementType::Climb, 30), 7);
    }

    #[test]
    fn max_distance_climb_minimum_five() {
        // 16 / 4 = 4, but minimum is 5
        assert_eq!(MovementRules::max_distance(&MovementType::Climb, 16), 5);
    }

    #[test]
    fn max_distance_swim_quarter_speed() {
        assert_eq!(MovementRules::max_distance(&MovementType::Swim, 40), 10);
    }

    #[test]
    fn max_distance_fly_equals_speed() {
        assert_eq!(MovementRules::max_distance(&MovementType::Fly, 60), 60);
    }

    #[test]
    fn max_distance_burrow_equals_speed() {
        assert_eq!(MovementRules::max_distance(&MovementType::Burrow, 20), 20);
    }
}
