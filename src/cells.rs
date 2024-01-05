use std::collections::HashMap;
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
    to_despawn: HashMap<(i32, i32), bool>, 
}

pub struct CellsPlugin;

impl Default for CellMap {
    fn default() -> Self {
        CellMap {
            current: HashMap::new(),
            to_spawn: HashMap::new(),
            to_despawn: HashMap::new(),
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

fn generate_random_cells(
    mut cell_map: ResMut<CellMap>,
) {
    let mut rng = rand::thread_rng();
    let perlin = Perlin::new(rng.gen());

    const NOICE_FACTOR: f64 = 5.5;
    const NOICE_THRESHOLD: f64 = 0.2;
    const INITIAL_WORLD_SIZE: i32 = 500;
    
    let half_world_size = INITIAL_WORLD_SIZE / 2;
    for x in -half_world_size..half_world_size {
        for y in -half_world_size..half_world_size {
            let noise = perlin.get([x as f64 / NOICE_FACTOR, y as f64 / NOICE_FACTOR]);
            // print!("{} \n", noise);
            if noise < NOICE_THRESHOLD {
                continue;
            }
            cell_map.to_spawn.insert((x, y), true);
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
    query: Query<(Entity, &Transform), With<Cell>>
) {


    for (entity, transform) in query.iter() {
        let x = transform.translation.x as i32;
        let y = transform.translation.y as i32;
        let is_existing = cell_map.current.get(&(x, y));
        let is_dead = cell_map.to_despawn.get(&(x, y));
        if is_dead == Some(&true) || is_existing != Some(&true) {
            commands.entity(entity).despawn_recursive();
            cell_map.current.remove(&(x, y));
            cell_map.to_despawn.remove(&(x, y));
        }
    }

    let to_spawn = cell_map.to_spawn.clone();
    for ( key, _) in to_spawn.iter() {
        let x = key.0;
        let y = key.1;
        cell_map.to_spawn.remove(&(x, y));
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
    cell_map.to_despawn.clear();
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

    let mut to_spawn: HashMap<(i32, i32), bool> = HashMap::new();
    let mut to_despawn: HashMap<(i32, i32), bool> = HashMap::new();

    let mut checked_cells: HashMap<(i32, i32), bool> = HashMap::new();

    for (key, _) in cell_map.current.iter() {
        
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
                
                let is_alive = cell_map.current.get(&(x2, y2));
                if is_alive == Some(&true) {
                    neighbours_count += 1;
                    continue;
                }
                
                if checked_cells.get(&(x2, y2)) == Some(&true) {
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
                     
                        
                        let is_alive = cell_map.current.get(&(x3, y3));
                        if is_alive == Some(&true) {
                            sub_neighbours_count += 1;
                        }
                    }
                }
                if sub_neighbours_count == 3 {
                    to_spawn.insert((x2, y2), true);
                }
                checked_cells.insert((x2, y2), true);
            }
        }

        if neighbours_count < 2 || neighbours_count > 3 {
            to_despawn.insert((x, y), true);
        }
    }
    cell_map.to_spawn = to_spawn;
    cell_map.to_despawn = to_despawn;
}

fn draw_cursor(
    camera_query: Query<(&Camera, &GlobalTransform)>,
    cells_query: Query<&Transform, With<Cell>>,
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
        let new_x = (point.x - 0.5).ceil() as i32;
        let new_y = (point.y - 0.5).ceil() as i32;

        if cells_query.iter().any(|transform| {
            let x = transform.translation.x as i32;
            let y = transform.translation.y as i32;
            x == new_x && y == new_y
        }) {
            return;
        }
    
        cell_map.to_spawn.insert((new_x, new_y), true);
    }
}

pub fn reset_cells(
    mut cell_map: ResMut<CellMap>,
) {
    cell_map.current.clear();
    cell_map.to_despawn.clear();
    cell_map.to_spawn.clear();
    spawn_initial_cells(cell_map);
}