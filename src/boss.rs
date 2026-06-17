use bevy::prelude::*;
use bevy::audio::{AudioSink, AudioSinkPlayback, Volume};
use bevy::text::FontSmoothing;
use rand::seq::SliceRandom;
use rand::Rng;

use crate::asset_loader::{MainMusic, SceneAssets};
use crate::coin::Coin;
use crate::collectable::Collectable;
use crate::enemy::Enemy;
use crate::map::{FreshGrass, Sidewalk, TrampledGrass};
use crate::player::Player;
use crate::state::{GameResult, GameState};

const PLAYER_START_Y: f32 = -62.0;
const PAD_X: f32 = 58.0;
const PAD_Y: f32 = -12.0;
const PAD_DETECT: f32 = 18.0;
// Bazowy czas na odpowiedz (skraca sie nieznacznie, gdy boss slabnie).
const BASE_TIME: f32 = 9.0;

const FLOOR_COLOR: Color = Color::srgb(0.93, 0.94, 0.96);
const FLOOR_ACCENT: Color = Color::srgb(0.84, 0.86, 0.90);
const PAD_COLOR: Color = Color::srgb(0.20, 0.30, 0.52);

const PLAYER_PIP: Color = Color::srgb(0.30, 0.80, 0.36);
const BOSS_PIP: Color = Color::srgb(0.88, 0.22, 0.22);
const PIP_OFF: Color = Color::srgb(0.20, 0.20, 0.24);

const QUESTIONS: &[(&str, &str, &str)] = &[
    ("1 + 1 w systemie binarnym to:", "10", "2"),
    ("Jezyk z borrow-checkerem to:", "RUST", "PHP"),
    ("0xFF w systemie dziesietnym to:", "255", "127"),
    ("Struktura danych typu LIFO to:", "STOS", "KOLEJKA"),
    ("Zlozonosc wyszukiwania binarnego:", "LOG N", "N^2"),
    ("Protokol HTTP dziala na porcie:", "80", "25"),
    ("8 bitow to:", "BAJT", "SLOWO"),
    ("Operator AND w logice to:", "&&", "||"),
];

#[derive(Component, Debug)]
pub struct Boss;

#[derive(Component, Debug)]
struct AnswerPad {
    correct: bool,
}

#[derive(Component, Debug)]
struct RoundEntity;

#[derive(Component, Debug, PartialEq, Eq, Clone, Copy)]
enum BossText {
    Timer,
    Question,
}

#[derive(Component, Debug)]
struct HpPip {
    is_player: bool,
    index: i32,
}

#[derive(Component, Debug)]
struct TimerBar;

#[derive(Component, Debug)]
struct IntroOverlay;

#[derive(Resource, Debug)]
struct BossBattle {
    boss_hp: i32,
    player_hp: i32,
    timer: Timer,
    awaiting: bool,
    intro: bool,
    // Talia indeksow pytan jeszcze nie zadanych w tej walce (bez powtorzen).
    remaining: Vec<usize>,
}

pub struct BossPlugin;

impl Plugin for BossPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::BossFight), (setup_boss_fight, start_boss_music))
            .add_systems(OnExit(GameState::BossFight), resume_main_music)
            .add_systems(
                Update,
                (boss_fight, update_boss_hud)
                    .chain()
                    .run_if(in_state(GameState::BossFight)),
            );
    }
}

// Pauzuje muzyke zwyklej gry i wlacza muzyke walki (placeholder: music_boss.mp3).
fn start_boss_music(
    mut commands: Commands,
    main_music: Query<&AudioSink, With<MainMusic>>,
    scene_assets: Res<SceneAssets>,
) {
    for sink in &main_music {
        sink.pause();
    }
    commands.spawn((
        DespawnOnExit(GameState::BossFight),
        AudioPlayer::new(scene_assets.boss_music.clone()),
        PlaybackSettings::LOOP.with_volume(Volume::Linear(0.25)),
    ));
}

