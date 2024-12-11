//打包时记得解开注释以在启动游戏时不显示终端
//#![windows_subsystem = "windows"] 
mod player;
mod world;
use bevy::prelude::*;

fn main() -> AppExit {
    App::new()
        .add_plugins(world::World)
        .add_plugins(player::Player)
        .run()
}
