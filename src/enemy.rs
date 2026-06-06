use std::ops::Range;
use bevy::prelude::*;
use rand::{ Rng};
use crate::asset_loader::SceneAssets;
use crate::collision_handler::Collider;
use crate::movement::{atlas_index, direction_from_velocity, FacingDirection, Velocity};
use crate::player::Player;

const VELOCITY_SCALAR : f32 = 10.;
const SPAWN_RANGE_X : Range<f32> = -100.0..100.0;
const SPAWN_RANGE_Y: Range<f32> = -100.0..100.0;
const SPAWN_TIME_SECONDS : f32 = 1.0;
const COLLISION_RADIUS : f32 = 16.0;

#[derive(Component, Debug)]
pub struct Enemy;

#[derive(Resource,Debug)]
pub struct SpawnTimer{
    pub timer: Timer,
}

#[derive(Component, Debug)]
pub struct EnemyAnimation {
    pub timer: Timer,
    pub frame: usize,
}

impl EnemyAnimation {
    pub fn new() -> Self {
        Self {
            timer : Timer::from_seconds(0.2, TimerMode::Repeating),
            frame : 0
        }
    }
}


pub struct EnemyPlugin;

impl Plugin for EnemyPlugin{
    fn build(&self, app: &mut App) {
        app.insert_resource(SpawnTimer {
            timer: Timer::from_seconds(SPAWN_TIME_SECONDS, TimerMode::Repeating),
        })
            .add_systems(Update, spawn_enemy)
            .add_systems(Update, animate_enemy)
            .add_systems(Update, handle_enemy_collision);
    }
}


fn spawn_enemy(mut commands: Commands, mut spawn_timer : ResMut<SpawnTimer>, time: Res<Time>, scene_assets: Res<SceneAssets>){
    spawn_timer.timer.tick(time.delta());
    if !spawn_timer.timer.just_finished(){
        return;
    }

    let mut rng = rand::thread_rng();

    let translation = Vec3::new(rng.gen_range(SPAWN_RANGE_X), rng.gen_range(SPAWN_RANGE_Y),0.);

    let velocity = random_unit_vector() * VELOCITY_SCALAR;
    let direction = direction_from_velocity(velocity);

    let mut sprite = Sprite ::from_atlas_image(
        scene_assets.enemy.sheet.clone(),
        TextureAtlas {
            layout: scene_assets.enemy.layout.clone(),
            index : atlas_index(&direction,0)
        }
    );
    sprite.custom_size = Some(Vec2::new(16., 16.));

    commands.spawn((
        sprite,
        Transform::from_translation(translation),
        Velocity{value: velocity},
        Collider::new(COLLISION_RADIUS),
        Enemy,
        direction,
        EnemyAnimation::new()
        ));
}

fn handle_enemy_collision(mut commands : Commands,
                          enemies: Query<(Entity, &Collider), With<Enemy>>,
                          players: Query<(), With<Player>>
) {
    for (enemy_entity, collider) in enemies.iter(){
        let hit_player = collider
            .colliding_entities
            .iter()
            .any(|&collided_entity| players.get(collided_entity).is_ok());

        if hit_player {
            //TODO GAME OVER
            commands.entity(enemy_entity).despawn();
        };
    }
}

fn animate_enemy(
    time: Res<Time>,
    mut query: Query<( &FacingDirection,&mut EnemyAnimation, &mut Sprite ), With<Enemy>>
) {
    for (direction, mut animation,mut sprite) in query.iter_mut(){
        animation.timer.tick(time.delta());

        if animation.timer.just_finished() {
            animation.frame = (animation.frame + 1) % 4;

            if let Some(atlas) = &mut sprite.texture_atlas {
                atlas.index = atlas_index(direction, animation.frame);
            }
        }
    }
}



fn random_unit_vector() -> Vec3{
    let mut rng = rand::thread_rng();
    let number = rng.gen_range(0.0..1.0);
    match number {
        0.0..0.25 => Vec3::new(1.,0.,0.),
        0.25..0.5 => Vec3::new(0.,-1.,0.),
        0.5..0.75 => Vec3::new(-1.,0.,0.),
        _ => Vec3::new(0.,1.,0.)
    }
}