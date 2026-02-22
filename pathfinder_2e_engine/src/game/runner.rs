//! Main game loop for the terminal-based sandbox game.
//!
//! Orchestrates the turn cycle: render → player input → resolve action →
//! enemy AI → check victory → repeat. Uses the pure mechanics functions
//! directly rather than Bevy's ECS schedule.

use std::collections::HashSet;

use rand::thread_rng;

use crate::action::ActionCost;
use crate::game::ai::{self, AiAction};
use crate::game::combat;
use crate::game::sandbox::{GameCreature, SandboxEncounter};
use crate::mechanics::traits::GameTrait;
use crate::spatial::grid2d::Grid2D;
use crate::spatial::position::Position;
use crate::spatial::topology::Topology;
use crate::ui::display::{self, CreatureDisplay};
use crate::ui::input::{self, PlayerAction};

/// Run the full game loop until victory, defeat, or quit.
pub fn run_game(encounter: &mut SandboxEncounter) {
    let mut rng = thread_rng();

    loop {
        // Check victory/defeat
        if encounter.player_won() {
            render_state(encounter);
            println!();
            println!("*** VICTORY! You have defeated all enemies! ***");
            println!();
            break;
        }
        if encounter.player_lost() {
            render_state(encounter);
            println!();
            println!("*** DEFEAT! You have fallen in battle. ***");
            println!();
            break;
        }

        if encounter.is_player_turn {
            // === PLAYER TURN ===
            player_turn(encounter, &mut rng);
        } else {
            // === ENEMY TURNS ===
            enemy_turns(encounter, &mut rng);

            // After all enemies act, start a new round
            encounter.round += 1;
            encounter.is_player_turn = true;

            // Reset player actions for new turn
            encounter.player.actions.reset_for_new_turn();
            encounter.player.actions.new_round();

            encounter.log(format!("--- Round {} begins! ---", encounter.round));
        }
    }
}

/// Handle the player's full turn (all 3 actions).
fn player_turn(encounter: &mut SandboxEncounter, rng: &mut impl rand::Rng) {
    loop {
        // Render the current state
        render_state(encounter);

        let actions_remaining = encounter.player.actions.remaining();
        if actions_remaining == 0 {
            println!("\nNo actions remaining. Ending your turn.");
            std::thread::sleep(std::time::Duration::from_millis(1000));
            encounter.is_player_turn = false;
            return;
        }

        // Find enemies adjacent to the player
        let adjacent_enemies = find_adjacent_enemies(
            encounter.player.position,
            &encounter.enemies,
            &encounter.grid,
        );

        // Get player input
        let action = input::get_player_action(actions_remaining, &adjacent_enemies);

        match action {
            PlayerAction::Move(direction) => {
                handle_player_move(encounter, direction);
            }
            PlayerAction::Attack(enemy_idx) => {
                handle_player_attack(encounter, enemy_idx, rng);
            }
            PlayerAction::EndTurn => {
                encounter.log("You end your turn.");
                encounter.is_player_turn = false;
                return;
            }
            PlayerAction::Quit => {
                println!("\nThanks for playing!");
                std::process::exit(0);
            }
        }

        // Check if combat ended due to this action
        if encounter.player_won() || encounter.player_lost() {
            return;
        }
    }
}

