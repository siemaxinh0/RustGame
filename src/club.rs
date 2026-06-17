use bevy::prelude::*;
use bevy::text::FontSmoothing;

use crate::asset_loader::SceneAssets;
use crate::coin::{Wallet, REQUIRED_COINS};
use crate::player::Player;
use crate::state::GameState;

// Zasięg, z którego można "zagadać" do ochroniarza.
const INTERACT_RANGE: f32 = 24.0;

// Środek bryły budynku D17 (przy górnej krawędzi mapy).
// >> Pozycję budynku na mapie zmieniasz tutaj (X, Y).
pub const CLUB_CENTER: Vec2 = Vec2::new(0.0, 64.0);
// Połowa rozmiaru bryły budynku - dopasowana do proporcji sprite'a club_d17.png
// (667x374 ~ 1.78), kolizja + brak spawnów monet/piw na budynku.
// >> Rozmiar budynku zmieniasz tutaj (to POŁOWA szerokości i wysokości).
pub const CLUB_HALF: Vec2 = Vec2::new(51.0, 28.6);

// Ochroniarz stoi tuż przy wejściu (dolna krawędź budynku), na środku.
// >> Pozycję ochroniarza dostrajasz tu: zwiększ ostatni dodawany offset (+8.0),
//    aby przesunąć go bardziej DO GÓRY, w stronę budynku.
const BOUNCER_POS: Vec3 = Vec3::new(0.0, CLUB_CENTER.y - CLUB_HALF.y + 1.0, 45.0);
// Rozmiar ochroniarza - taki sam jak przeciwnicy (16x16).
const BOUNCER_SIZE: f32 = 16.0;
// Połowa prostokąta kolizji ochroniarza.
const BOUNCER_HALF: f32 = BOUNCER_SIZE / 2.0;

#[derive(Component, Debug)]
pub struct Club;

#[derive(Component, Debug)]
pub struct Bouncer;

// Pokemonowy dymek dialogowy na dole ekranu.
#[derive(Component, Debug)]
struct DialogBox;

#[derive(Component, Debug)]
struct DialogText;

// Czy punkt (z marginesem promienia) koliduje z bryłą budynku - używane też
// do blokowania spawnu monet i piw na budynku.
pub fn overlaps_club(pos: Vec2, radius: f32) -> bool {
    (pos.x - CLUB_CENTER.x).abs() < CLUB_HALF.x + radius
        && (pos.y - CLUB_CENTER.y).abs() < CLUB_HALF.y + radius
}

pub struct ClubPlugin;

