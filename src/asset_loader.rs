use bevy::asset::Handle;
use bevy::prelude::*;

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
    pub sidewalk: Handle<Image>,
    pub fresh_grass: Handle<Image>,
    pub trampled_grass: Handle<Image>,
    pub dead_skull: Handle<Image>
}



pub struct AssetLoaderPlugin;

impl Plugin for AssetLoaderPlugin{
    fn build(&self, app: &mut App) {
        app.init_resource::<SceneAssets>().add_systems(Startup,load_assets);
    }
}

fn load_assets(mut scene_assets: ResMut<SceneAssets>, asset_server : Res<AssetServer>, mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>){
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
        sidewalk: asset_server.load("sidewalk.png"),
        trampled_grass: asset_server.load("soil.png"),
        fresh_grass: asset_server.load("grass.png"),
        dead_skull:asset_server.load("skull.png")

    }
}