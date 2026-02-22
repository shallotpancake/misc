//! # Input Module
//!
//! Handles player input from stdin for the terminal UI. Prompts the player
//! for actions (move, attack, end turn, quit) and parses their responses
//! into structured [`PlayerAction`] values.
//!
//! Uses only `std::io` -- no external crates.

use crate::spatial::position::Direction;
use std::io::{self, Write};

/// Actions the player can take during their turn.
///
/// Each variant corresponds to a menu option presented to the player.
/// The game loop receives a `PlayerAction` and applies it to the game state.
#[derive(Debug, Clone)]
pub enum PlayerAction {
    /// Move the player character one square in the given direction.
    Move(Direction),
    /// Attack the enemy at the given index (0-based into the enemy list).
    Attack(usize),
    /// End the current turn, forfeiting any remaining actions.
    EndTurn,
    /// Quit the game entirely.
    Quit,
}

/// Get the player's chosen action by prompting on stdout and reading from stdin.
///
/// This function loops until the player provides valid input. It displays the
/// action prompt and handles sub-prompts for movement direction and attack
/// target selection.
///
/// # Arguments
///
/// * `actions_remaining` - How many actions the player has left this turn.
///   If zero, a message is printed and [`PlayerAction::EndTurn`] is returned
///   immediately.
/// * `adjacent_enemies` - A list of `(index, name)` pairs for enemies that
///   are adjacent to the player and can be attacked. The index refers to the
///   enemy's position in the global enemy list. An empty slice means no
///   enemies are in melee range.
///
/// # Returns
///
/// A [`PlayerAction`] representing the player's choice.
pub fn get_player_action(
    actions_remaining: u32,
    adjacent_enemies: &[(usize, String)],
) -> PlayerAction {
    if actions_remaining == 0 {
        println!("No actions remaining. Ending turn automatically.");
        pause_for_enter();
        return PlayerAction::EndTurn;
    }

    loop {
        print!("Choose action (M/A/E/Q): ");
        io::stdout().flush().unwrap();

        let line = read_line_trimmed();
        if line.is_empty() {
            continue;
        }

        match line.chars().next().unwrap().to_ascii_lowercase() {
            'm' => {
                if let Some(dir) = get_direction() {
                    return PlayerAction::Move(dir);
                }
                // User cancelled -- re-show the action prompt.
                println!("Movement cancelled.");
                println!();
            }
            'a' => {
                if adjacent_enemies.is_empty() {
                    println!("No enemies are adjacent to you.");
                    println!();
                    continue;
                }
                if let Some(target_idx) = get_attack_target(adjacent_enemies) {
                    return PlayerAction::Attack(target_idx);
                }
                // User cancelled -- re-show the action prompt.
                println!("Attack cancelled.");
                println!();
            }
            'e' => {
                return PlayerAction::EndTurn;
            }
            'q' => {
                return PlayerAction::Quit;
            }
            _ => {
                println!("Invalid choice. Please enter M, A, E, or Q.");
                println!();
            }
        }
    }
}

/// Prompt the player for a movement direction.
///
/// Accepts cardinal directions (n/s/e/w), diagonal directions (ne/nw/se/sw),
/// and their full names (north, south, etc.). Case-insensitive.
///
/// Entering "back" or "cancel" returns `None`, allowing the player to
/// abort the movement and return to the main action menu.
///
/// # Returns
///
/// `Some(Direction)` if the player chose a valid direction, or `None` if
/// they cancelled.
fn get_direction() -> Option<Direction> {
    loop {
        print!("Direction (N/S/E/W/NE/NW/SE/SW, or 'cancel'): ");
        io::stdout().flush().unwrap();

        let input = read_line_trimmed().to_ascii_lowercase();
        if input.is_empty() {
            continue;
        }

        match input.as_str() {
            "n" | "north" => return Some(Direction::North),
            "s" | "south" => return Some(Direction::South),
            "e" | "east" => return Some(Direction::East),
            "w" | "west" => return Some(Direction::West),
            "ne" | "northeast" => return Some(Direction::NorthEast),
            "nw" | "northwest" => return Some(Direction::NorthWest),
            "se" | "southeast" => return Some(Direction::SouthEast),
            "sw" | "southwest" => return Some(Direction::SouthWest),
            "back" | "cancel" => return None,
            _ => {
                println!("Invalid direction. Use N/S/E/W/NE/NW/SE/SW or 'cancel'.");
            }
        }
    }
}

