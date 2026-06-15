use bevy::prelude::*;
use rand::Rng;
use crate::asset_loader::SceneAssets;
use crate::collision_handler::Collider;
use crate::map::MapBounds;
use crate::movement::Velocity;
use crate::player::{Immortal, Player};

const COLLISION_RADIUS : f32 = 8.0;
const SPAWN_TIME_SECONDS : f32 = 5.0;
#[derive(Component,Debug)]
pub struct Collectable;

#[derive(Resource,Debug)]
pub struct SpawnTimer{
    pub timer: Timer,
}

pub struct CollectablePlugin;

impl Plugin for CollectablePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SpawnTimer {
            timer : Timer::from_seconds(SPAWN_TIME_SECONDS, TimerMode::Repeating),
        })
            .add_systems(Update, spawn_collectable)
            .add_systems(Update, handle_collectable_collision);
    }
}

fn spawn_collectable(mut commands : Commands,
                     mut spawn_timer: ResMut<SpawnTimer>,
                     time: Res<Time>,
                     scene_assets: Res<SceneAssets>,
                     bounds : Res<MapBounds>) {
    spawn_timer.timer.tick(time.delta());
    if !spawn_timer.timer.just_finished(){
        return;
    }

    let mut rng = rand::thread_rng();

    let range_x = bounds.x_min..bounds.x_max;
    let range_y = bounds.y_min..bounds.y_max;

    let translation = Vec3::new(rng.gen_range(range_x), rng.gen_range(range_y), -30.);

    commands.spawn((
        Sprite {
            image: scene_assets.collectable.clone(),
            custom_size: Some(Vec2::new(8., 8.)),
            ..default()
        },
        Transform::from_translation(translation),
        Velocity{value : Vec3::new(0.,0.,0.,)},
        Collider::new(COLLISION_RADIUS),
        Collectable
        ));
}

fn handle_collectable_collision(mut commands: Commands,
                                collectables: Query<(Entity, &Collider), With<Collectable>>,
                                players : Query<Entity, With<Player>>
){

    let Ok(player_entity) = players.single() else { return; };

    for (collectable_entity, collider) in collectables.iter() {
       if collider.colliding_entities.contains(&player_entity){
           commands.entity(player_entity).insert(Immortal{
               timer: Timer::from_seconds(3.0,TimerMode::Once)

           });
           commands.entity(collectable_entity).despawn();

       }
    }
}