use bevy::app::{App, Plugin, Update};
use bevy::math::Vec3;
use bevy::prelude::*;
use crate::state::GameState;


#[derive (Component,Debug)]
pub struct Velocity {
    pub value: Vec3
}

#[derive (Component,Debug)]
pub enum FacingDirection {
    Up,
    Down,
    Left,
    Right,
}

pub fn direction_from_velocity(velocity: Vec3) -> FacingDirection {
    if velocity.x > 0.0 {
        FacingDirection::Right
    } else if velocity.x < 0.0 {
        FacingDirection::Left
    } else if velocity.y > 0.0 {
        FacingDirection::Up
    } else {
        FacingDirection::Down
    }
}

pub fn atlas_index(direction: &FacingDirection, frame: usize) -> usize {
    let row = match direction {
        FacingDirection::Up => 0,
        FacingDirection::Down => 1,
        FacingDirection::Left => 2,
        FacingDirection::Right => 3,
    };
    row * 4 + frame
}

pub struct MovementPlugin;

impl Plugin for MovementPlugin{
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_position.run_if(in_state(GameState::Playing)));
        // (update_position,draw_map_bounds));
    }
}

fn update_position(mut query: Query<(&Velocity, &mut Transform)>,time : Res<Time>){
    for (velocity, mut transform) in query.iter_mut() {
        transform.translation += velocity.value * time.delta_secs();
    }
}



// fn draw_map_bounds(mut gizmos: Gizmos, bounds: Res<MapBounds>) {
//     let bottom_left = Vec2::new(bounds.x_min, bounds.y_min);
//     let top_left = Vec2::new(bounds.x_min, bounds.y_max);
//     let top_right = Vec2::new(bounds.x_max, bounds.y_max);
//     let bottom_right = Vec2::new(bounds.x_max, bounds.y_min);
//
//     let color = Color::srgb(1.0, 0.0, 0.0);
//
//     gizmos.line_2d(bottom_left, top_left, color);
//     gizmos.line_2d(top_left, top_right, color);
//     gizmos.line_2d(top_right, bottom_right, color);
//     gizmos.line_2d(bottom_right, bottom_left, color);
// }