// Po wyjsciu z walki przywraca muzyke zwyklej gry.
fn resume_main_music(main_music: Query<&AudioSink, With<MainMusic>>) {
    for sink in &main_music {
        sink.play();
    }
}

fn setup_boss_fight(
    mut commands: Commands,
    scene_assets: Res<SceneAssets>,
    overworld: Query<
        Entity,
        Or<(
            With<Enemy>,
            With<Collectable>,
            With<Coin>,
            With<Sidewalk>,
            With<FreshGrass>,
            With<TrampledGrass>,
        )>,
    >,
    mut player: Query<&mut Transform, With<Player>>,
) {
    for entity in &overworld {
        commands.entity(entity).despawn();
    }

    if let Ok(mut transform) = player.single_mut() {
        transform.translation.x = 0.0;
        transform.translation.y = PLAYER_START_Y;
    }

    commands.spawn((
        DespawnOnExit(GameState::BossFight),
        Sprite::from_color(FLOOR_COLOR, Vec2::new(340.0, 190.0)),
        Transform::from_translation(Vec3::new(0.0, 0.0, -60.0)),
    ));
    commands.spawn((
        DespawnOnExit(GameState::BossFight),
        Sprite::from_color(FLOOR_ACCENT, Vec2::new(340.0, 70.0)),
        Transform::from_translation(Vec3::new(0.0, 40.0, -55.0)),
    ));

    // Dziekan jako sprite (placeholder - podmien assets/dean.png). Kwadrat,
    // by tekstura 500x500 nie byla rozciagnieta; caly widoczny u gory ekranu.
    commands.spawn((
        DespawnOnExit(GameState::BossFight),
        Boss,
        Sprite {
            image: scene_assets.dean.clone(),
            custom_size: Some(Vec2::new(96.0, 96.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 40.0, -10.0)),
    ));

    spawn_boss_hud(&mut commands);
    spawn_intro_overlay(&mut commands);

    commands.insert_resource(BossBattle {
        boss_hp: 3,
        player_hp: 3,
        timer: Timer::from_seconds(BASE_TIME, TimerMode::Once),
        awaiting: false,
        intro: true,
        remaining: shuffled_deck(),
    });
}

// Przetasowana talia indeksow wszystkich pytan (kazde raz, bez powtorzen).
fn shuffled_deck() -> Vec<usize> {
    let mut deck: Vec<usize> = (0..QUESTIONS.len()).collect();
    deck.shuffle(&mut rand::thread_rng());
    deck
}

fn boss_fight(
    mut commands: Commands,
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut battle: ResMut<BossBattle>,
    intro_overlay: Query<Entity, With<IntroOverlay>>,
    mut player_and_pads: ParamSet<(
        Query<&mut Transform, With<Player>>,
        Query<(&Transform, &AnswerPad)>,
    )>,
    rounds: Query<Entity, With<RoundEntity>>,
    mut texts: Query<(&mut Text, &BossText)>,
    mut next_state: ResMut<NextState<GameState>>,
    mut game_result: ResMut<GameResult>,
) {
    if battle.intro {
        if keyboard.just_pressed(KeyCode::Enter)
            || keyboard.just_pressed(KeyCode::Space)
        {
            for entity in &intro_overlay {
                commands.entity(entity).despawn();
            }
            battle.intro = false;
        }
        return;
    }

    if !battle.awaiting {
        if let Ok(mut transform) = player_and_pads.p0().single_mut() {
            transform.translation.x = 0.0;
            transform.translation.y = PLAYER_START_Y;
        }

        let mut rng = rand::thread_rng();
        // Pobierz nastepne pytanie z talii bez powtorzen (gdy pusta - przetasuj na nowo).
        if battle.remaining.is_empty() {
            battle.remaining = shuffled_deck();
        }
        let question_index = battle.remaining.pop().unwrap();
        let (question, correct, wrong) = QUESTIONS[question_index];
        let correct_on_left = rng.gen_bool(0.5);

        let (left_label, left_correct) = if correct_on_left {
            (correct, true)
        } else {
            (wrong, false)
        };
        let (right_label, right_correct) = if correct_on_left {
            (wrong, false)
        } else {
            (correct, true)
        };

        spawn_pad(&mut commands, -PAD_X, left_label, left_correct);
        spawn_pad(&mut commands, PAD_X, right_label, right_correct);

        // Im slabszy boss, tym nieco krotszy czas - ale wciaz komfortowy.
        let secs = (BASE_TIME - (3 - battle.boss_hp) as f32 * 0.5).max(6.0);
        battle.timer = Timer::from_seconds(secs, TimerMode::Once);
        battle.awaiting = true;

        set_boss_text(&mut texts, BossText::Question, question);
        return;
    }

    battle.timer.tick(time.delta());

    let player_pos = match player_and_pads.p0().single() {
        Ok(transform) => transform.translation,
        Err(_) => return,
    };

    // Czy gracz stoi na którymś polu odpowiedzi?
    let mut hit: Option<bool> = None;
    for (transform, pad) in player_and_pads.p1().iter() {
        if player_pos
            .truncate()
            .distance(transform.translation.truncate())
            < PAD_DETECT
        {
            hit = Some(pad.correct);
            break;
        }
    }

    let timed_out = battle.timer.is_finished();
    if hit.is_none() && !timed_out {
        return;
    }

    // Rozstrzygnięcie rundy - posprzątaj pola.
    for entity in &rounds {
        commands.entity(entity).despawn();
    }

    match hit {
        Some(true) => battle.boss_hp -= 1, // poprawna odpowiedź - boss obrywa
        Some(false) => battle.player_hp -= 1, // zła odpowiedź
        None => battle.player_hp -= 1,     // czas minął
    }

    if battle.boss_hp <= 0 {
        game_result.set(
            true,
            "ZALICZONE!",
            "Pokonales Dziekana Gajeckiego na egzaminie z informatyki!",
        );
        next_state.set(GameState::GameOver);
    } else if battle.player_hp <= 0 {
        game_result.set(
            false,
            "EGZAMIN OBLANY!",
            "Dziekan Gajecki odeslal cie na poprawke...",
        );
        next_state.set(GameState::GameOver);
    } else {
        // Następna runda w kolejnej klatce.
        battle.awaiting = false;
    }
}

fn spawn_pad(commands: &mut Commands, x: f32, label: &str, correct: bool) {
    commands.spawn((
        RoundEntity,
        AnswerPad { correct },
        Sprite::from_color(PAD_COLOR, Vec2::new(36.0, 28.0)),
        Transform::from_translation(Vec3::new(x, PAD_Y, -30.0)),
        children![(
            Text2d::new(label),
            TextFont {
                font_size: 18.0,
                font_smoothing: FontSmoothing::None,
                ..default()
            },
            TextColor(Color::BLACK),
            Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
        )],
    ));
}

// Aktualizuje HUD walki co klatkę: paski HP (pipy) i pasek czasu.
fn update_boss_hud(
    battle: Res<BossBattle>,
    mut texts: Query<(&mut Text, &BossText)>,
    mut pips: Query<(&mut BackgroundColor, &HpPip)>,
    mut bar: Query<&mut Node, With<TimerBar>>,
) {
    // Pipy HP: zapalone, gdy index < aktualne HP.
    for (mut color, pip) in pips.iter_mut() {
        let hp = if pip.is_player {
            battle.player_hp
        } else {
            battle.boss_hp
        };
        let lit = pip.index < hp;
        let on = if pip.is_player { PLAYER_PIP } else { BOSS_PIP };
        *color = if lit { on.into() } else { PIP_OFF.into() };
    }

    // Numeryczny czas.
    for (mut text, kind) in texts.iter_mut() {
        if *kind == BossText::Timer {
            if battle.intro {
                text.0 = "CZAS: --".to_string();
            } else {
                text.0 = format!("CZAS: {:.1}s", battle.timer.remaining_secs());
            }
        }
    }

    // Pasek czasu - szeroko\u015b\u0107 proporcjonalna do pozosta\u0142ego czasu.
    let total = battle.timer.duration().as_secs_f32().max(0.0001);
    let ratio = if battle.intro {
        1.0
    } else {
        (battle.timer.remaining_secs() / total).clamp(0.0, 1.0)
    };
    for mut node in bar.iter_mut() {
        node.width = Val::Percent(ratio * 100.0);
    }
}

fn set_boss_text(texts: &mut Query<(&mut Text, &BossText)>, kind: BossText, value: &str) {
    for (mut text, current) in texts.iter_mut() {
        if *current == kind {
            text.0 = value.to_string();
        }
    }
}

fn boss_pixel_text(content: &str, size: f32, color: Color) -> (Text, TextFont, TextColor) {
    (
        Text::new(content),
        TextFont {
            font_size: size,
            font_smoothing: FontSmoothing::None,
            ..default()
        },
        TextColor(color),
    )
}

fn spawn_boss_hud(commands: &mut Commands) {
    // Górny pasek: panel HP gracza (lewo) i panel HP bossa (prawo).
    commands.spawn((
        DespawnOnExit(GameState::BossFight),
        Node {
            width: Val::Percent(100.0),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::FlexStart,
            padding: UiRect::all(Val::Px(12.0)),
            column_gap: Val::Px(10.0),
            ..default()
        },
        children![
            hp_panel("TY", true, Color::srgb(0.30, 0.80, 0.36)),
            hp_panel("DZIEKAN GAJECKI", false, Color::srgb(0.90, 0.30, 0.30)),
        ],
    ));

    // Panel czasu - osobna, idealnie wyśrodkowana warstwa u góry ekranu.
    commands.spawn((
        DespawnOnExit(GameState::BossFight),
        Node {
            width: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::FlexStart,
            padding: UiRect::top(Val::Px(12.0)),
            ..default()
        },
        children![timer_panel()],
    ));

    // Dolny pas ekranu: ramka z pytaniem + instrukcja. Umieszczony nisko, aby
    // byl wyrazny, ale nie zaslanial sprite'a dziekana u gory.
    commands.spawn((
        DespawnOnExit(GameState::BossFight),
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::FlexEnd,
            align_items: AlignItems::Center,
            padding: UiRect::bottom(Val::Px(16.0)),
            row_gap: Val::Px(8.0),
            ..default()
        },
        children![
            // Ramka pytania.
            (
                Node {
                    padding: UiRect::axes(Val::Px(20.0), Val::Px(12.0)),
                    border: UiRect::all(Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.06, 0.05, 0.10, 0.94)),
                BorderColor::all(Color::srgb(1.0, 0.85, 0.3)),
                children![(
                    boss_pixel_text("Przygotuj sie...", 24.0, Color::srgb(0.95, 0.95, 0.80)),
                    BossText::Question,
                )],
            ),
            // Instrukcja pod pytaniem.
            (boss_pixel_text(
                "Wejdz (WASD) na pole z POPRAWNA odpowiedzia zanim minie czas!",
                16.0,
                Color::srgb(0.85, 0.86, 0.92),
            ),),
        ],
    ));
}

