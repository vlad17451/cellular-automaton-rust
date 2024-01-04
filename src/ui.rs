use bevy::prelude::*;

use crate::Age;
use crate::IsPaused;
use crate::AgeTimer;
use crate::cells::Cell;
use crate::cells::CellMap;
use crate::cells::reset_cells;
use crate::slow_age_speed;
use crate::speed_up_age_speed;
use crate::toggle_pause;

#[derive(Component, Debug)]
pub enum ButtonIcon {
    Play,
    Pause
}

#[derive(Component)]
pub enum ButtonAction {
    Minus,
    Plus,
    PlayOrPouse,
    Reset,
}

#[derive(Component, Debug)]
struct Scoreboard;

#[derive(Component)]
struct ProgressBar;

pub struct ButtonsPlugin;

impl Plugin for ButtonsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_buttons)
            .add_systems(Startup, setup_progress_bar)
            .add_systems(Startup, setup_reset_button)
            .add_systems(Startup, setup_scoreboard)
            .add_systems(Update, pause_button_style)
            .add_systems(Update, button_style)
            .add_systems(Update, button_action)
            .add_systems(Update, progress_bar_system)
            .add_systems(Update, scoreboard_system);
    }
}

fn progress_bar_system(
    mut query: Query<&mut Style, With<ProgressBar>>,
    timer: Res<AgeTimer>,
) {
    let mut style = query.single_mut();
    style.width = Val::Percent(timer.0.percent() * 100.);
}

fn button_action(
    interaction_query: Query<
        (&Interaction, &ButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut timer: ResMut<AgeTimer>,
    mut is_paused: ResMut<IsPaused>,
    cell_map: ResMut<CellMap>,
) {
    for (interaction, menu_button_action) in &interaction_query {
        if *interaction != Interaction::Pressed {
           continue;
        }
        match menu_button_action {
            ButtonAction::Minus => {
                slow_age_speed(&mut timer);
            },
            ButtonAction::Plus => {
                speed_up_age_speed(&mut timer);
            },
            ButtonAction::PlayOrPouse => {
                toggle_pause(&mut is_paused);
            },
            ButtonAction::Reset => {
                reset_cells(cell_map);
                return;
            }
        }
    }
}

fn pause_button_style(
    is_paused: Res<IsPaused>,
    mut icons_query: Query<(&ButtonIcon, &mut Style)>
) {
    if !is_paused.is_changed() {
        return;
    }
    for (icon, mut style) in icons_query.iter_mut() {
        if is_paused.0 {
            match icon {
                ButtonIcon::Play => {
                    style.display = Display::Flex;
                },
                ButtonIcon::Pause => {
                    style.display = Display::None;
                }
            }
        } else {
            match icon {
                ButtonIcon::Play => {
                    style.display = Display::None;
                },
                ButtonIcon::Pause => {
                    style.display = Display::Flex;
                }
            }
        }

    }
}


fn button_style(
    mut interaction_query: Query<
        (&Interaction, &mut BorderColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                border_color.0 = Color::SILVER;
            }
            Interaction::Hovered => {
                border_color.0 = Color::rgb(0.25, 0.25, 0.25);
            }
            Interaction::None => {
                border_color.0 = Color::BLACK;
            }
        }
    }
}

fn setup_scoreboard(mut commands: Commands) {
     commands.spawn((
        TextBundle::from_section(
            "",
            TextStyle {
                font_size: 40.0,
                color: Color::WHITE,
                ..default()
            },
        ),
        Scoreboard
    ));
}

