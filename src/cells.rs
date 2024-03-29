use std::{collections::HashMap, ops::Range};
use bevy::prelude::*;
use rand::Rng;
use noise::{NoiseFn, Perlin};

use crate::{WORLD_EDGE, IsPaused, AgeTimer, Age};

#[derive(Component, Debug)]
pub struct Cell;

#[derive(Resource)]
pub struct CellMap {
    current: HashMap<(i32, i32), bool>,
    to_spawn: HashMap<(i32, i32), bool>,
}

pub struct CellsPlugin;

impl Default for CellMap {
    fn default() -> Self {
        CellMap {
            current: HashMap::new(),
            to_spawn: HashMap::new(),
        }
    }
}

impl Plugin for CellsPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<CellMap>()
            .add_systems(Startup, setup)
            .add_systems(Update, check_cells)
            .add_systems(Update, draw_cursor)
            .add_systems(Update, apply_age);
    }
}

fn setup(
    cell_map: ResMut<CellMap>,
) {
    spawn_initial_cells(cell_map);
}

 struct NoiceLayer {
        factor: f64,
        threshold: Range<f64>,
    }

fn generate_random_cells(
    mut cell_map: ResMut<CellMap>,
) {
    let mut rng = rand::thread_rng();
    let perlin = Perlin::new(rng.gen());

    let noice_layers = vec![
        NoiceLayer {
            factor: 250.0,
            threshold: -0.5..1.0,
        },
        NoiceLayer {
            factor: 6.0,
            threshold: 0.3..0.7,
        },
        NoiceLayer {
            factor: 30.0,
            threshold: -0.3..0.8,
        },
    ];

    const INITIAL_WORLD_SIZE: i32 = 600;
    
    let half_world_size = INITIAL_WORLD_SIZE / 2;
    for x in -half_world_size..half_world_size {
        for y in -half_world_size..half_world_size {

            let mut spawn_flag = true;
            for noice_layer in noice_layers.iter() {
    
                let distance_from_center = ((x * x + y * y) as f64).sqrt();
                let max_distance = half_world_size as f64;
          
                let circle_factor = 1. - (distance_from_center / max_distance as f64) * 0.75;
           
                let outside_circle = distance_from_center > max_distance;

                let factor = noice_layer.factor;
                let threshold = &noice_layer.threshold;
                let noise = perlin.get([x as f64 / factor, y as f64 / factor]) * circle_factor;
                let passed_threshold = noise > threshold.start && noise < threshold.end;
                if !passed_threshold || outside_circle {
                    spawn_flag = false;
                    continue;
                }
            }
            cell_map.to_spawn.insert((x, y), spawn_flag);
        }   
    }
}

fn spawn_initial_cells(
    cell_map: ResMut<CellMap>
) {
    generate_random_cells(cell_map);
}

fn apply_age(
    mut commands: Commands,
    mut cell_map: ResMut<CellMap>,
    query: Query<(Entity, &Transform), With<Cell>>,
) {

    for (entity, transform) in query.iter() {
        let x = transform.translation.x as i32;
        let y = transform.translation.y as i32;
        let is_existing = cell_map.current.get(&(x, y));
        if is_existing != Some(&true) {
            commands.entity(entity).despawn_recursive();
        }
    }

    let to_spawn = cell_map.to_spawn.clone();
    for (key, value) in to_spawn.iter() {
        if !value {
            continue;
        }
        let x = key.0;
        let y = key.1;
        cell_map.to_spawn.remove(&(x, y));
        if cell_map.current.get(&(x, y)) == Some(&true) {
            continue;
        }
        if x.abs() > WORLD_EDGE || y.abs() > WORLD_EDGE {
            continue;
        }
        cell_map.current.insert((x, y), true);
        commands.spawn((
            SpriteBundle {
                transform: Transform::from_xyz(x as f32, y as f32, 0.),
                ..default()
            },
            Cell,
        ));
    }
    cell_map.to_spawn.clear();
}

fn check_cells(
    time: Res<Time>,
    mut timer: ResMut<AgeTimer>,
    is_paused: Res<IsPaused>,
    mut age: ResMut<Age>,
    mut cell_map: ResMut<CellMap>,
) {

    if is_paused.0 {
        return;
    }

    timer.0.tick(time.delta());


    if !timer.0.finished() {
        return;
    }
    
    age.0 += 1;
    
    let old_cells = cell_map.current.clone();

    let mut checked_cells: HashMap<(i32, i32), bool> = HashMap::new();

    for (key, _) in old_cells.iter() {
        
        let x = key.0;
        let y = key.1;

        let mut neighbours_count = 0;
        for i in -1..2 {
            for j in -1..2 {
                if i == 0 && j == 0 {
                    continue;
                }
                let x2 = x + i;
                let y2 = y + j;
                
                let is_alive = old_cells.get(&(x2, y2));
                if is_alive == Some(&true) {
                    neighbours_count += 1;
                    continue;
                }
                
                let is_already_checked = checked_cells.get(&(x2, y2));
                if is_already_checked == Some(&true) {
                    continue;
                }

                let is_already_spawned = cell_map.to_spawn.get(&(x2, y2));
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
                        
                        let is_alive = old_cells.get(&(x3, y3));
                        if is_alive == Some(&true) {
                            sub_neighbours_count += 1;
                        }
                    }
                }

                if sub_neighbours_count == 3 {
                    cell_map.to_spawn.insert((x2, y2), true);
                }

                checked_cells.insert((x2, y2), true);
            }
        }

        if neighbours_count < 2 || neighbours_count > 3 {
            cell_map.current.remove(&(x, y));
        }
    }
}

fn draw_cursor(
    camera_query: Query<(&Camera, &GlobalTransform)>,
    windows: Query<&Window>,
    buttons: Res<Input<MouseButton>>,
    mut cell_map: ResMut<CellMap>,
) {
    let (camera, camera_transform) = camera_query.single();

    let Some(cursor_position) = windows.single().cursor_position() else {
        return;
    };

    let Some(point) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };

    if buttons.pressed(MouseButton::Left) {
        let x = (point.x - 0.5).ceil() as i32;
        let y = (point.y - 0.5).ceil() as i32;
        cell_map.to_spawn.insert((x, y), true);
    
    }
}

pub fn reset_cells(
    mut cell_map: ResMut<CellMap>,
) {
    cell_map.current.clear();
    cell_map.to_spawn.clear();
    spawn_initial_cells(cell_map);
}