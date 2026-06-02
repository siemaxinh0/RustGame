use bevy::app::{App, Plugin,};
use bevy::asset::AssetServer;
use bevy::math::Vec3;
use bevy::prelude::*;
use bevy::sprite::Sprite;
use crate::movement::Velocity;

const STARTING_TRANSLATION : Vec3 = Vec3::new(0.,0.,-20.);
const STARTING_VELOCITY : Vec3 = Vec3::new(0.,0.,0.);
const SPACESHIP_SPEED : f32 = 25.0;

#[derive(Component,Debug)]
pub struct Spaceship;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin{
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, spawn_player)
            .add_systems(Update,player_movement_controls);
    }
}

fn spawn_player(mut commands: Commands, assets_server : Res<AssetServer>) {
    commands.spawn((
        Sprite {
            image: assets_server.load("imperator.png"),
            custom_size: Some(Vec2::new(32., 32.)),
            ..default()
        },
        Transform::from_translation(STARTING_TRANSLATION),
        Velocity{ value:STARTING_VELOCITY },
        Spaceship,
    ));
}

fn player_movement_controls(
    mut query: Query<&mut Velocity, With<Spaceship>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    let Ok(mut velocity) = query.single_mut() else {
        return;
    };
    let mut movement_y = 0.0;
    let mut movement_x = 0.0;

    if keyboard.pressed(KeyCode::KeyW) {
        movement_y = SPACESHIP_SPEED;
    }
    else if keyboard.pressed(KeyCode::KeyS) {
        movement_y = -SPACESHIP_SPEED;
    }
    else if keyboard.pressed(KeyCode::KeyA) {
        movement_x = -SPACESHIP_SPEED;
    }
    else if keyboard.pressed(KeyCode::KeyD) {
        movement_x = SPACESHIP_SPEED;
    }

    //update player velocity
    velocity.value = Vec3::new(movement_x, movement_y, 0.0);
}