impl Plugin for ClubPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_club)
            .add_systems(
                Update,
                (block_player_into_club, block_player_into_bouncer, bouncer_interaction)
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

fn spawn_club(mut commands: Commands, scene_assets: Res<SceneAssets>) {
    let width = CLUB_HALF.x * 2.0;
    let height = CLUB_HALF.y * 2.0;
    let center = CLUB_CENTER.extend(40.0);

    // Budynek D17 jako sprite (napis "D17" jest już częścią tekstury).
    commands.spawn((
        DespawnOnExit(GameState::Playing),
        Club,
        Sprite {
            image: scene_assets.club_building.clone(),
            custom_size: Some(Vec2::new(width, height)),
            ..default()
        },
        Transform::from_translation(center),
    ));

    // Ochroniarz przed wejściem - rozmiar jak przeciwnicy (16x16), placeholder.
    commands.spawn((
        DespawnOnExit(GameState::Playing),
        Bouncer,
        Sprite {
            image: scene_assets.bouncer.clone(),
            custom_size: Some(Vec2::new(BOUNCER_SIZE, BOUNCER_SIZE)),
            ..default()
        },
        Transform::from_translation(BOUNCER_POS),
    ));
}

// Wypycha gracza poza bryłę budynku - budynek jest przeszkodą, nie tylko dekoracją.
fn block_player_into_club(mut players: Query<&mut Transform, With<Player>>) {
    let Ok(mut transform) = players.single_mut() else {
        return;
    };

    const PLAYER_RADIUS: f32 = 8.0;
    let half_x = CLUB_HALF.x + PLAYER_RADIUS;
    let half_y = CLUB_HALF.y + PLAYER_RADIUS;

    let dx = transform.translation.x - CLUB_CENTER.x;
    let dy = transform.translation.y - CLUB_CENTER.y;

    // Gracz wewnątrz prostokąta kolizji - wypchnij po osi mniejszego zagłębienia.
    if dx.abs() < half_x && dy.abs() < half_y {
        let push_x = half_x - dx.abs();
        let push_y = half_y - dy.abs();

        if push_x < push_y {
            transform.translation.x = CLUB_CENTER.x + dx.signum() * half_x;
        } else {
            transform.translation.y = CLUB_CENTER.y + dy.signum() * half_y;
        }
    }
}

// Ochroniarz blokuje gracza, dopóki nie zbierze wymaganej liczby monet.
fn block_player_into_bouncer(
    wallet: Res<Wallet>,
    bouncers: Query<&Transform, (With<Bouncer>, Without<Player>)>,
    mut players: Query<&mut Transform, With<Player>>,
) {
    // Mając dość monet, ochroniarz przepuszcza gracza do środka.
    if wallet.coins >= REQUIRED_COINS {
        return;
    }

    let Ok(bouncer) = bouncers.single() else {
        return;
    };
    let Ok(mut player) = players.single_mut() else {
        return;
    };

    const PLAYER_RADIUS: f32 = 8.0;
    let half = BOUNCER_HALF + PLAYER_RADIUS;

    let dx = player.translation.x - bouncer.translation.x;
    let dy = player.translation.y - bouncer.translation.y;

    // Gracz w prostokącie kolizji ochroniarza - wypchnij po osi mniejszego zagłębienia.
    if dx.abs() < half && dy.abs() < half {
        let push_x = half - dx.abs();
        let push_y = half - dy.abs();

        if push_x < push_y {
            player.translation.x = bouncer.translation.x + dx.signum() * half;
        } else {
            player.translation.y = bouncer.translation.y + dy.signum() * half;
        }
    }
}

fn dialog_message(coins: u32) -> String {
    format!(
        "OCHRONIARZ: Spokojnie... Masz {coins}/{REQUIRED_COINS} monet.\nWroc, gdy uzbierasz dosc kasy na wejscie do D17!"
    )
}

fn bouncer_interaction(
    mut commands: Commands,
    players: Query<&Transform, With<Player>>,
    bouncers: Query<&Transform, With<Bouncer>>,
    wallet: Res<Wallet>,
    dialog: Query<Entity, With<DialogBox>>,
    mut dialog_text: Query<&mut Text, With<DialogText>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let Ok(player) = players.single() else {
        return;
    };
    let Ok(bouncer) = bouncers.single() else {
        return;
    };

    let distance = player
        .translation
        .truncate()
        .distance(bouncer.translation.truncate());

    if distance < INTERACT_RANGE {
        if wallet.coins >= REQUIRED_COINS {
            // Wpuszczamy do klubu - start walki z bossem.
            for entity in &dialog {
                commands.entity(entity).despawn();
            }
            next_state.set(GameState::BossFight);
        } else if dialog.is_empty() {
            spawn_dialog(&mut commands, wallet.coins);
        } else {
            // Aktualizuj liczbę monet, gdyby zmieniła się przy ochroniarzu.
            for mut text in &mut dialog_text {
                text.0 = dialog_message(wallet.coins);
            }
        }
    } else {
        // Gracz odszedł - schowaj dymek.
        for entity in &dialog {
            commands.entity(entity).despawn();
        }
    }
}

fn spawn_dialog(commands: &mut Commands, coins: u32) {
    commands.spawn((
        DespawnOnExit(GameState::Playing),
        DialogBox,
        // Kontener kotwiczący dymek przy dolnej krawędzi ekranu.
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::FlexEnd,
            padding: UiRect::bottom(Val::Px(24.0)),
            ..default()
        },
        children![(
            // Pokemonowy box: kremowe tło, gruba ciemna ramka, ostre rogi.
            Node {
                width: Val::Px(560.0),
                padding: UiRect::axes(Val::Px(22.0), Val::Px(18.0)),
                border: UiRect::all(Val::Px(5.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.96, 0.94, 0.86)),
            BorderColor::all(Color::srgb(0.15, 0.13, 0.20)),
            children![(
                Text::new(dialog_message(coins)),
                TextFont {
                    font_size: 22.0,
                    font_smoothing: FontSmoothing::None,
                    ..default()
                },
                TextColor(Color::srgb(0.12, 0.10, 0.16)),
                DialogText,
            )],
        )],
    ));
}
