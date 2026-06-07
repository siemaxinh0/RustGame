use crate::asset_loader::SceneAssets;
use crate::player::Player;
use bevy::prelude::*;

#[derive(Component)]
pub struct Sidewalk;

#[derive(Component)]
pub struct FreshGrass;

#[derive(Component)]
pub struct TrampledGrass;

const TILE_SIZE: f32 = 16.0;

const MAP_LAYOUT: [&str; 8] = [
    "########", "#......#", "#.####.#", "#.#..#.#", "#.#..#.#", "#.####.#", "#......#", "########",
];

fn spawn_map(mut commands: Commands, scene_assets: Res<SceneAssets>) {
    let map_width=MAP_LAYOUT[0].len() as f32 * TILE_SIZE;
    let map_height=MAP_LAYOUT.len() as f32 * TILE_SIZE;

    let start_x=-(map_width/2.0)+ (TILE_SIZE /2.0);
    let start_y=(map_height /2.0)- (TILE_SIZE/ 2.0);

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
fn check_win_state(mut commands: Commands,fresh_grass_query: Query<(), With<FreshGrass>>,mut has_won: Local<bool>){
    if *has_won{
        return;
    }
    if fresh_grass_query.is_empty(){
        *has_won=true;
        info!("WYGRANA");

    commands.spawn((
        Text2d::new("WYGRANA!!!"),
        TextFont{
            font_size:32.0,
            ..default()
        },
        TextColor(Color::srgb(1.0,0.8,0.0)),
        Transform::from_translation(Vec3::new(0.0,0.0,100.0))
        ));
    }
}

pub struct MapPlugin;

impl Plugin for MapPlugin{
    fn build(&self,app: &mut App){
        app.add_systems(PostStartup,spawn_map)
            .add_systems(Update,(trample_grass, check_win_state));
    }
}