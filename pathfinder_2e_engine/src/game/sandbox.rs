//! Sandbox map and encounter setup for the first playable game.
//!
//! Creates a 20x15 arena with varied terrain and spawns a player
//! character plus a handful of enemies.

use crate::action::ActionPool;
use crate::condition::Conditions;
use crate::entity::bestiary;
use crate::entity::creature::HitPoints;
use crate::entity::enemy::AttackData;
use crate::game::faction::Faction;
use crate::mechanics::dice::Die;
use crate::mechanics::traits::{GameTrait, TraitCategory};
use crate::spatial::grid2d::Grid2D;
use crate::spatial::position::Position;
use crate::spatial::terrain::Terrain;
use crate::spatial::topology::Topology;

/// A creature in the sandbox game (no ECS — plain struct for terminal game).
pub struct GameCreature {
    pub name: String,
    pub hp: HitPoints,
    pub ac: i32,
    pub speed: u32,
    pub position: Position,
    pub actions: ActionPool,
    pub conditions: Conditions,
    pub attacks: Vec<AttackData>,
    pub faction: Faction,
    pub display_char: char,
}

impl GameCreature {
    pub fn is_alive(&self) -> bool {
        self.hp.current > 0
    }
}

/// Everything needed to play a sandbox encounter.
pub struct SandboxEncounter {
    pub grid: Grid2D,
    pub player: GameCreature,
    pub enemies: Vec<GameCreature>,
    pub combat_log: Vec<String>,
    pub round: u32,
    pub is_player_turn: bool,
    pub current_enemy_index: usize,
}

impl SandboxEncounter {
    /// Check if the player has won (all enemies dead).
    pub fn player_won(&self) -> bool {
        self.enemies.iter().all(|e| !e.is_alive())
    }

    /// Check if the player has lost (player dead).
    pub fn player_lost(&self) -> bool {
        !self.player.is_alive()
    }

    /// Log a combat message.
    pub fn log(&mut self, msg: impl Into<String>) {
        self.combat_log.push(msg.into());
    }
}

/// Create the sandbox encounter: a 20x15 arena with terrain and enemies.
pub fn create_sandbox() -> SandboxEncounter {
    let mut grid = Grid2D::new(20, 15);

    // Create interesting terrain layout:
    // Stone walls forming partial cover
    for x in 4..=6 {
        grid.set_terrain(Position::new(x, 3), Terrain::Impassable);
    }
    for x in 13..=15 {
        grid.set_terrain(Position::new(x, 3), Terrain::Impassable);
    }
    grid.set_terrain(Position::new(4, 4), Terrain::Impassable);
    grid.set_terrain(Position::new(15, 4), Terrain::Impassable);

    // Difficult terrain: rubble patches
    for x in 8..=11 {
        grid.set_terrain(Position::new(x, 6), Terrain::Difficult);
        grid.set_terrain(Position::new(x, 7), Terrain::Difficult);
    }

    // More walls in the south
    for x in 6..=8 {
        grid.set_terrain(Position::new(x, 10), Terrain::Impassable);
    }
    for x in 11..=13 {
        grid.set_terrain(Position::new(x, 10), Terrain::Impassable);
    }

    // Hazardous terrain: fire pit in center
    grid.set_terrain(Position::new(9, 4), Terrain::Hazardous);
    grid.set_terrain(Position::new(10, 4), Terrain::Hazardous);

    // Difficult terrain: mud near south
    grid.set_terrain(Position::new(3, 11), Terrain::Difficult);
    grid.set_terrain(Position::new(4, 11), Terrain::Difficult);
    grid.set_terrain(Position::new(3, 12), Terrain::Difficult);

    // Player: a capable level 1 fighter
    let player_attacks = vec![
        AttackData {
            name: "Longsword".into(),
            attack_bonus: 9, // +4 Str, +3 trained, +2 level
            damage_dice: Die::D8,
            damage_dice_count: 1,
            damage_bonus: 4,
            damage_type: "slashing".into(),
            traits: vec![],
            reach_in_feet: 5,
        },
        AttackData {
            name: "Shield Bash".into(),
            attack_bonus: 9,
            damage_dice: Die::D4,
            damage_dice_count: 1,
            damage_bonus: 4,
            damage_type: "bludgeoning".into(),
            traits: vec![GameTrait::new("Agile", TraitCategory::Attack)],
            reach_in_feet: 5,
        },
    ];

    let player = GameCreature {
        name: "Hero".into(),
        hp: HitPoints::new(25),
        ac: 18,
        speed: 25,
        position: Position::new(10, 12),
        actions: ActionPool::new_turn(),
        conditions: Conditions::default(),
        attacks: player_attacks,
        faction: Faction::Player,
        display_char: '@',
    };

    // Enemies from the bestiary
    let goblin_stats = bestiary::goblin_warrior();
    let skeleton_stats = bestiary::skeleton_guard();
    let rat_stats = bestiary::giant_rat();

    let enemies = vec![
        GameCreature {
            name: goblin_stats.name.clone(),
            hp: HitPoints::new(goblin_stats.hp),
            ac: goblin_stats.ac,
            speed: goblin_stats.speed,
            position: Position::new(8, 2),
            actions: ActionPool::new_turn(),
            conditions: Conditions::default(),
            attacks: goblin_stats.enemy_abilities.strikes.clone(),
            faction: Faction::Enemy,
            display_char: 'G',
        },
        GameCreature {
            name: skeleton_stats.name.clone(),
            hp: HitPoints::new(skeleton_stats.hp),
            ac: skeleton_stats.ac,
            speed: skeleton_stats.speed,
            position: Position::new(14, 5),
            actions: ActionPool::new_turn(),
            conditions: Conditions::default(),
            attacks: skeleton_stats.enemy_abilities.strikes.clone(),
            faction: Faction::Enemy,
            display_char: 'S',
        },
        GameCreature {
            name: rat_stats.name.clone(),
            hp: HitPoints::new(rat_stats.hp),
            ac: rat_stats.ac,
            speed: rat_stats.speed,
            position: Position::new(5, 8),
            actions: ActionPool::new_turn(),
            conditions: Conditions::default(),
            attacks: rat_stats.enemy_abilities.strikes.clone(),
            faction: Faction::Enemy,
            display_char: 'R',
        },
        {
            let goblin2 = bestiary::goblin_warrior();
            GameCreature {
                name: "Goblin Archer".into(),
                hp: HitPoints::new(goblin2.hp),
                ac: goblin2.ac,
                speed: goblin2.speed,
                position: Position::new(16, 8),
                actions: ActionPool::new_turn(),
                conditions: Conditions::default(),
                attacks: goblin2.enemy_abilities.strikes,
                faction: Faction::Enemy,
                display_char: 'g',
            }
        },
    ];

    let mut encounter = SandboxEncounter {
        grid,
        player,
        enemies,
        combat_log: Vec::new(),
        round: 1,
        is_player_turn: true,
        current_enemy_index: 0,
    };

    encounter.log("=== WELCOME TO THE SANDBOX ARENA ===");
    encounter.log("Defeat all enemies to win!");
    encounter.log(format!(
        "You face: {}",
        encounter
            .enemies
            .iter()
            .map(|e| e.name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    ));
    encounter.log("--- Round 1 begins! ---");

    encounter
}
