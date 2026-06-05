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

        .insert_resource(movement::MapBounds {
            x_min: -100.0,
            x_max: 100.0,
            y_min: -75.0,
            y_max: 75.0,
        })

        .add_plugins(DefaultPlugins)

        //user defined
        .add_plugins(CameraPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(MovementPlugin)
        .add_plugins(DebugPlugin)
        .run();
}



