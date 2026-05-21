use winit::keyboard::KeyCode;
use winit::event::KeyEvent;

use crate::rect;
use crate::util::{clamp};

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
    pub fn new( position: [f32; 2], size: [f32; 2], color: [f32; 4], camera_position: [f32; 2], device: &wgpu::Device, bind_group_layout: &wgpu::BindGroupLayout, screen_size: [f32; 2],) -> Self {
        Player {
            rect: rect::Rect {
                rect_object: rect::RectObject { position, size, color },
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
            top_speed: 150.0, // todo remove magic numbers and put them in a config file or something 
            speed: 100.0,
            acceleration: 1.0,
            jump_strength: 150.0,
            friction: 0.8,
            gravity: 100.0,
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
            println!("jumping");
        }
        
        self.velocity[1] += self.gravity * dt;

        self.velocity[0] = clamp(self.velocity[0], -self.top_speed, self.top_speed);
        self.velocity[1] = clamp(self.velocity[1], -self.jump_strength, self.top_speed );

        self.rect.move_by([
            self.velocity[0] * dt,
            self.velocity[1] * dt,
        ]);
    }
}
