use crate::asset_loader::SceneAssets;
use crate::player::Player;
use bevy::prelude::*;
use crate::movement::{Velocity};
use crate::state::{GameResult, GameState};

#[derive(Component)]
pub struct Sidewalk;

#[derive(Component)]
pub struct FreshGrass;

#[derive(Component)]
pub struct TrampledGrass;

const TILE_SIZE: f32 = 18.0;

const MAP_LAYOUT: [&str; 10] = [
"##################",
"###............###",
"###..########..###",
"###..##....##..###",
"###..##....##..###",
"###..##....##..###",
"###..##....##..###",
"###..########..###",
"###............###",
"##################"
];

#[derive(Resource)]
pub struct MapBounds {
    pub x_min: f32,
    pub y_min: f32,
    pub x_max: f32,
    pub y_max: f32,
}

const MAP_WIDTH : f32 = MAP_LAYOUT[0].len() as f32 * TILE_SIZE;
const MAP_HEIGHT : f32=MAP_LAYOUT.len() as f32 * TILE_SIZE;


fn draw_map(mut commands: Commands, scene_assets: Res<SceneAssets>) {


    let start_x=-(MAP_WIDTH/2.0)+ (TILE_SIZE /2.0);
    let start_y=(MAP_HEIGHT/2.0)- (TILE_SIZE/ 2.0);

    for (row_idx, row) in MAP_LAYOUT.iter().enumerate() {
        for (col_idx, tile_char) in row.chars().enumerate() {
            let x = start_x + (col_idx as f32 * TILE_SIZE);
            let y = start_y - (row_idx as f32 * TILE_SIZE);
            let translation = Vec3::new(x, y, -50.00);

            match tile_char {
                '#' => {
                    commands.spawn((
                        Sprite {
                            image: scene_assets.sidewalk.clone(),
                            custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                            ..default()
                        },
                        Transform::from_translation(translation),
                        Sidewalk,
                    ));
                }
                '.' => {
                    commands.spawn((
                        Sprite {
                            image: scene_assets.fresh_grass.clone(),
                            custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                            ..default()
                        },
                        Transform::from_translation(translation),
                        FreshGrass,
                    ));
                }
                _ => {}
            }
        }
    }
}
fn trample_grass(mut commands: Commands,
                 player_query: Query<&Transform,With<Player>>,
                 mut grass_query: Query<(Entity,&Transform, &mut Sprite),With<FreshGrass>>,
                 scene_assets: Res<SceneAssets>,

){
    let Ok(player_transform)=player_query.single() else {return;};

    let trample_radius=10.0;

    let player_pos_2d=player_transform.translation.truncate();


    for (entity,grass_transform, mut sprite)in grass_query.iter_mut(){
        let grass_pos_2d=grass_transform.translation.truncate();
        let distance=player_pos_2d.distance(grass_pos_2d);

        if distance<trample_radius{
            sprite.image=scene_assets.trampled_grass.clone();
            commands.entity(entity).remove::<FreshGrass>().insert(TrampledGrass);
        }
    }

}
fn check_win_state(
    fresh_grass_query: Query<(), With<FreshGrass>>,
    mut game_result: ResMut<GameResult>,
    mut next_state: ResMut<NextState<GameState>>,
){
    if fresh_grass_query.is_empty(){
        info!("WYGRANA");
        game_result.won = true;
        next_state.set(GameState::GameOver);
    }
}


fn enforce_map_bounds(mut query: Query<(&mut Transform, &mut Sprite), With<Velocity>>, bounds: Res<MapBounds>,) {
    for (mut transform, sprite) in query.iter_mut() {

        let sprite_size = sprite.custom_size.unwrap_or(Vec2::new(32.0, 32.0));

        let half_width = sprite_size.x / 2.0;
        let half_height = sprite_size.y / 2.0;

        let clamped_x = transform.translation.x.clamp(
            bounds.x_min + half_width,
            bounds.x_max - half_width,
        );

        let clamped_y = transform.translation.y.clamp(
            bounds.y_min + half_height,
            bounds.y_max - half_height,
        );

        transform.translation.x = clamped_x;
        transform.translation.y = clamped_y;
    }
}


pub struct MapPlugin;

impl Plugin for MapPlugin{
    fn build(&self,app: &mut App){
        app
            .insert_resource(MapBounds { 
                x_min:-MAP_WIDTH/2., 
                y_min:-MAP_HEIGHT/2.,
                x_max : MAP_WIDTH/2.,
                y_max : MAP_HEIGHT/2.
            })
            .add_systems(OnEnter(GameState::Playing), draw_map)
            .add_systems(
                Update,
                (trample_grass, check_win_state, enforce_map_bounds)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}