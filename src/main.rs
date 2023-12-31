use std::collections::HashMap;

use bevy::prelude::*;

#[derive(Component, Debug)]
struct Cell;


#[derive(Resource)]
struct CircleTimer(Timer);


// TODO show current speed and age
// TODO implement logic of buttons
// TODO implement drawing

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(CircleTimer(Timer::from_seconds(
            0.1,
            TimerMode::Repeating,
        )))
        .add_systems(Startup, setup)
        .add_systems(Startup, setup_buttons)
        .add_systems(Update, update)
        .add_systems(Update, button_system)
        .add_systems(Update, metadata_system)
        .run();
}

fn metadata_system(

) {
    
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

fn setup_buttons(
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {

    let button_icon_style = Style {
        width: Val::Px(30.0),
        ..default()
    };

    let btn_bundle = ButtonBundle {
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
        background_color: NORMAL_BUTTON.into(),
        ..default()
    };

    let text_style = TextStyle {
        font: asset_server.load("fonts/FiraSans-Medium.ttf"),
        font_size: 40.0,
        color: Color::WHITE,
    };

    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::End,
                padding: UiRect::bottom(Val::Px(40.0)),
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(NodeBundle::default())
                .with_children(|parent| {
                    // spawn "-" dutton
                    parent
                        .spawn(btn_bundle.clone())
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section("-", text_style.clone() ));
                        });
                    // spawn "pause" dutton
                    parent
                        .spawn(btn_bundle.clone())
                        .with_children(|parent| {
                            let icon = asset_server.load("icons/pause.png");
                            parent.spawn(ImageBundle {
                                style: button_icon_style.clone(),
                                image: UiImage::new(icon),
                                ..default()
                            });
                        });
                    // spawn "+" dutton
                    parent
                        .spawn(btn_bundle.clone())
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section("+", text_style.clone() ));
                        });
                });
        });
}

fn setup(
    mut commands: Commands
) {
    commands.spawn(Camera2dBundle::default());
    
    let cells = vec![
        vec![1, 1, 1, 0, 0, 0, 0, 0, 0, 0],
        vec![1, 0, 0, 0, 0, 0, 1, 1, 0, 0],
        vec![0, 1, 0, 1, 0, 0, 0, 0, 1, 0],
        vec![0, 0, 0, 0, 1, 0, 1, 1, 1, 0],
        vec![0, 0, 0, 0, 0, 1, 1, 0, 1, 0],
        vec![0, 0, 0, 0, 1, 0, 0, 1, 0, 1],
        vec![0, 0, 0, 0, 0, 1, 1, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 1, 0, 1, 0, 0],
        vec![0, 0, 0, 0, 0, 0, 1, 1, 0, 0],
        vec![0, 0, 0, 0, 0, 0, 0, 1, 0, 0],
    ];

    for (y, row) in cells.iter().enumerate() {
        for (x, &cell) in row.iter().enumerate() {
            if cell == 1 {
                spawn_cell(&mut commands, x as i32, y as i32);
            }
        }
    }
    
}

const CELL_SIZE: f32 = 10.;

fn spawn_cell(
    commands: &mut Commands,
    x: i32,
    y: i32,
) {
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::WHITE,
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(x as f32 * CELL_SIZE, y as f32 * CELL_SIZE, 0.),
                scale: Vec3::new(CELL_SIZE, CELL_SIZE, 1.),
                ..default()
            },
            ..default()
        },
        Cell,
    ));
}

fn update(
    mut commands: Commands,
    query: Query<(Entity, &Transform), With<Cell>>,
    time: Res<Time>,
    mut timer: ResMut<CircleTimer>,
) {
    if !timer.0.tick(time.delta()).finished() {
        return;
    }
    // println!("----------------------");

    let cells = query.iter().map(|(_entity, transform)| {
        let x = transform.translation.x;
        let y = transform.translation.y;
        (x, y)
    });

    let mut cell_map: HashMap<String, bool> = HashMap::new();

    for (x, y) in cells {
        cell_map.insert(format!("{}#{}", x, y), true);
    }

    for (key, _value) in cell_map.iter() {
        let (x_str, y_str) = key.split_once("#").unwrap();
        let x: i32 = x_str.parse().unwrap();
        let y: i32 = y_str.parse().unwrap();
        // println!("{}, {}", x, y);

        let mut neighbours_count = 0;
        for i in -1..2 { // for x
            for j in -1..2 { // for y
                if i == 0 && j == 0 {
                    continue;
                }
                let x2 = x as f32 + (i as f32 * CELL_SIZE);
                let y2 = y as f32 + (j as f32 * CELL_SIZE);
                let is_alive = cell_map.get(&format!("{}#{}", x2, y2));
                if is_alive == Some(&true) {
                    neighbours_count += 1;
                }
                
                let mut sub_neighbours_count = 0;

                for i in -1..2 { // for x2
                    for j in -1..2 { // for y2
                        if i == 0 && j == 0 {
                            continue;
                        }
                        let x3 = x2 + (i as f32 * CELL_SIZE);
                        let y3 = y2 + (j as f32 * CELL_SIZE);
                        let is_alive = cell_map.get(&format!("{}#{}", x3, y3));
                        if is_alive == Some(&true) {
                            sub_neighbours_count += 1;
                        }
                    }
                }
                if sub_neighbours_count == 3 {
                    spawn_cell(&mut commands, (x2 / CELL_SIZE) as i32, (y2 / CELL_SIZE) as i32);
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

// TODO pause button
// TODO loop speed setting
// TODO draw option