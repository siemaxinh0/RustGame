mod debug;
mod movement;
mod player;
mod camera;

use bevy::prelude::*;

use debug::DebugPlugin;
use movement::MovementPlugin;
use player::PlayerPlugin;
use camera::CameraPlugin;



fn main() {
    App::new()
        //Bevy built-ins.
        .insert_resource(ClearColor(Color::srgb(0.1, 0.0, 0.15)))

        .add_plugins(DefaultPlugins)

        //user defined
        .add_plugins(CameraPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(MovementPlugin)
        .add_plugins(DebugPlugin)
        .run();
}