// Panel HP z etykietą i trzema pipami (pełnymi "sercami").
fn hp_panel(label: &str, is_player: bool, label_color: Color) -> impl Bundle {
    (
        Node {
            flex_direction: FlexDirection::Column,
            align_items: if is_player {
                AlignItems::FlexStart
            } else {
                AlignItems::FlexEnd
            },
            row_gap: Val::Px(6.0),
            padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
            border: UiRect::all(Val::Px(3.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.06, 0.05, 0.10, 0.90)),
        BorderColor::all(label_color),
        children![
            boss_pixel_text(label, 20.0, label_color),
            (
                // Rząd trzech pipów HP.
                Node {
                    column_gap: Val::Px(5.0),
                    ..default()
                },
                children![
                    hp_pip(is_player, 0),
                    hp_pip(is_player, 1),
                    hp_pip(is_player, 2),
                ],
            ),
        ],
    )
}

fn hp_pip(is_player: bool, index: i32) -> impl Bundle {
    let color = if is_player { PLAYER_PIP } else { BOSS_PIP };
    (
        Node {
            width: Val::Px(22.0),
            height: Val::Px(22.0),
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        BackgroundColor(color),
        BorderColor::all(Color::srgb(0.04, 0.04, 0.06)),
        HpPip { is_player, index },
    )
}

// Panel czasu z numerem i kurczącym się paskiem.
fn timer_panel() -> impl Bundle {
    (
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::Px(6.0),
            padding: UiRect::axes(Val::Px(14.0), Val::Px(8.0)),
            border: UiRect::all(Val::Px(3.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.06, 0.05, 0.10, 0.90)),
        BorderColor::all(Color::srgb(1.0, 0.6, 0.2)),
        children![
            (
                boss_pixel_text("CZAS: --", 22.0, Color::srgb(1.0, 0.7, 0.25)),
                BossText::Timer,
            ),
            (
                // Tło paska czasu.
                Node {
                    width: Val::Px(180.0),
                    height: Val::Px(16.0),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.15, 0.15, 0.18)),
                BorderColor::all(Color::srgb(0.04, 0.04, 0.06)),
                children![(
                    // Wypełnienie paska (szerokość zmieniana w update_boss_hud).
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(1.0, 0.6, 0.2)),
                    TimerBar,
                )],
            ),
        ],
    )
}

