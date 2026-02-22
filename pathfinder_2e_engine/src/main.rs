use bevy::prelude::*;
use pathfinder_mechanics::PathfinderPlugin;

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(PathfinderPlugin)
        .run();
}
