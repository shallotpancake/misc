//! # Display Module
//!
//! Renders the game state to the terminal using plain `print!`/`println!`
//! and ANSI escape codes. No external crates are used.
//!
//! The display is split into sections:
//! 1. Header (round/turn info)
//! 2. Grid (terrain + creatures)
//! 3. Legend
//! 4. Player status
//! 5. Enemy list
//! 6. Combat log
//! 7. Action menu

use std::io::Write;

use crate::spatial::grid2d::Grid2D;
use crate::spatial::position::Position;
use crate::spatial::terrain::Terrain;
use crate::spatial::topology::Topology;

/// Maximum number of columns to display in the grid viewport.
const MAX_DISPLAY_COLS: u32 = 40;

/// Maximum number of rows to display in the grid viewport.
const MAX_DISPLAY_ROWS: u32 = 25;

/// Information about a creature to display on the grid.
#[derive(Debug, Clone)]
pub struct CreatureDisplay {
    /// The creature's grid position.
    pub position: Position,
    /// The character used to represent this creature on the grid.
    pub display_char: char,
    /// The creature's name (for legend/status displays).
    pub name: String,
    /// Whether the creature is still alive (dead creatures are not rendered).
    pub is_alive: bool,
}

/// Clear the terminal screen using ANSI escape codes.
///
/// Sends the "erase entire screen" sequence followed by "cursor to home",
/// which resets the terminal viewport to the top-left corner.
pub fn clear_screen() {
    print!("\x1b[2J\x1b[H");
    std::io::stdout().flush().unwrap();
}

/// Render the full game view to stdout.
///
/// This is the primary entry point for the display system. It clears the
/// screen and draws every section of the UI in order:
///
/// - Header with round number and turn indicator
/// - The grid with terrain and creature overlays
/// - A legend explaining grid symbols
/// - Player HP, actions remaining, and speed
/// - Enemy list with HP and coordinates
/// - Recent combat log messages
/// - The action menu prompt
///
/// # Arguments
///
/// * `grid` - The spatial grid to render.
/// * `creatures` - Slice of creature display info (including the player).
/// * `player_name` - Name of the player character (for the header).
/// * `player_hp_current` - Player's current hit points.
/// * `player_hp_max` - Player's maximum hit points.
/// * `actions_remaining` - How many actions the player has left this turn.
/// * `speed` - Player's movement speed in feet.
/// * `round` - The current combat round number.
/// * `is_player_turn` - Whether it is currently the player's turn.
/// * `enemies` - Slice of `(name, hp_current, hp_max, position, alive)` tuples.
/// * `combat_log` - All combat log messages (most recent last).
/// * `max_log_lines` - Maximum number of log lines to display.
#[allow(clippy::too_many_arguments)]
pub fn render_game(
    grid: &Grid2D,
    creatures: &[CreatureDisplay],
    _player_name: &str,
    player_hp_current: i32,
    player_hp_max: i32,
    actions_remaining: u32,
    speed: u32,
    round: u32,
    is_player_turn: bool,
    enemies: &[(String, i32, i32, Position, bool)],
    combat_log: &[String],
    max_log_lines: usize,
) {
    clear_screen();

    // --- Header ---
    println!("=== PATHFINDER 2E - SANDBOX ARENA ===");
    let turn_label = if is_player_turn {
        "Your Turn"
    } else {
        "Enemy Turn"
    };
    println!("Round {} | {}", round, turn_label);
    println!();

    // --- Grid ---
    render_grid(grid, creatures);
    println!();

    // --- Legend ---
    println!(
        "Legend: @ = You  G = Goblin  S = Skeleton  # = Wall  ~ = Difficult  ! = Hazard  . = Normal"
    );
    println!();

    // --- Player Status ---
    println!("--- YOUR STATUS ---");
    println!(
        "HP: {}/{} | Actions: {} | Speed: {} ft",
        player_hp_current, player_hp_max, actions_remaining, speed
    );
    println!();

    // --- Enemies ---
    println!("--- ENEMIES ---");
    if enemies.is_empty() {
        println!("  No enemies remaining.");
    } else {
        for (i, (name, hp_cur, hp_max, pos, alive)) in enemies.iter().enumerate() {
            let status = if *alive {
                format!("HP: {}/{}", hp_cur, hp_max)
            } else {
                "DEAD".to_string()
            };
            println!(
                "{}. {} ({}) [{},{}]",
                i + 1,
                name,
                status,
                pos.x,
                pos.y
            );
        }
    }
    println!();

    // --- Combat Log ---
    println!("--- COMBAT LOG ---");
    if combat_log.is_empty() {
        println!("> Waiting...");
    } else {
        let start = if combat_log.len() > max_log_lines {
            combat_log.len() - max_log_lines
        } else {
            0
        };
        for msg in &combat_log[start..] {
            println!("> {}", msg);
        }
    }
    println!();

    // --- Action Menu ---
    println!("--- ACTIONS ---");
    println!("[M] Move (N/S/E/W/NE/NW/SE/SW)");
    println!("[A] Attack an adjacent enemy");
    println!("[E] End Turn");
    println!("[Q] Quit");
    println!();

    std::io::stdout().flush().unwrap();
}