/// Prompt the player to choose which adjacent enemy to attack.
///
/// If there is exactly one adjacent enemy, it is selected automatically
/// with a confirmation message. Otherwise, a numbered list is displayed
/// and the player picks by number.
///
/// Entering "back" or "cancel" (or 0) returns `None`, allowing the player
/// to abort the attack and return to the main action menu.
///
/// # Arguments
///
/// * `adjacent_enemies` - A non-empty slice of `(index, name)` pairs. The
///   `index` is the enemy's position in the global enemy list and will be
///   returned as the target identifier.
///
/// # Returns
///
/// `Some(index)` if the player selected a target, or `None` if they
/// cancelled.
fn get_attack_target(adjacent_enemies: &[(usize, String)]) -> Option<usize> {
    // Auto-select if only one enemy is adjacent.
    if adjacent_enemies.len() == 1 {
        let (idx, ref name) = adjacent_enemies[0];
        println!("Attacking {} (only adjacent enemy).", name);
        return Some(idx);
    }

    // Show the list of adjacent enemies.
    println!("Adjacent enemies:");
    for (list_num, (_, ref name)) in adjacent_enemies.iter().enumerate() {
        println!("  {}. {}", list_num + 1, name);
    }

    loop {
        print!("Choose target (1-{}, or 'cancel'): ", adjacent_enemies.len());
        io::stdout().flush().unwrap();

        let input = read_line_trimmed().to_ascii_lowercase();
        if input.is_empty() {
            continue;
        }

        if input == "back" || input == "cancel" || input == "0" {
            return None;
        }

        match input.parse::<usize>() {
            Ok(n) if n >= 1 && n <= adjacent_enemies.len() => {
                let (idx, _) = adjacent_enemies[n - 1];
                return Some(idx);
            }
            Ok(_) => {
                println!(
                    "Out of range. Please enter a number between 1 and {}.",
                    adjacent_enemies.len()
                );
            }
            Err(_) => {
                println!("Invalid input. Enter a number or 'cancel'.");
            }
        }
    }
}

/// Read a line from stdin, trim whitespace, and return it as a `String`.
///
/// If reading fails (e.g. stdin is closed), returns an empty string rather
/// than panicking, which allows the caller to handle it gracefully.
fn read_line_trimmed() -> String {
    let mut buf = String::new();
    match io::stdin().read_line(&mut buf) {
        Ok(0) => {
            // EOF -- treat as empty input.
            String::new()
        }
        Ok(_) => buf.trim().to_string(),
        Err(_) => String::new(),
    }
}

/// Print a "press Enter to continue" prompt and wait for a keypress.
///
/// Used when the player needs to acknowledge a message before the display
/// updates (e.g., "no actions remaining").
fn pause_for_enter() {
    print!("(Press Enter to continue) ");
    io::stdout().flush().unwrap();
    let _ = read_line_trimmed();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_action_debug_display() {
        // Verify the Debug derive works for all variants.
        let actions: Vec<PlayerAction> = vec![
            PlayerAction::Move(Direction::North),
            PlayerAction::Attack(0),
            PlayerAction::EndTurn,
            PlayerAction::Quit,
        ];
        for action in &actions {
            let debug_str = format!("{:?}", action);
            assert!(!debug_str.is_empty());
        }
    }

    #[test]
    fn player_action_clone() {
        let original = PlayerAction::Move(Direction::SouthWest);
        let cloned = original.clone();
        // Both should produce the same debug output.
        assert_eq!(format!("{:?}", original), format!("{:?}", cloned));
    }
}
