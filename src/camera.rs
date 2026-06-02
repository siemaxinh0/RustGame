use bevy::app::{App, Plugin};
use bevy::camera::ScalingMode;
use bevy::prelude::*;

pub struct CameraPlugin;

impl Plugin for CameraPlugin{
    fn build(&self, app: &mut App) {
        app.add_systems(Startup,spawn_camera);
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical { viewport_height: 200.0 },
            scale: 1.0,
            ..OrthographicProjection::default_2d()
        }),
    ));
}
