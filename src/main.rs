
use bevy::prelude::*;
use bevy_pancam::{PanCam, PanCamPlugin};

mod ui;
mod cells;

use ui::ButtonsPlugin;
use cells::CellsPlugin;

#[derive(Resource)]
pub struct AgeTimer(Timer);

#[derive(Resource, Default)]
struct Age(i32); // TODO AgeCounter

#[derive(Resource, Default)]
pub struct IsPaused(bool);

// #[derive(Resource)]
// struct StopPropagation(bool);

// TODO generate by perelin noise
// TODO add keyboard shortcuts
// TODO space to move camera
// TODO asset server
// TODO remove `to_despawn`

pub const MIN_AGE_DURATION: f32 = 0.01;
pub const MAX_AGE_DURATION: f32 = 10.;
pub const WORLD_EDGE: i32 = 4000;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PanCamPlugin::default()))        

        .insert_resource(Age(0))
        .insert_resource(IsPaused(false))
        .insert_resource(AgeTimer(Timer::from_seconds(
            0.1,
            TimerMode::Repeating,
        )))
        .add_systems(Startup, setup)        
        .add_plugins(ButtonsPlugin)
        .add_plugins(CellsPlugin)
        .run();
}


fn normolize_speed(speed: f32) -> f32 {
    if speed < MIN_AGE_DURATION {
        return MIN_AGE_DURATION
    }
    if speed > MAX_AGE_DURATION {
        return MAX_AGE_DURATION
    }
    speed
}

pub fn speed_up_age_speed(timer: &mut ResMut<AgeTimer>) {
    timer.0 = Timer::from_seconds(
        normolize_speed(timer.0.duration().as_secs_f32() * 0.5),
        TimerMode::Repeating,
    );
}

pub fn slow_age_speed(timer: &mut ResMut<AgeTimer>) {
    timer.0 = Timer::from_seconds(
        normolize_speed(timer.0.duration().as_secs_f32() * 2.),
        TimerMode::Repeating,
    );
}

pub fn toggle_pause(is_paused: &mut ResMut<IsPaused>) {
    is_paused.0 = !is_paused.0;
}

fn setup(
    mut commands: Commands
) {
    commands.spawn((
        Camera2dBundle::default(),
        PanCam {
            grab_buttons: vec![MouseButton::Right, MouseButton::Middle],
            ..default()
        }
    ));
}
