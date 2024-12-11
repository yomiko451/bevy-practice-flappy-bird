//#![windows_subsystem = "windows"]
//TODO: 记得打开
mod player;
mod world;
use bevy::prelude::*;

fn main() -> AppExit {
    App::new()
        .add_plugins(world::World)
        .add_plugins(player::Player)
        .run()
}
