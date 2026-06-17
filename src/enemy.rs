use bevy::math::ops::abs;
use bevy::prelude::*;
use rand::{ Rng};
use crate::asset_loader::SceneAssets;
use crate::club::overlaps_club;
use crate::collision_handler::Collider;
use crate::map::MapBounds;
use crate::movement::{atlas_index, direction_from_velocity, FacingDirection, Velocity};
use crate::player::{Immortal, Player};
use crate::state::{DeathMarker, GameResult, GameState};

const VELOCITY_SCALAR : f32 = 10.;
const SPAWN_TIME_SECONDS : f32 = 1.0;
const COLLISION_RADIUS : f32 = 16.0;
const PLAYER_SAFE_RANGE : f32 = 20.;

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
            .add_systems(Update, spawn_enemy.run_if(in_state(GameState::Playing)))
            .add_systems(Update, animate_enemy.run_if(in_state(GameState::Playing)))
            .add_systems(Update, handle_enemy_collision.run_if(in_state(GameState::Playing)))
            .add_systems(Update, despawn_on_map_border.run_if(in_state(GameState::Playing)));
    }
}


fn spawn_enemy(mut commands: Commands,
               mut spawn_timer : ResMut<SpawnTimer>, 
               time: Res<Time>, scene_assets: Res<SceneAssets>,
               players: Query<&Transform, With<Player>>,
               bounds : Res<MapBounds>
){
    spawn_timer.timer.tick(time.delta());
    if !spawn_timer.timer.just_finished(){
        return;
    }
    
    let velocity = random_unit_vector() * VELOCITY_SCALAR;
    let direction = direction_from_velocity(velocity);

    let mut sprite = Sprite ::from_atlas_image(
        scene_assets.enemy.sheet.clone(),
        TextureAtlas {
            layout: scene_assets.enemy.layout.clone(),
            index : atlas_index(&direction,0)
        }
    );
    let sprite_size = Vec2::new(16., 16.);
    sprite.custom_size = Some(sprite_size);

    let Ok( player_transform) = players.single() else { return; };

    let translation = player_safe_spawn_position(player_transform.translation,bounds,sprite_size);

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
                          players: Query<(Entity,&Transform,Option<&Immortal>), With<Player>>,
                          scene_assets: Res<SceneAssets>,
                          mut game_result: ResMut<GameResult>,
                          mut next_state: ResMut<NextState<GameState>>,
) {
    let Ok((player_entity, player_transform,is_immortal)) = players.single() else { return; };
    for (_enemy_entity, collider) in enemies.iter(){
        let hit_player = collider
            .colliding_entities
            .iter()
            .any(|&collided_entity| players.get(collided_entity).is_ok());

        if hit_player {
            if is_immortal.is_some(){
                continue;
            }
            info!("Jaguar cie dopadl");
            let death_position = player_transform.translation;
            commands.spawn((
                Sprite {
                    image: scene_assets.dead_skull.clone(),
                    custom_size: Some(Vec2::new(16.0, 16.0)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(death_position.x, death_position.y, 100.0)),
                DeathMarker,
            ));

            commands.entity(player_entity).despawn();

            game_result.won = false;
            next_state.set(GameState::GameOver);

            break;
        };
    }
}


//TODO enemy despwan ob map border collision
fn despawn_on_map_border(mut commands: Commands,
                         mut enemies: Query<(Entity, &mut Transform, &mut Sprite), With<Enemy>>,
                         bounds: Res<MapBounds>){
    for (enemy, enemy_transform, sprite) in enemies.iter_mut(){

        let sprite_size = sprite.custom_size.unwrap_or(Vec2::new(32.0, 32.0));

        let half_width = sprite_size.x / 2.0;
        let half_height = sprite_size.y / 2.0;

        if enemy_transform.translation.x >= bounds.x_max  - half_width||
            enemy_transform.translation.x <= bounds.x_min + half_width ||
            enemy_transform.translation.y >= bounds.y_max - half_height||
            enemy_transform.translation.y <= bounds.y_min + half_height ||
            // Przeciwnik wszedl w obreb budynku D17 - despawn (jak na krancu mapy).
            overlaps_club(enemy_transform.translation.truncate(), half_width.min(half_height))
        {
            commands.entity(enemy).despawn();
        }
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

fn player_safe_spawn_position(player_translation: Vec3, bounds : Res<MapBounds>, sprite_size: Vec2) -> Vec3 {

    let mut rng = rand::thread_rng();

    let half_width = sprite_size.x / 2.0;
    let half_height = sprite_size.y / 2.0;


    let range_x = bounds.x_min+half_width..bounds.x_max-half_width;
    let range_y = bounds.y_min+half_height..bounds.y_max-half_height;

    let mut spawn_x : f32 =rng.gen_range(range_x.clone());
    let mut spawn_y : f32 = rng.gen_range(range_y.clone());

    //not deterministic but acceptable for our project scale
    while abs(spawn_x - player_translation.x )< PLAYER_SAFE_RANGE{
        spawn_x =rng.gen_range(range_x.clone());
    }

    while abs(spawn_y - player_translation.y )< PLAYER_SAFE_RANGE{
        spawn_y =rng.gen_range(range_y.clone());
    }
    Vec3::new(spawn_x,spawn_y ,0.)
}