/// Handle a player movement action.
fn handle_player_move(
    encounter: &mut SandboxEncounter,
    direction: crate::spatial::position::Direction,
) {
    let (dx, dy) = direction.offset();
    let new_pos = encounter.player.position.offset(dx, dy);

    // Check bounds
    if !encounter.grid.in_bounds(new_pos) {
        encounter.log("You can't move there — it's out of bounds!");
        return;
    }

    // Check terrain
    let terrain = encounter.grid.terrain_at(new_pos);
    if terrain == crate::spatial::terrain::Terrain::Impassable {
        encounter.log("You can't move there — the way is blocked!");
        return;
    }

    // Check occupancy (enemies)
    let occupied = get_occupied_positions(encounter);
    if occupied.contains(&new_pos) {
        encounter.log("You can't move there — the space is occupied!");
        return;
    }

    // Check action cost based on terrain
    let action_cost = match terrain.movement_cost() {
        Some(1) => 1u32,
        Some(_) => {
            // Difficult/greater difficult terrain costs more
            // For simplicity: difficult costs 1 action but noted
            1
        }
        None => {
            encounter.log("You can't move there — impassable!");
            return;
        }
    };

    // Spend the action
    let move_traits = vec![GameTrait::move_trait()];
    encounter
        .player
        .actions
        .spend(ActionCost::Actions(action_cost), &move_traits);

    let old_pos = encounter.player.position;
    encounter.player.position = new_pos;

    let terrain_note = if terrain == crate::spatial::terrain::Terrain::Difficult {
        " (difficult terrain)"
    } else if terrain == crate::spatial::terrain::Terrain::Hazardous {
        " (hazardous!)"
    } else {
        ""
    };

    encounter.log(format!(
        "You move from ({},{}) to ({},{}){}.",
        old_pos.x, old_pos.y, new_pos.x, new_pos.y, terrain_note
    ));
}

/// Handle a player attack action.
fn handle_player_attack(
    encounter: &mut SandboxEncounter,
    enemy_idx: usize,
    rng: &mut impl rand::Rng,
) {
    if enemy_idx >= encounter.enemies.len() {
        encounter.log("Invalid target!");
        return;
    }

    if !encounter.enemies[enemy_idx].is_alive() {
        encounter.log("That enemy is already dead!");
        return;
    }

    // Check adjacency
    if !encounter
        .grid
        .is_adjacent(encounter.player.position, encounter.enemies[enemy_idx].position)
    {
        encounter.log("That enemy is not adjacent to you!");
        return;
    }

    // Determine which weapon to use (first melee weapon)
    let attack = if encounter.player.attacks.is_empty() {
        encounter.log("You have no attacks!");
        return;
    } else {
        encounter.player.attacks[0].clone()
    };

    // Get MAP penalty
    let is_agile = attack
        .traits
        .iter()
        .any(|t| t.name.to_lowercase() == "agile");
    let map_penalty = encounter.player.actions.map.current_penalty(is_agile);

    // Collect condition modifiers
    let condition_mods =
        crate::condition::collect_condition_modifiers(&encounter.player.conditions);

    // Spend the action (attack trait)
    let attack_traits = vec![GameTrait::attack()];
    encounter
        .player
        .actions
        .spend(ActionCost::Actions(1), &attack_traits);

    // Resolve the strike
    let target_ac = encounter.enemies[enemy_idx].ac;
    let target_name = encounter.enemies[enemy_idx].name.clone();
    let result = combat::resolve_strike(
        attack.attack_bonus,
        target_ac,
        map_penalty,
        &condition_mods,
        &attack,
        rng,
    );

    // Log the result
    encounter.log(format!(
        "You attack {} with {} — {}",
        target_name, attack.name, result.description
    ));

    // Apply damage
    if result.damage_dealt > 0 {
        let died = combat::apply_damage(&mut encounter.enemies[enemy_idx].hp, result.damage_dealt);
        if died {
            encounter.log(format!("{} is slain!", target_name));
        }
    }
}

