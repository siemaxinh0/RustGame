use bevy::app::{App, Plugin, Update};
use bevy::math::Vec3;
use bevy::prelude::*;


#[derive (Component,Debug)]
pub struct Velocity {
    pub value: Vec3
}

#[derive(Resource)]
pub struct MapBounds {
    pub x_min: f32,
    pub x_max: f32,
    pub y_min: f32,
    pub y_max: f32,
}

pub struct MovementPlugin;

impl Plugin for MovementPlugin{
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (update_position, enforce_map_bounds.after(update_position), draw_map_bounds));

    }
}

fn update_position(mut query: Query<(&Velocity, &mut Transform)>,time : Res<Time>){
    for (velocity, mut transform) in query.iter_mut() {
        transform.translation += velocity.value * time.delta_secs();
    }
}

fn enforce_map_bounds(mut query: Query<(&mut Transform, &mut Sprite), With<Velocity>>, bounds: Res<MapBounds>,) {
    for (mut transform, sprite) in query.iter_mut() {

        let sprite_size = sprite.custom_size.unwrap_or(Vec2::new(32.0, 32.0));

        let half_width = sprite_size.x / 2.0;
        let half_height = sprite_size.y / 2.0;

        let clamped_x = transform.translation.x.clamp(
            bounds.x_min + half_width,
            bounds.x_max - half_width,
        );

        let clamped_y = transform.translation.y.clamp(
            bounds.y_min + half_height,
            bounds.y_max - half_height,
        );

        transform.translation.x = clamped_x;
        transform.translation.y = clamped_y;
    }
}

fn draw_map_bounds(mut gizmos: Gizmos, bounds: Res<MapBounds>) {
    let bottom_left = Vec2::new(bounds.x_min, bounds.y_min);
    let top_left = Vec2::new(bounds.x_min, bounds.y_max);
    let top_right = Vec2::new(bounds.x_max, bounds.y_max);
    let bottom_right = Vec2::new(bounds.x_max, bounds.y_min);

    let color = Color::srgb(1.0, 0.0, 0.0);

    gizmos.line_2d(bottom_left, top_left, color);
    gizmos.line_2d(top_left, top_right, color);
    gizmos.line_2d(top_right, bottom_right, color);
    gizmos.line_2d(bottom_right, bottom_left, color);
}
