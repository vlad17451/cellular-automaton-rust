use std::collections::HashMap;
use bevy::prelude::*;

use crate::{INITIAL_CELLS, WORLD_EDGE, IsPaused, AgeTimer, Age};


#[derive(Component, Debug)]
pub struct Cell;

pub struct CellsPlugin;

impl Plugin for CellsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup)
            .add_systems(Update, check_cells)
            .add_systems(Update, draw_cursor);
    }
}

fn setup(
    mut commands: Commands
) {
    spawn_initial_cells(&mut commands);
}

fn spawn_initial_cells(
    commands: &mut Commands,
    // mut commands: Commands,
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

pub fn reset_cells(
    cells_query: &Query<Entity, With<Cell>>,
    commands: &mut Commands,
) {
    for entity in cells_query.iter() {
        // TODO mark cell as dead
        commands.entity(entity).despawn_recursive();
    }
    spawn_initial_cells(commands);
}