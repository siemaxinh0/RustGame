mod debug;
mod movement;
mod player;
mod camera;
mod enemy;
mod asset_loader;
mod collision_handler;
mod collectable;

use bevy::prelude::*;

use debug::DebugPlugin;
use movement::MovementPlugin;
use player::PlayerPlugin;
use camera::CameraPlugin;
use crate::asset_loader::AssetLoaderPlugin;
use crate::collectable::CollectablePlugin;
use crate::collision_handler::CollisionDetectionPlugin;
use crate::enemy::EnemyPlugin;

fn main() {
    App::new()
        //Bevy built-ins.
        .insert_resource(ClearColor(Color::srgb(0.65, 0.75, 0.8)))

        // .insert_resource(movement::MapBounds {
        //     x_min: -1000.0,
        //     x_max: 1000.0,
        //     y_min: -1000.0,
        //     y_max: 1000.0,
        // })

        .add_plugins(DefaultPlugins)

        //user defined
        .add_plugins(AssetLoaderPlugin)
        .add_plugins(CameraPlugin)
        .add_plugins(CollisionDetectionPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(EnemyPlugin)
        .add_plugins(CollectablePlugin)
        .add_plugins(MovementPlugin)
        .add_plugins(DebugPlugin)
        .run();
}



