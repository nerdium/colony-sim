use std::collections::VecDeque;

use godot::global::MouseButton;
use godot::prelude::*;
use godot::classes::{Camera2D, ICamera2D, Input, InputEvent, InputEventMouseButton};

use crate::action::{ActionQueue, Wait};
use crate::task::{Action, TaskRunner};

// TODO: middle mouse button panning
// TODO: zoom based off of mouse position

#[derive(GodotClass)]
#[class(base = Camera2D)]
pub struct PlayerCamera {
    // zoom controls
    #[export] zoom_speed: f32,
    #[export] zoom_smoothing: f32,
    #[export] min_zoom: f32,
    #[export] max_zoom: f32,

    // pan controls
    #[export] pan_speed: f32,
    #[export] pan_smoothing: f32,

    target_zoom: f32,
    target_position: Vector2,

    base: Base<Camera2D>
}

#[godot_api]
impl ICamera2D for PlayerCamera {
    fn init(base: Base<Camera2D>) -> Self {
        Self {
            zoom_speed: 1.1,
            zoom_smoothing: 10.0,
            min_zoom: 0.25,
            max_zoom: 4.0,

            pan_speed: 600.0,
            pan_smoothing: 10.0,

            target_zoom: 1.0,
            target_position: Vector2::ZERO,

            base
        }
    }

    fn ready(&mut self) {
        self.target_zoom = self.base().get_zoom().x;
        self.target_position = self.base().get_position();
    }

    fn process(&mut self, delta: f64) {
        let current_zoom = self.base().get_zoom().x;
        let current_pos = self.base().get_position();

        // handle panning input
        let pan_input = Input::singleton().get_vector("move_left", "move_right", "move_up", "move_down");
        self.target_position += pan_input * (self.pan_speed / current_zoom) * delta as f32;

        // smooth pan
        let pan_delta = 1.0 - (-(self.pan_smoothing as f64) * delta).exp() as f32;
        let new_pos = current_pos.lerp(self.target_position, pan_delta);
        self.base_mut().set_position(new_pos);

        // smooth zoom
        let zoom_delta = 1.0 - (-(self.zoom_smoothing as f64) * delta).exp() as f32;
        let new_zoom = current_zoom.lerp(self.target_zoom, zoom_delta);
        self.base_mut().set_zoom(Vector2::splat(new_zoom));
    }

    fn unhandled_input(&mut self, event: Gd<InputEvent>) {
        let Ok(mb) = event.try_cast::<InputEventMouseButton>() else {
            return;
        };

        if !mb.is_pressed() { return; }

        match mb.get_button_index() {
            MouseButton::WHEEL_UP => self.target_zoom *= self.zoom_speed,
            MouseButton::WHEEL_DOWN => self.target_zoom /= self.zoom_speed,
            _ => return
        }

        self.target_zoom = self.target_zoom.clamp(self.min_zoom, self.max_zoom);
    }
}