/// Render just the grid portion of the display.
///
/// Draws column headers across the top, row numbers on the left, and fills
/// each cell with either the terrain character or a creature's display
/// character if one occupies that cell.
///
/// The viewport is capped at [`MAX_DISPLAY_COLS`] x [`MAX_DISPLAY_ROWS`] so
/// that very large grids do not overflow the terminal.
fn render_grid(grid: &Grid2D, creatures: &[CreatureDisplay]) {
    let display_width = grid.width.min(MAX_DISPLAY_COLS);
    let display_height = grid.height.min(MAX_DISPLAY_ROWS);

    // Determine the column-number width for alignment. We need enough space
    // to print row labels on the left side.
    let row_label_width = digit_count(display_height.saturating_sub(1));
    let col_label_width = 2usize; // each column is 2 chars wide ("X ")

    // Print column header row.
    // Pad with spaces to align with the row-label gutter.
    print!("{:width$}", "", width = row_label_width + 1);
    for col in 0..display_width {
        print!("{:<width$}", col, width = col_label_width);
    }
    println!();

    // Print each row.
    for row in 0..display_height {
        // Row label.
        print!("{:>width$} ", row, width = row_label_width);

        for col in 0..display_width {
            let pos = Position::new(col as i32, row as i32);

            // Check if any living creature occupies this cell.
            let cell_char = creature_at(creatures, pos)
                .unwrap_or_else(|| terrain_char(grid.terrain_at(pos)));

            print!("{} ", cell_char);
        }
        println!();
    }
}

/// Look up the display character for a living creature at `pos`.
///
/// Returns `Some(char)` if an alive creature is found at the position,
/// otherwise `None`.
fn creature_at(creatures: &[CreatureDisplay], pos: Position) -> Option<char> {
    creatures
        .iter()
        .find(|c| c.is_alive && c.position == pos)
        .map(|c| c.display_char)
}

/// Get the character to display for a terrain type.
///
/// Mapping:
/// - `Normal` -> `'.'`
/// - `Difficult` / `GreaterDifficult` -> `'~'`
/// - `Impassable` -> `'#'`
/// - `Hazardous` -> `'!'`
fn terrain_char(terrain: Terrain) -> char {
    match terrain {
        Terrain::Normal => '.',
        Terrain::Difficult => '~',
        Terrain::GreaterDifficult => '~',
        Terrain::Impassable => '#',
        Terrain::Hazardous => '!',
    }
}

/// Return the number of decimal digits needed to represent `n`.
///
/// Used for aligning row labels in the grid display.
fn digit_count(n: u32) -> usize {
    if n == 0 {
        return 1;
    }
    let mut count = 0usize;
    let mut val = n;
    while val > 0 {
        count += 1;
        val /= 10;
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terrain_char_mapping() {
        assert_eq!(terrain_char(Terrain::Normal), '.');
        assert_eq!(terrain_char(Terrain::Difficult), '~');
        assert_eq!(terrain_char(Terrain::GreaterDifficult), '~');
        assert_eq!(terrain_char(Terrain::Impassable), '#');
        assert_eq!(terrain_char(Terrain::Hazardous), '!');
    }

    #[test]
    fn digit_count_values() {
        assert_eq!(digit_count(0), 1);
        assert_eq!(digit_count(1), 1);
        assert_eq!(digit_count(9), 1);
        assert_eq!(digit_count(10), 2);
        assert_eq!(digit_count(99), 2);
        assert_eq!(digit_count(100), 3);
    }

    #[test]
    fn creature_at_finds_alive() {
        let creatures = vec![
            CreatureDisplay {
                position: Position::new(3, 4),
                display_char: '@',
                name: "Player".into(),
                is_alive: true,
            },
            CreatureDisplay {
                position: Position::new(5, 5),
                display_char: 'G',
                name: "Goblin".into(),
                is_alive: false,
            },
        ];
        assert_eq!(creature_at(&creatures, Position::new(3, 4)), Some('@'));
        // Dead creature should not be returned.
        assert_eq!(creature_at(&creatures, Position::new(5, 5)), None);
        // Empty cell.
        assert_eq!(creature_at(&creatures, Position::new(0, 0)), None);
    }
}