/// Handle all enemy turns.
fn enemy_turns(encounter: &mut SandboxEncounter, rng: &mut impl rand::Rng) {
    for i in 0..encounter.enemies.len() {
        if !encounter.enemies[i].is_alive() {
            continue;
        }

        // Reset enemy actions for their turn
        encounter.enemies[i].actions.reset_for_new_turn();
        encounter.enemies[i].actions.new_round();

        let enemy_name = encounter.enemies[i].name.clone();
        encounter.log(format!("{}'s turn.", enemy_name));

        // Each enemy gets up to 3 actions
        for _action_num in 0..3 {
            if !encounter.enemies[i].is_alive() || !encounter.player.is_alive() {
                break;
            }

            let my_pos = encounter.enemies[i].position;
            let my_speed = encounter.enemies[i].speed;
            let actions_remaining = encounter.enemies[i].actions.remaining();

            if actions_remaining == 0 {
                break;
            }

            let occupied = get_occupied_positions(encounter);

            let ai_action = ai::choose_enemy_action(
                my_pos,
                my_speed,
                actions_remaining,
                encounter.player.position,
                &encounter.grid,
                &occupied,
            );

            match ai_action {
                AiAction::MoveTo(pos) => {
                    let move_traits = vec![GameTrait::move_trait()];
                    encounter.enemies[i]
                        .actions
                        .spend(ActionCost::Actions(1), &move_traits);
                    encounter.enemies[i].position = pos;
                    encounter.log(format!(
                        "{} moves to ({},{}).",
                        enemy_name, pos.x, pos.y
                    ));
                }
                AiAction::Strike { attack_index } => {
                    let attack_idx =
                        attack_index.min(encounter.enemies[i].attacks.len().saturating_sub(1));
                    let attack = encounter.enemies[i].attacks[attack_idx].clone();

                    let is_agile = attack
                        .traits
                        .iter()
                        .any(|t| t.name.to_lowercase() == "agile");
                    let map_penalty =
                        encounter.enemies[i].actions.map.current_penalty(is_agile);

                    let condition_mods = crate::condition::collect_condition_modifiers(
                        &encounter.enemies[i].conditions,
                    );

                    let attack_traits = vec![GameTrait::attack()];
                    encounter.enemies[i]
                        .actions
                        .spend(ActionCost::Actions(1), &attack_traits);

                    let target_ac = encounter.player.ac;
                    let result = combat::resolve_strike(
                        attack.attack_bonus,
                        target_ac,
                        map_penalty,
                        &condition_mods,
                        &attack,
                        rng,
                    );

                    encounter.log(format!(
                        "{} attacks you with {} — {}",
                        enemy_name, attack.name, result.description
                    ));

                    if result.damage_dealt > 0 {
                        let died =
                            combat::apply_damage(&mut encounter.player.hp, result.damage_dealt);
                        if died {
                            encounter.log("You have fallen!");
                            return;
                        }
                    }
                }
                AiAction::EndTurn => {
                    break;
                }
            }
        }
    }
}

/// Render the full game state to the terminal.
fn render_state(encounter: &SandboxEncounter) {
    // Note: render_game() calls clear_screen() internally

    // Build creature display list
    let mut creatures = Vec::new();
    if encounter.player.is_alive() {
        creatures.push(CreatureDisplay {
            position: encounter.player.position,
            display_char: encounter.player.display_char,
            name: encounter.player.name.clone(),
            is_alive: true,
        });
    }
    for enemy in &encounter.enemies {
        if enemy.is_alive() {
            creatures.push(CreatureDisplay {
                position: enemy.position,
                display_char: enemy.display_char,
                name: enemy.name.clone(),
                is_alive: true,
            });
        }
    }

    // Build enemy info list
    let enemy_info: Vec<(String, i32, i32, Position, bool)> = encounter
        .enemies
        .iter()
        .map(|e| {
            (
                e.name.clone(),
                e.hp.current,
                e.hp.max,
                e.position,
                e.is_alive(),
            )
        })
        .collect();

    display::render_game(
        &encounter.grid,
        &creatures,
        &encounter.player.name,
        encounter.player.hp.current,
        encounter.player.hp.max,
        encounter.player.actions.remaining(),
        encounter.player.speed,
        encounter.round,
        encounter.is_player_turn,
        &enemy_info,
        &encounter.combat_log,
        8,
    );
}

/// Find enemies adjacent to the given position.
/// Returns (enemy_index, enemy_name) pairs.
fn find_adjacent_enemies(
    pos: Position,
    enemies: &[GameCreature],
    grid: &Grid2D,
) -> Vec<(usize, String)> {
    enemies
        .iter()
        .enumerate()
        .filter(|(_, e)| e.is_alive() && grid.is_adjacent(pos, e.position))
        .map(|(i, e)| (i, e.name.clone()))
        .collect()
}

/// Get all occupied positions (player + living enemies).
fn get_occupied_positions(encounter: &SandboxEncounter) -> HashSet<Position> {
    let mut positions = HashSet::new();
    positions.insert(encounter.player.position);
    for enemy in &encounter.enemies {
        if enemy.is_alive() {
            positions.insert(enemy.position);
        }
    }
    positions
}
