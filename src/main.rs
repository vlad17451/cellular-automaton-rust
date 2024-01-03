use std::collections::HashMap;

use bevy::prelude::*;
use bevy_pancam::{PanCam, PanCamPlugin};


#[derive(Component, Debug)]
struct Cell;



#[derive(Component)]
enum ButtonAction {
    DecriceSpeed,
    IncriceSpeed,
    TogglePause,
    Reset,
}

#[derive(Component, Debug)]
struct AgeDurationIndicator;

#[derive(Component, Debug)]
enum Icon {
    Play,
    Pause
}

#[derive(Resource)]
struct AgeTimer(Timer);

#[derive(Resource, Default)]
struct Age(i32);

#[derive(Resource, Default)]
struct IsPaused(bool);

// #[derive(Resource)]
// struct StopPropagation(bool);

const MIN_AGE_DURATION: f32 = 0.01;
const MAX_AGE_DURATION: f32 = 10.;
const WORLD_EDGE: i32 = 4000;

const INITIAL_CELLS : [[i32; 10]; 10] = [
    [1, 1, 1, 0, 0, 0, 0, 0, 0, 0],
    [1, 0, 0, 0, 0, 0, 1, 1, 0, 0],
    [0, 1, 0, 1, 0, 0, 0, 0, 1, 0],
    [0, 0, 0, 0, 1, 0, 1, 1, 1, 0],
    [0, 0, 1, 0, 0, 1, 1, 0, 1, 0],
    [0, 0, 1, 0, 1, 0, 0, 1, 0, 1],
    [0, 0, 1, 0, 0, 1, 0, 0, 0, 0],
    [0, 0, 0, 1, 0, 1, 0, 1, 0, 0],
    [0, 0, 0, 0, 0, 0, 1, 1, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 1, 0, 0],
];

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PanCamPlugin::default()))
        .init_resource::<Age>()
        .init_resource::<IsPaused>()
        .insert_resource(AgeTimer(Timer::from_seconds(
            0.1,
            TimerMode::Repeating,
        )))
        .add_systems(Startup, setup)        
        .add_systems(Startup, setup_buttons)
        .add_systems(Update, check_cells)
        .add_systems(Update, button_action)
        .add_systems(Update, button_system)
        .add_systems(Update, scoreboard_system)
        .add_systems(Update, pause_button_system)
        .add_systems(Update, draw_cursor)
        .add_systems(
            Update,
            apply_deferred
                .before(check_cells)
                .after(button_action)
        )
        .run();
}

fn draw_cursor(
    camera_query: Query<(&Camera, &GlobalTransform)>,
    cells_query: Query<&Transform, With<Cell>>,
    windows: Query<&Window>,
    buttons: Res<Input<MouseButton>>,
    mut commands: Commands,
) {
    let (camera, camera_transform) = camera_query.single();

    let Some(cursor_position) = windows.single().cursor_position() else {
        return;
    };

    let Some(point) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };

    // TODO add circle to show when age will be updated
    // TODO add keyboard shortcuts
    // TODO add reset button

    if buttons.pressed(MouseButton::Left) {
        let new_x = (point.x - 0.5).ceil() as i32;
        let new_y = (point.y - 0.5).ceil() as i32;

        if cells_query.iter().any(|transform| {
            let x = transform.translation.x as i32;
            let y = transform.translation.y as i32;
            x == new_x && y == new_y
        }) {
            return;
        }
        spawn_cell(&mut commands, new_x, new_y);
    }
}

fn scoreboard_system(
    mut query: Query<&mut Text, With<AgeDurationIndicator>>,
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
        Icon::Play
    );

    let pause_button_icon = (
        ImageBundle {
            style: button_icon_style.clone(),
            image: UiImage::new(asset_server.load("icons/pause.png")),
            ..default()
        },
        Icon::Pause
    );

    let decrice_speed_button = (
        button_bundle.clone(),
        ButtonAction::DecriceSpeed,
    );
    let decrice_speed_text = TextBundle::from_section("-", text_style.clone());


    let toggle_pause_button = (
        button_bundle.clone(),
        ButtonAction::TogglePause,
    );

    let incrice_speed_button = (
        button_bundle.clone(),
        ButtonAction::IncriceSpeed,
    );

    let incrice_speed_text = TextBundle::from_section("+", text_style.clone());

    let top_right_container = NodeBundle {
        style: Style {
            align_items: AlignItems::Start,
            padding: UiRect::all(Val::Px(10.0)),
            justify_content: JustifyContent::End,
            ..bottom_center_container.style.clone()
        },
        ..default()
    };

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

    commands
        .spawn(bottom_center_container)
        .with_children(|parent| {
             parent
                .spawn(decrice_speed_button)
                .with_children(|parent| {
                    parent.spawn(decrice_speed_text);
                });
            parent
                .spawn(toggle_pause_button)
                .with_children(|parent| {
                    parent.spawn(pause_button_icon);
                    parent.spawn(play_button_icon);
                });
            parent
                .spawn(incrice_speed_button)
                .with_children(|parent| {
                    parent.spawn(incrice_speed_text);
                });
        });
}

