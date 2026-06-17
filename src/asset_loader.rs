use bevy::asset::Handle;
use bevy::prelude::*;
use bevy::audio::Volume;
use crate::state::GameState;

#[derive(Resource, Default)]
pub struct SpriteAssets {
    pub sheet: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>
}

#[derive(Resource, Default)]
pub struct SceneAssets {
    pub enemy: SpriteAssets,
    pub player: SpriteAssets,
    pub collectable: Handle<Image>,
    pub coin: Handle<Image>,
    pub bouncer: Handle<Image>,
    pub club_building: Handle<Image>,
    pub dean: Handle<Image>,
    pub sidewalk: Handle<Image>,
    pub fresh_grass: Handle<Image>,
    pub trampled_grass: Handle<Image>,
    pub dead_skull: Handle<Image>,
    pub background_music: Handle<AudioSource>,
    pub boss_music: Handle<AudioSource>,
}

// Marker encji odtwarzajacej muzyke zwyklej rozgrywki (pauzowana na czas walki z bossem).
#[derive(Component)]
pub struct MainMusic;

pub struct AssetLoaderPlugin;

impl Plugin for AssetLoaderPlugin{
    fn build(&self, app: &mut App) {
        app.init_resource::<SceneAssets>()
            .add_systems(Startup, load_assets)
            .add_systems(Update, check_assets_ready.run_if(in_state(GameState::Loading)));
    }
}

fn load_assets(
    mut scene_assets: ResMut<SceneAssets>,
    asset_server: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut commands: Commands,
){
    let player_layout = atlas_layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(64, 64),
        4,
        4,
        None,
        None,
    ));

    let enemy_layout = atlas_layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(64, 64),
        4,
        4,
        None,
        None,
    ));
    let music_handle: Handle<AudioSource> = asset_server.load("music_mainmap.mp3");
    let boss_music_handle: Handle<AudioSource> = asset_server.load("music_boss.mp3");

    *scene_assets = SceneAssets {
        player : SpriteAssets {
            sheet : asset_server.load("player_sheet.png"),
            layout : player_layout
        },

        enemy : SpriteAssets {
            sheet : asset_server.load("enemy_sheet.png"),
            layout : enemy_layout
        },
        collectable : asset_server.load("collectable.png"),
        // Placeholdery - podmien plik w assets/, aby zmienic wyglad.
        coin : asset_server.load("coin.png"),
        bouncer : asset_server.load("bouncer.png"),
        club_building : asset_server.load("club_d17.png"),
        dean : asset_server.load("dean.png"),
        sidewalk: asset_server.load("sidewalk.png"),
        trampled_grass: asset_server.load("soil.png"),
        fresh_grass: asset_server.load("grass.png"),
        dead_skull: asset_server.load("skull.png"),
        background_music: music_handle.clone(),
        boss_music: boss_music_handle,
    };

    commands.spawn((
        AudioPlayer::new(music_handle),
        // Zmiana z Volume::new na Volume::Linear
        PlaybackSettings::LOOP.with_volume(Volume::Linear(0.2)),
        MainMusic,
    ));
}


fn check_assets_ready(
    asset_server: Res<AssetServer>,
    scene_assets: Res<SceneAssets>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let ready = asset_server.is_loaded_with_dependencies(&scene_assets.player.sheet)
        && asset_server.is_loaded_with_dependencies(&scene_assets.enemy.sheet)
        && asset_server.is_loaded_with_dependencies(&scene_assets.collectable)
        && asset_server.is_loaded_with_dependencies(&scene_assets.coin)
        && asset_server.is_loaded_with_dependencies(&scene_assets.bouncer)
        && asset_server.is_loaded_with_dependencies(&scene_assets.club_building)
        && asset_server.is_loaded_with_dependencies(&scene_assets.dean)
        && asset_server.is_loaded_with_dependencies(&scene_assets.sidewalk)
        && asset_server.is_loaded_with_dependencies(&scene_assets.fresh_grass)
        && asset_server.is_loaded_with_dependencies(&scene_assets.trampled_grass)
        && asset_server.is_loaded_with_dependencies(&scene_assets.dead_skull);

    if ready {
        next_state.set(GameState::Playing);
    }
}