// Okno wprowadzające do walki - wyjaśnia zasady, znika po naciśnięciu klawisza.
fn spawn_intro_overlay(commands: &mut Commands) {
    commands.spawn((
        DespawnOnExit(GameState::BossFight),
        IntroOverlay,
        // Pełnoekranowe przyciemnienie z wyśrodkowanym oknem.
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.02, 0.02, 0.05, 0.86)),
        children![(
            // Czarna gruba ramka.
            Node {
                border: UiRect::all(Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.04, 0.04, 0.06)),
            BorderColor::all(Color::srgb(0.04, 0.04, 0.06)),
            children![(
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    row_gap: Val::Px(18.0),
                    padding: UiRect::axes(Val::Px(46.0), Val::Px(34.0)),
                    border: UiRect::all(Val::Px(5.0)),
                    max_width: Val::Px(640.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.13, 0.12, 0.18)),
                BorderColor::all(Color::srgb(0.90, 0.22, 0.22)),
                children![
                    boss_pixel_text("!! WALKA Z BOSSEM !!", 40.0, Color::srgb(0.95, 0.25, 0.25)),
                    boss_pixel_text("DZIEKAN MAREK GAJECKI", 28.0, Color::srgb(1.0, 0.85, 0.3)),
                    boss_pixel_text(
                        "Egzamin z informatyki! Dziekan zadaje pytanie,\na Ty masz dwa pola z odpowiedziami.\nWejdz (WASD) na POPRAWNE pole zanim minie czas.\nDobra odpowiedz rani dziekana, zla rani Ciebie.\nKazdy ma 3 zycia. Zbij go na egzaminie!",
                        20.0,
                        Color::srgb(0.88, 0.90, 0.80),
                    ),
                    boss_pixel_text(
                        "[ Nacisnij ENTER lub SPACJE, aby rozpoczac ]",
                        22.0,
                        Color::srgb(0.45, 0.85, 0.50),
                    ),
                ],
            )],
        )],
    ));
}