fn button_system(
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

fn pause_button_system(
    is_paused: Res<IsPaused>,
    mut icons_query: Query<(&Icon, &mut Style)>
) {
    if !is_paused.is_changed() {
        return;
    }
    for (icon, mut style) in icons_query.iter_mut() {
        if is_paused.0 {
            match icon {
                Icon::Play => {
                    style.display = Display::Flex;
                },
                Icon::Pause => {
                    style.display = Display::None;
                }
            }
        } else {
            match icon {
                Icon::Play => {
                    style.display = Display::None;
                },
                Icon::Pause => {
                    style.display = Display::Flex;
                }
            }
        }

    }
}

fn get_decreased_speed(speed: f32) -> f32 {
    let new_speed = speed * 2.;
    if new_speed > MAX_AGE_DURATION {
        return MAX_AGE_DURATION
    }
    new_speed
}

fn get_increased_speed(speed: f32) -> f32 {
    let new_speed = speed * 0.5;
    if new_speed < MIN_AGE_DURATION {
        return MIN_AGE_DURATION
    }
    new_speed
}

fn button_action(
    interaction_query: Query<
        (&Interaction, &ButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut timer: ResMut<AgeTimer>,
    mut is_paused: ResMut<IsPaused>,
    cells_query: Query<Entity, With<Cell>>,
    mut commands: Commands,
) {
    for (interaction, menu_button_action) in &interaction_query {
        if *interaction != Interaction::Pressed {
           continue;
        }
        match menu_button_action {
            ButtonAction::IncriceSpeed => {
                timer.0 = Timer::from_seconds(
                    get_increased_speed(timer.0.duration().as_secs_f32()),
                    TimerMode::Repeating,
                );
            },
            ButtonAction::DecriceSpeed => {
                timer.0 = Timer::from_seconds(
                    get_decreased_speed(timer.0.duration().as_secs_f32()),
                    TimerMode::Repeating,
                );
            },
            ButtonAction::TogglePause => {
                is_paused.0 = !is_paused.0;
            },
            ButtonAction::Reset => {
                reset(&cells_query, &mut commands);
            }
        }
    }
}

fn reset(
    cells_query: &Query<Entity, With<Cell>>,
    commands: &mut Commands,
) {
    for entity in cells_query.iter() {
        // TODO mark cell as dead
        commands.entity(entity).despawn_recursive();
    }
    spawn_initial_cells(commands);
}

fn setup(
    mut commands: Commands,
    mut is_paused: ResMut<IsPaused>,
    mut age: ResMut<Age>
) {
    age.0 = 0;
    is_paused.0 = false;

    commands.spawn((
        Camera2dBundle::default(),
        PanCam {
            grab_buttons: vec![MouseButton::Right, MouseButton::Middle],
            ..default()
        }
    ));

    commands.spawn((
        TextBundle::from_section(
            "",
            TextStyle {
                font_size: 40.0,
                color: Color::WHITE,
                ..default()
            },
        ),
        AgeDurationIndicator
    ));


    spawn_initial_cells(&mut commands);
}

fn spawn_initial_cells(
    commands: &mut Commands,
) {
    for (y, row) in INITIAL_CELLS.iter().enumerate() {
        for (x, &cell) in row.iter().enumerate() {
            if cell == 1 {
                spawn_cell(commands, x as i32, y as i32);
            }
        }
    }
}


fn spawn_cell(
    commands: &mut Commands,
    x: i32,
    y: i32,
) {
    if x.abs() > WORLD_EDGE || y.abs() > WORLD_EDGE {
        return;
    }
    
    commands.spawn((
        SpriteBundle {
            transform: Transform::from_xyz(x as f32, y as f32, 0.),
            ..default()
        },
        Cell,
    ));
}

fn check_cells(
    mut commands: Commands,
    query: Query<(Entity, &Transform), With<Cell>>,
    time: Res<Time>,
    mut timer: ResMut<AgeTimer>,
    is_paused: Res<IsPaused>,
    mut age: ResMut<Age>
) {
    
    if is_paused.0 {
        return;
    }

    if !timer.0.tick(time.delta()).finished() {
        return;
    }
    age.0 += 1;

    let cells = query.iter().map(|(_entity, transform)| {
        let x = transform.translation.x;
        let y = transform.translation.y;
        (x, y)
    });

    let mut old_cell_map: HashMap<String, bool> = HashMap::new();
    let mut new_cell_map: HashMap<String, bool> = HashMap::new();

    for (x, y) in cells {
        old_cell_map.insert(format!("{}#{}", x, y), true);
    }

    for (key, _value) in old_cell_map.iter() {
        let (x_str, y_str) = key.split_once("#").unwrap();
        let x: i32 = x_str.parse().unwrap();
        let y: i32 = y_str.parse().unwrap();

        let mut neighbours_count = 0;
        for i in -1..2 {
            for j in -1..2 {
                if i == 0 && j == 0 {
                    continue;
                }
                let x2 = x + i;
                let y2 = y + j;
                let is_alive = old_cell_map.get(&format!("{}#{}", x2, y2));
                if is_alive == Some(&true) {
                    neighbours_count += 1;
                    continue;
                }
                
                let is_already_spawned = new_cell_map.get(&format!("{}#{}", x2, y2));
                if is_already_spawned == Some(&true) {
                    continue;
                }

                let mut sub_neighbours_count = 0;

                for i in -1..2 {
                    for j in -1..2 {
                        if i == 0 && j == 0 {
                            continue;
                        }
                        let x3 = x2 + i;
                        let y3 = y2 + j;
                        let is_alive = old_cell_map.get(&format!("{}#{}", x3, y3));
                        if is_alive == Some(&true) {
                            sub_neighbours_count += 1;
                        }
                    }
                }
                if sub_neighbours_count == 3 {
                    spawn_cell(&mut commands, x2 as i32, y2 as i32);
                    new_cell_map.insert(format!("{}#{}", x2 as i32, y2 as i32), true);
                }
            }
        }

        if neighbours_count < 2 || neighbours_count > 3 {
            for (entity, transform) in query.iter() {
                if transform.translation.x == x as f32 && transform.translation.y == y as f32 {
                    commands.entity(entity).despawn_recursive();
                }
            }
        }
    }
}