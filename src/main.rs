mod debug;
mod movement;
mod player;
mod camera;
mod enemy;
mod asset_loader;
mod collision_handler;
mod collectable;
mod map;
mod state;
mod coin;
mod club;
mod boss;

use bevy::prelude::*;

use debug::DebugPlugin;
use movement::MovementPlugin;
use player::PlayerPlugin;
use camera::CameraPlugin;
use crate::asset_loader::AssetLoaderPlugin;
use crate::collectable::CollectablePlugin;
use crate::collision_handler::CollisionDetectionPlugin;
use crate::enemy::EnemyPlugin;
use crate::map::MapPlugin;
use crate::state::StatePlugin;
use crate::coin::CoinPlugin;
use crate::club::ClubPlugin;
use crate::boss::BossPlugin;

fn main() {
    App::new()
        //Bevy built-ins.
        .insert_resource(ClearColor(Color::srgb(0.65, 0.75, 0.8)))
        

        .add_plugins(DefaultPlugins)

        //user defined
        .add_plugins(StatePlugin)
        .add_plugins(AssetLoaderPlugin)
        .add_plugins(CameraPlugin)
        .add_plugins(CollisionDetectionPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(EnemyPlugin)
        .add_plugins(CollectablePlugin)
        .add_plugins(MovementPlugin)
        .add_plugins(MapPlugin)
        .add_plugins(CoinPlugin)
        .add_plugins(ClubPlugin)
        .add_plugins(BossPlugin)
        .add_plugins(DebugPlugin)
        .run();
}



