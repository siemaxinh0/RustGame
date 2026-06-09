use bevy::prelude::*;
use rand::{ Rng};
use crate::asset_loader::SceneAssets;
use crate::collision_handler::Collider;
use crate::map::MapBounds;
use crate::movement::{atlas_index, direction_from_velocity, FacingDirection, Velocity};
use crate::player::Player;

const VELOCITY_SCALAR : f32 = 10.;
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
            .add_systems(Update, handle_enemy_collision)
            .add_systems(Update,despawn_on_map_border);
    }
}


fn spawn_enemy(mut commands: Commands,
               mut spawn_timer : ResMut<SpawnTimer>, 
               time: Res<Time>, scene_assets: Res<SceneAssets>,
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


    let mut rng = rand::thread_rng();

    let half_width = sprite_size.x / 2.0;
    let half_height = sprite_size.y / 2.0;

    let range_x = bounds.x_min+half_width..bounds.x_max-half_width;
    let range_y = bounds.y_min+half_height..bounds.y_max-half_height;

    let translation = Vec3::new(rng.gen_range(range_x), rng.gen_range(range_y),0.);

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
                          players: Query<(Entity,&Transform), With<Player>>,
                          mut has_lost: Local<bool>,
                          scene_assets: Res<SceneAssets>,
) {
    if *has_lost{
        return;
    }
    let Ok((player_entity, player_transform)) = players.single() else { return; };
    for (_enemy_entity, collider) in enemies.iter(){
        let hit_player = collider
            .colliding_entities
            .iter()
            .any(|&collided_entity| players.get(collided_entity).is_ok());

        if hit_player {
            *has_lost=true;
            info!("Jaguar cie dopadl");
            let death_position = player_transform.translation;
            commands.spawn((
                Sprite {
                    image: scene_assets.dead_skull.clone(),
                    custom_size: Some(Vec2::new(16.0, 16.0)),
                    ..default()
                },

                Transform::from_translation(Vec3::new(death_position.x, death_position.y, 100.0)),
            ));

            commands.spawn((
                Text2d::new("PRZEGRANA!!"),
                TextFont{
                    font_size: 32.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0,0.0,0.0)),
                Transform::from_translation(Vec3::new(0.0,0.0,105.0))
            ));
            commands.entity(player_entity).despawn();

            // for (enemy, _collider) in enemies.iter_mut() {
            //     commands.entity(enemy).despawn();
            // }

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
            enemy_transform.translation.y <= bounds.y_min + half_height
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