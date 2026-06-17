use bevy::prelude::*;
use bevy::text::FontSmoothing;

use crate::club::{Bouncer, Club};
use crate::coin::{Coin, Wallet};
use crate::collectable::Collectable;
use crate::enemy::Enemy;
use crate::map::{FreshGrass, Sidewalk, TrampledGrass};
use crate::player::Player;

// Globalny stan gry.
#[derive(States, Clone, Copy, Default, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    // Czekamy aż assety (tekstury) skończą się ładować, zanim narysujemy planszę.
    #[default]
    Loading,
    Playing,
    // Walka z bossem (dziekanem) w klubie.
    BossFight,
    GameOver,
}

// Warunek uruchomienia systemów wspólnych dla zwykłej rozgrywki i walki z bossem
// (np. sterowanie graczem, ruch, animacje) - aktywne w Playing oraz BossFight.
pub fn in_gameplay(state: Res<State<GameState>>) -> bool {
    matches!(state.get(), GameState::Playing | GameState::BossFight)
}

// Wynik ostatniej rozgrywki - decyduje co pokazujemy na ekranie końca gry.
#[derive(Resource, Debug, Default)]
pub struct GameResult {
    pub won: bool,
    // Niestandardowy tytuł/podtytuł; gdy puste, używane są domyślne komunikaty.
    pub title: String,
    pub subtitle: String,
}

impl GameResult {
    pub fn set(&mut self, won: bool, title: &str, subtitle: &str) {
        self.won = won;
        self.title = title.to_string();
        self.subtitle = subtitle.to_string();
    }
}

// Marker dla efektów ekranu śmierci (np. czaszka), aby posprzątać je przy restarcie.
#[derive(Component, Debug)]
pub struct DeathMarker;

// Marker przycisku "Zagraj ponownie".
#[derive(Component, Debug)]
pub struct PlayAgainButton;

// Marker przycisku "Wyjdz" (zamyka okno z gra).
#[derive(Component, Debug)]
pub struct ExitButton;

// Pikselowa paleta spójna z resztą gry (ostre, mocne kolory bez gradientów).
const OVERLAY_COLOR: Color = Color::srgba(0.04, 0.05, 0.07, 0.82);
const PANEL_BG: Color = Color::srgb(0.13, 0.12, 0.18); // ciemny granat panelu
const FRAME_DARK: Color = Color::srgb(0.04, 0.04, 0.06); // zewnętrzny czarny obrys
const SUBTITLE_COLOR: Color = Color::srgb(0.80, 0.82, 0.70);

// Kolory przycisku (zielone, jak świeża trawa) w zależności od interakcji.
const BUTTON_NORMAL: Color = Color::srgb(0.20, 0.52, 0.26);
const BUTTON_HOVERED: Color = Color::srgb(0.28, 0.68, 0.34);
const BUTTON_PRESSED: Color = Color::srgb(0.14, 0.38, 0.18);
const BUTTON_BORDER: Color = Color::srgb(0.07, 0.20, 0.10);
const BUTTON_BORDER_HOVERED: Color = Color::srgb(0.85, 0.92, 0.70);

// Kolory przycisku "Wyjdz" (czerwone).
const EXIT_NORMAL: Color = Color::srgb(0.62, 0.16, 0.16);
const EXIT_HOVERED: Color = Color::srgb(0.80, 0.24, 0.22);
const EXIT_PRESSED: Color = Color::srgb(0.44, 0.10, 0.10);
const EXIT_BORDER: Color = Color::srgb(0.28, 0.06, 0.06);

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .init_resource::<GameResult>()
            .add_systems(OnEnter(GameState::GameOver), spawn_game_over_ui)
            .add_systems(OnExit(GameState::GameOver), cleanup_gameplay_entities)
            .add_systems(
                Update,
                (play_again_button, exit_button).run_if(in_state(GameState::GameOver)),
            );
    }
}

