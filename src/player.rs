use winit::event::KeyEvent;
use winit::keyboard::KeyCode;

use crate::config::*;
use crate::rect;
use crate::util::clamp;

#[derive(Default)]
pub struct InputState {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
}

pub struct Player {
    pub rect: rect::Rect,
    state: InputState,
    top_speed: f32,
    speed: f32,
    acceleration: f32,
    jump_strength: f32,
    friction: f32,
    gravity: f32,
    pub velocity: [f32; 2],
    pub grounded: bool,
}

impl Player {
    pub fn new(
        position: [f32; 2],
        size: [f32; 2],
        color: [f32; 4],
        camera_position: [f32; 2],
        screen_size: [f32; 2],
        device: &wgpu::Device,
        bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let previous_position = position;
        Player {
            rect: rect::Rect {
                rect_object: rect::RectObject {
                    position,
                    size,
                    color,
                    previous_position,
                },
                render_rect: rect::create_render_rect(
                    device,
                    bind_group_layout,
                    position,
                    size,
                    screen_size,
                    color,
                    camera_position,
                ),
            },
            top_speed: PLAYER_TOP_SPEED,
            speed: PLAYER_SPEED,
            acceleration: PLAYER_ACCELERATION,
            jump_strength: PLAYER_JUMP_STRENGTH,
            friction: PLAYER_FRICTION,
            gravity: GRAVITY,
            state: InputState::default(),
            velocity: [0.0; 2],
            grounded: true,
        }
    }

    pub fn handle_key(&mut self, code: KeyCode, event: KeyEvent) {
        let pressed = event.state == winit::event::ElementState::Pressed;

        match code {
            KeyCode::KeyW | KeyCode::ArrowUp => {
                self.state.up = pressed;
            }
            KeyCode::KeyA | KeyCode::ArrowLeft => {
                self.state.left = pressed;
            }
            KeyCode::KeyS | KeyCode::ArrowDown => {
                self.state.down = pressed;
            }
            KeyCode::KeyD | KeyCode::ArrowRight => {
                self.state.right = pressed;
            }
            _ => (),
        }
    }

    pub fn update(&mut self, dt: f32) {
        let mut direction_x = 0.0;

        if self.state.left {
            direction_x -= self.acceleration;
        }

        if self.state.right {
            direction_x += self.acceleration;
        }

        self.velocity[0] += direction_x * self.speed * dt;

        if direction_x == 0.0 {
            self.velocity[0] *= self.friction;
        }

        if self.state.up && self.grounded {
            self.velocity[1] = -self.jump_strength;
            self.grounded = false;
        }

        self.velocity[1] += self.gravity * dt;

        self.velocity[0] = clamp(self.velocity[0], -self.top_speed, self.top_speed);
        self.velocity[1] = clamp(self.velocity[1], -self.jump_strength, self.top_speed);

        self.rect.rect_object.previous_position = self.rect.rect_object.position;

        self.rect
            .move_by([self.velocity[0] * dt, self.velocity[1] * dt]);
    }

    pub fn get_velocity(&self) -> [f32; 2] {
        return self.velocity;
    }
    pub fn set_velocity(&mut self, v: [f32; 2]) {
        self.velocity = v;
    }

    pub fn stop_horizontal_velocity(&mut self) {
        self.velocity[0] = 0.0;
    }

    pub fn stop_vertical_velocity(&mut self) {
        self.velocity[1] = 0.0;
    }
}
