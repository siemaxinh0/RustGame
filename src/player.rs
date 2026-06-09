use bevy::app::{App, Plugin,};
use bevy::math::Vec3;
use bevy::prelude::*;
use bevy::sprite::Sprite;
use crate::asset_loader::SceneAssets;
use crate::collision_handler::Collider;
use crate::movement::{atlas_index, direction_from_velocity, FacingDirection, Velocity};

const STARTING_TRANSLATION : Vec3 = Vec3::new(0.,0.,-20.);
const STARTING_VELOCITY : Vec3 = Vec3::new(0.,0.,0.);
const PLAYER_SPEED : f32 = 25.0;
const COLLISION_RADIUS : f32 = 8.0;


#[derive(Component,Debug)]
pub struct Player;

#[derive(Component, Debug, Clone, PartialEq)]
pub enum PlayerState {
    Idle,
    Moving
}
#[derive(Component)]
pub struct Immortal{
    pub timer: Timer
}

#[derive(Component, Debug)]
pub struct PlayerAnimation {
    pub timer: Timer,
    pub frame: usize,
}

impl PlayerAnimation {
    pub fn new() -> Self {
        Self {
            timer : Timer::from_seconds(0.12, TimerMode::Repeating),
            frame : 0
        }
    }
}


pub struct PlayerPlugin;

impl Plugin for PlayerPlugin{
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, spawn_player)
            .add_systems(Update,player_movement_controls)
            .add_systems(Update,animate_player)
            .add_systems(Update,update_immortal);
    }
}
fn update_immortal(mut commands: Commands,time: Res<Time>,mut query: Query<(Entity,&mut Immortal,&mut Sprite)>){
    for (entity, mut immortal,mut sprite) in query.iter_mut(){
        immortal.timer.tick(time.delta());

        if immortal.timer.fraction()%0.2<0.1{
            sprite.color=Color::srgb(1.0,0.8,0.0);
        }
        else{
            sprite.color=Color::WHITE;
        }

        if immortal.timer.just_finished(){
            sprite.color=Color::WHITE;
            commands.entity(entity).remove::<Immortal>();

        }


    }
}

fn spawn_player(mut commands: Commands, scene_assets : Res<SceneAssets>) {
    let mut sprite = Sprite::from_atlas_image(
        scene_assets.player.sheet.clone(),
        TextureAtlas {
            layout : scene_assets.player.layout.clone(),
            index : 3
        },
    );
    sprite.custom_size = Some(Vec2::new(16., 16.));

    let state = PlayerState::Idle;
    let direction = direction_from_velocity(STARTING_VELOCITY);

    commands.spawn((
        sprite,
        Transform::from_translation(STARTING_TRANSLATION),
        Velocity{ value:STARTING_VELOCITY },
        Collider ::new(COLLISION_RADIUS),
        Player,
        PlayerAnimation::new(),
        direction,
        state,
    ));
}

fn player_movement_controls(
    mut query: Query<(&mut Velocity, &mut FacingDirection, &mut PlayerState), With<Player>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    let Ok((mut velocity, mut direction, mut state)) = query.single_mut() else {
        return;
    };

    let mut movement_y = 0.0;
    let mut movement_x = 0.0;
    *state = PlayerState::Moving;

    if keyboard.pressed(KeyCode::KeyW) {
        movement_y = PLAYER_SPEED;
        *direction = FacingDirection::Up;
    }
    else if keyboard.pressed(KeyCode::KeyS) {
        movement_y = -PLAYER_SPEED;
        *direction = FacingDirection::Down;
    }
    else if keyboard.pressed(KeyCode::KeyA) {
        movement_x = -PLAYER_SPEED;
        *direction = FacingDirection::Left;
    }
    else if keyboard.pressed(KeyCode::KeyD) {
        movement_x = PLAYER_SPEED;
        *direction = FacingDirection::Right;
    }
    else {
        *state = PlayerState::Idle;
    }

    //update player velocity
    velocity.value = Vec3::new(movement_x, movement_y, 0.0);
}


fn animate_player(
    time: Res<Time>,
    mut query: Query<( &FacingDirection,&mut PlayerAnimation, &mut Sprite, &PlayerState ), With<Player>>
) {
    for (direction, mut animation,mut sprite, state) in query.iter_mut(){
        animation.timer.tick(time.delta());

        if animation.timer.just_finished() {

            if *state==PlayerState::Idle { //wyzeruj pozycje dla idle
                animation.frame = animation.frame - animation.frame % 4;
            }
            else {
                animation.frame = (animation.frame + 1) % 4;
            }

            if let Some(atlas) = &mut sprite.texture_atlas {
                atlas.index = atlas_index(direction, animation.frame);
            }
        }
    }
}