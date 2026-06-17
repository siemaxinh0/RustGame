use bevy::prelude::*;
use bevy::text::FontSmoothing;
use rand::Rng;

use crate::asset_loader::SceneAssets;
use crate::club::overlaps_club;
use crate::collision_handler::Collider;
use crate::map::MapBounds;
use crate::movement::Velocity;
use crate::player::Player;
use crate::state::GameState;

const COLLISION_RADIUS: f32 = 8.0;
// Taki sam interwał jak piwa w collectable.rs (jednakowy "ratio" spawnu).
const SPAWN_TIME_SECONDS: f32 = 5.0;
// Ile monet potrzeba, by ochroniarz wpuścił do klubu.
pub const REQUIRED_COINS: u32 = 10;

const COIN_COLOR: Color = Color::srgb(1.0, 0.84, 0.0);

#[derive(Component, Debug)]
pub struct Coin;

// Konto monet gracza - przeżywa wejście do klubu, zerowane przy nowej grze.
#[derive(Resource, Debug, Default)]
pub struct Wallet {
    pub coins: u32,
}

#[derive(Resource, Debug)]
struct CoinSpawnTimer {
    timer: Timer,
}

// Marker tekstu licznika monet w rogu ekranu.
#[derive(Component, Debug)]
struct CoinCounter;

pub struct CoinPlugin;

impl Plugin for CoinPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Wallet>()
            .insert_resource(CoinSpawnTimer {
                timer: Timer::from_seconds(SPAWN_TIME_SECONDS, TimerMode::Repeating),
            })
            .add_systems(OnEnter(GameState::Playing), spawn_coin_counter)
            .add_systems(
                Update,
                (spawn_coin, handle_coin_collision, cheat_coins, update_coin_counter)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

// Ukryty skrót testowy: klawisz C dorzuca od razu 10 monet (do testowania bossa).
fn cheat_coins(keyboard: Res<ButtonInput<KeyCode>>, mut wallet: ResMut<Wallet>) {
    if keyboard.just_pressed(KeyCode::KeyC) {
        wallet.coins += 10;
    }
}

fn spawn_coin(
    mut commands: Commands,
    mut spawn_timer: ResMut<CoinSpawnTimer>,
    time: Res<Time>,
    bounds: Res<MapBounds>,
    scene_assets: Res<SceneAssets>,
) {
    spawn_timer.timer.tick(time.delta());
    if !spawn_timer.timer.just_finished() {
        return;
    }

    let mut rng = rand::thread_rng();
    // Losuj poza bryłą budynku D17 (monety nie pojawiają się na budynku).
    let (mut x, mut y);
    loop {
        x = rng.gen_range(bounds.x_min..bounds.x_max);
        y = rng.gen_range(bounds.y_min..bounds.y_max);
        if !overlaps_club(Vec2::new(x, y), 6.0) {
            break;
        }
    }

    commands.spawn((
        Sprite {
            image: scene_assets.coin.clone(),
            custom_size: Some(Vec2::new(8.0, 8.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(x, y, -30.0)),
        Velocity { value: Vec3::ZERO },
        Collider::new(COLLISION_RADIUS),
        Coin,
    ));
}

fn handle_coin_collision(
    mut commands: Commands,
    coins: Query<(Entity, &Collider), With<Coin>>,
    players: Query<Entity, With<Player>>,
    mut wallet: ResMut<Wallet>,
) {
    let Ok(player_entity) = players.single() else {
        return;
    };

    for (coin_entity, collider) in coins.iter() {
        if collider.colliding_entities.contains(&player_entity) {
            wallet.coins += 1;
            commands.entity(coin_entity).despawn();
        }
    }
}

fn spawn_coin_counter(mut commands: Commands) {
    commands.spawn((
        DespawnOnExit(GameState::Playing),
        // Pełnoekranowy kontener kotwiczący licznik w lewym górnym rogu.
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::FlexStart,
            padding: UiRect::all(Val::Px(10.0)),
            ..default()
        },
        children![(
            // Pikselowe "okienko" licznika z grubą ramką.
            Node {
                padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                border: UiRect::all(Val::Px(3.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.08, 0.07, 0.05, 0.85)),
            BorderColor::all(Color::srgb(0.95, 0.74, 0.10)),
            children![(
                Text::new("MONETY: 0/10"),
                TextFont {
                    font_size: 22.0,
                    font_smoothing: FontSmoothing::None,
                    ..default()
                },
                TextColor(COIN_COLOR),
                CoinCounter,
            )],
        )],
    ));
}

fn update_coin_counter(wallet: Res<Wallet>, mut query: Query<&mut Text, With<CoinCounter>>) {
    for mut text in &mut query {
        text.0 = format!("MONETY: {}/{}", wallet.coins, REQUIRED_COINS);
    }
}