// Buduje napis w pikselowym stylu - bez wygładzania krawędzi czcionki.
fn pixel_text(content: &str, size: f32, color: Color) -> (Text, TextFont, TextColor) {
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

fn spawn_game_over_ui(mut commands: Commands, result: Res<GameResult>) {
    // Dobór motywu (kolor ramki/akcentu + treść) zależnie od wyniku.
    let (def_title, title_color, accent, def_subtitle) = if result.won {
        (
            "WYGRANA!",
            Color::srgb(1.0, 0.84, 0.0),   // złoty tytuł
            Color::srgb(0.95, 0.74, 0.10), // złota ramka
            "Zdeptales caly swiezy trawnik!",
        )
    } else {
        (
            "PRZEGRANA",
            Color::srgb(0.92, 0.24, 0.22),  // czerwony tytuł
            Color::srgb(0.66, 0.14, 0.14),  // ciemnoczerwona ramka
            "Jaguar cie dopadl...",
        )
    };

    // Jeśli wynik niesie własny komunikat (np. z walki z bossem), użyj go.
    let title = if result.title.is_empty() { def_title } else { result.title.as_str() };
    let subtitle = if result.subtitle.is_empty() { def_subtitle } else { result.subtitle.as_str() };

    commands.spawn((
        // Cały kontener UI znika automatycznie po wyjściu ze stanu GameOver.
        DespawnOnExit(GameState::GameOver),
        // Pełnoekranowa, lekko przyciemniona nakładka wyśrodkowująca panel.
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(OVERLAY_COLOR),
        // Zewnętrzna, gruba czarna ramka okna (warstwa 1 podwójnego obramowania).
        children![(
            Node {
                border: UiRect::all(Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(FRAME_DARK),
            BorderColor::all(FRAME_DARK),
            // Wewnętrzny panel z kolorową ramką akcentu (warstwa 2) - ostre rogi.
            children![(
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    row_gap: Val::Px(20.0),
                    padding: UiRect::axes(Val::Px(48.0), Val::Px(36.0)),
                    border: UiRect::all(Val::Px(5.0)),
                    ..default()
                },
                BackgroundColor(PANEL_BG),
                BorderColor::all(accent),
                children![
                    pixel_text(title, 68.0, title_color),
                    (
                        pixel_text(subtitle, 20.0, SUBTITLE_COLOR),
                        Node {
                            margin: UiRect::bottom(Val::Px(8.0)),
                            ..default()
                        },
                    ),
                    (
                        // Wiersz z przyciskami: "Zagraj ponownie" + "Wyjdz".
                        Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(20.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        children![
                            (
                                Button,
                                PlayAgainButton,
                                Node {
                                    width: Val::Px(300.0),
                                    height: Val::Px(64.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    border: UiRect::all(Val::Px(4.0)),
                                    ..default()
                                },
                                BackgroundColor(BUTTON_NORMAL),
                                BorderColor::all(BUTTON_BORDER),
                                children![pixel_text("ZAGRAJ PONOWNIE", 26.0, Color::WHITE)],
                            ),
                            (
                                Button,
                                ExitButton,
                                Node {
                                    width: Val::Px(180.0),
                                    height: Val::Px(64.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    border: UiRect::all(Val::Px(4.0)),
                                    ..default()
                                },
                                BackgroundColor(EXIT_NORMAL),
                                BorderColor::all(EXIT_BORDER),
                                children![pixel_text("WYJDZ", 26.0, Color::WHITE)],
                            ),
                        ],
                    ),
                ],
            )],
        )],
    ));
}

fn play_again_button(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<PlayAgainButton>),
    >,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (interaction, mut color, mut border) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = BUTTON_PRESSED.into();
                *border = BorderColor::all(BUTTON_BORDER_HOVERED);
                next_state.set(GameState::Playing);
            }
            Interaction::Hovered => {
                *color = BUTTON_HOVERED.into();
                *border = BorderColor::all(BUTTON_BORDER_HOVERED);
            }
            Interaction::None => {
                *color = BUTTON_NORMAL.into();
                *border = BorderColor::all(BUTTON_BORDER);
            }
        }
    }
}

// Obsługa przycisku "Wyjdz" - zamyka okno z grą.
fn exit_button(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<ExitButton>),
    >,
    mut exit: MessageWriter<AppExit>,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = EXIT_PRESSED.into();
                exit.write(AppExit::Success);
            }
            Interaction::Hovered => {
                *color = EXIT_HOVERED.into();
            }
            Interaction::None => {
                *color = EXIT_NORMAL.into();
            }
        }
    }
}

// Sprząta wszystkie encje rozgrywki, aby OnEnter(Playing) mogło zbudować świeżą planszę.
fn cleanup_gameplay_entities(
    mut commands: Commands,
    mut wallet: ResMut<Wallet>,
    entities: Query<
        Entity,
        Or<(
            With<Enemy>,
            With<Collectable>,
            With<Coin>,
            With<Sidewalk>,
            With<FreshGrass>,
            With<TrampledGrass>,
            With<Player>,
            With<DeathMarker>,
            With<Club>,
            With<Bouncer>,
        )>,
    >,
) {
    for entity in &entities {
        commands.entity(entity).despawn();
    }
    // Nowa gra zaczyna się z zerowym kontem monet.
    wallet.coins = 0;
}
