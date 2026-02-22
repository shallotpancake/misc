use pathfinder_mechanics::game::runner;
use pathfinder_mechanics::game::sandbox;

fn main() {
    println!("=== PATHFINDER 2e - SANDBOX ARENA ===");
    println!();
    println!("A turn-based tactical combat game.");
    println!("Defeat all enemies in the arena to win!");
    println!();
    println!("Controls:");
    println!("  [M] Move      - Move one square in any direction");
    println!("  [A] Attack    - Strike an adjacent enemy");
    println!("  [E] End Turn  - End your turn early");
    println!("  [Q] Quit      - Exit the game");
    println!();
    println!("Press Enter to begin...");

    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf).ok();

    let mut encounter = sandbox::create_sandbox();
    runner::run_game(&mut encounter);
}