fn setup_progress_bar(mut commands: Commands) {

    let progress_bar_container = NodeBundle {
        style: Style {
            width: Val::Px(65.),
            height: Val::Px(10.),
            align_items: AlignItems::Center,
            margin: UiRect::bottom(Val::Px(75.)),
            justify_content: JustifyContent::Start,
            ..default()
        },
        background_color: Color::BLACK.into(),
        ..default()
    };

    let progress_bar = (
        NodeBundle {
            style: Style {
                width: Val::Percent(35.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Start,
                ..default()
            },
            border_color: BorderColor(Color::BLACK),
            background_color: Color::WHITE.into(),
            ..default()
        },
        ProgressBar
    );

    let bottom_center_container = NodeBundle {
        style: Style {
            position_type: PositionType::Relative,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::End,
            padding: UiRect::bottom(Val::Px(40.0)),
            justify_content: JustifyContent::Center,
            ..default()
        },
        ..default()
    };

    commands
        .spawn(bottom_center_container.clone())
        .with_children(|parent| {
            parent.spawn(progress_bar_container)
                .with_children(|parent| {
                    parent.spawn(progress_bar);
                });
        });
}

fn get_button_bundle() -> ButtonBundle {
    let button_bundle = ButtonBundle {
        style: Style {
            width: Val::Px(65.0),
            height: Val::Px(65.0),
            border: UiRect::all(Val::Px(5.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            margin: UiRect::all(Val::Px(5.0)),
            ..default()
        },
        border_color: BorderColor(Color::BLACK),
        background_color: Color::rgb(0.15, 0.15, 0.15).into(),
        ..default()
    };
    return button_bundle;
}

fn setup_reset_button(
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
    let top_right_container = NodeBundle {
        style: Style {
            position_type: PositionType::Relative,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Start,
            padding: UiRect::all(Val::Px(10.0)),
            justify_content: JustifyContent::End,
            ..default()
        },
        ..default()
    };

    let text_style = TextStyle {
        font: asset_server.load("fonts/FiraSans-Medium.ttf"),
        font_size: 40.0,
        color: Color::WHITE,
    };

    let button_bundle = get_button_bundle();

    let reset_button = (
        ButtonBundle {
            style: Style {
                width: Val::Px(130.0),
                ..button_bundle.style.clone()
            },
            ..button_bundle.clone()
        }, 
        ButtonAction::Reset,
    );

    let reset_text = TextBundle::from_section("Reset", text_style.clone());

    commands
        .spawn(top_right_container)
        .with_children(|parent| {
            parent.spawn(reset_button)
                .with_children(|parent| {
                    parent.spawn(reset_text);
                });
        });
}

fn setup_buttons(
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {

    let button_icon_style = Style {
        width: Val::Px(30.0),
        display: Display::None,
        ..default()
    };

    let button_bundle = ButtonBundle {
        style: Style {
            width: Val::Px(65.0),
            height: Val::Px(65.0),
            border: UiRect::all(Val::Px(5.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            margin: UiRect::all(Val::Px(5.0)),
            ..default()
        },
        border_color: BorderColor(Color::BLACK),
        background_color: Color::rgb(0.15, 0.15, 0.15).into(),
        ..default()
    };

    let text_style = TextStyle {
        font: asset_server.load("fonts/FiraSans-Medium.ttf"),
        font_size: 40.0,
        color: Color::WHITE,
    };

    let bottom_center_container = NodeBundle {
        style: Style {
            position_type: PositionType::Relative,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::End,
            padding: UiRect::bottom(Val::Px(40.0)),
            justify_content: JustifyContent::Center,
            ..default()
        },
        ..default()
    };

    let play_button_icon = (
        ImageBundle {
            style: button_icon_style.clone(),
            image: UiImage::new(asset_server.load("icons/play.png")),
            ..default()
        },
        ButtonIcon::Play
    );

    let pause_button_icon = (
        ImageBundle {
            style: button_icon_style.clone(),
            image: UiImage::new(asset_server.load("icons/pause.png")),
            ..default()
        },
        ButtonIcon::Pause
    );

    let minus_button = (
        button_bundle.clone(),
        ButtonAction::Minus,
    );
    let minus_text = TextBundle::from_section("-", text_style.clone());


    let toggle_pause_button = (
        button_bundle.clone(),
        ButtonAction::PlayOrPouse,
    );

    let plus_button = (
        button_bundle.clone(),
        ButtonAction::Plus,
    );

    let plus_text = TextBundle::from_section("+", text_style.clone());

    commands
        .spawn(bottom_center_container)
        .with_children(|parent| {
             parent
                .spawn(minus_button)
                .with_children(|parent| {
                    parent.spawn(minus_text);
                });
            parent
                .spawn(toggle_pause_button)
                .with_children(|parent| {
                    parent.spawn(pause_button_icon);
                    parent.spawn(play_button_icon);
                });
            parent
                .spawn(plus_button)
                .with_children(|parent| {
                    parent.spawn(plus_text);
                });

        });
}

fn scoreboard_system(
    mut query: Query<&mut Text, With<Scoreboard>>,
    timer: Res<AgeTimer>,
    age: Res<Age>,
    cells: Query<&Cell>
) {
    if !timer.is_changed() && !age.is_changed() {
        return;
    }
    let mut text = query.single_mut();
    text.sections[0].value = format!("Speed: {}ms/age\nAge: {}\nCells: {}", timer.0.duration().as_millis(), age.0, cells.iter().count());
}