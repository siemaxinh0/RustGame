use crate::asset_loader::SceneAssets;
use crate::player::Player;
use bevy::prelude::*;
use crate::movement::{Velocity};
use crate::state::{in_gameplay, GameResult, GameState};

#[derive(Component)]
pub struct Sidewalk;

#[derive(Component)]
pub struct FreshGrass;

#[derive(Component)]
pub struct TrampledGrass;

const TILE_SIZE: f32 = 14.0;
const BUILDING_TILE_W: f32 = 3.*TILE_SIZE;
const BUILDING_TILE_H: f32 = 4.*TILE_SIZE;

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

fn draw_map_borders(mut commands: Commands, scene_assets: Res<SceneAssets>) {
    let ratio = BUILDING_TILE_W / TILE_SIZE; // = 3.0

    // Ile kafelków budynku pokrywa szerokość/wysokość mapy (zaokrąglamy w górę)
    let map_cols_in_buildings = (MAP_LAYOUT[0].len() as f32 / ratio).ceil() as i32;
    let map_rows_in_buildings = (MAP_LAYOUT.len() as f32 / ratio).ceil() as i32 + 1;

    // Cała siatka: mapa + 1 kafelek budynku z każdej strony
    let total_cols = map_cols_in_buildings + 2;
    let total_rows = map_rows_in_buildings + 2;

    // Środek lewego górnego kafelka obramowania jest przesunięty, aby wziąć pod uwagę cień
    let start_x = -(MAP_WIDTH / 2.0) - (BUILDING_TILE_W / 2.0);
    let start_y =  (MAP_HEIGHT / 2.0) + (BUILDING_TILE_H / 6.0);

    for row in 0..total_rows {
        for col in 0..total_cols {
            // Pomiń wnętrze (obszar należący do mapy)
            if row >= 1 && row <= map_rows_in_buildings
                && col >= 1 && col <= map_cols_in_buildings {
                continue;
            }
            let mut image = scene_assets.building_2.clone();

            if (row==0 || row ==total_rows-1) && col!=0 && col!=total_cols-1 {
                image = scene_assets.building_1.clone();
            }

            let x = start_x + col as f32 * BUILDING_TILE_W;
            let y = start_y - row as f32 * 0.5*BUILDING_TILE_H;

            commands.spawn((
                // Domki znikają po wyjściu z głównej mapy (np. wejście do D17/walka z bossem)
                // i są odtwarzane przy powrocie do stanu Playing.
                DespawnOnExit(GameState::Playing),
                Sprite {
                    image,
                    custom_size: Some(Vec2::new(BUILDING_TILE_W, BUILDING_TILE_H)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(x, y, 20.0)),
            ));
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
            .add_systems(OnEnter(GameState::Playing), (draw_map,draw_map_borders))
            .add_systems(
                Update,
                (trample_grass, check_win_state).run_if(in_state(GameState::Playing)),
            )
            .add_systems(Update, enforce_map_bounds.run_if(in_gameplay));
    }
}