use winit::keyboard::KeyCode;
use winit::event::KeyEvent;

use crate::rect;
use crate::util::normalize;

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
    speed: f32,
    velocity: [f32; 2],
    direction: [f32; 2],
    jump_strength: f32,
    gravity: f32,
    pub grounded: bool,
}

impl Player {
    pub fn new( position: [f32; 2], size: [f32; 2], color: [f32; 4], device: &wgpu::Device, bind_group_layout: &wgpu::BindGroupLayout, screen_size: [f32; 2],) -> Self {
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
                ),
            },
            speed: 150.0,
            state: InputState::default(),
            velocity: [0.0; 2],
            direction: [0.0; 2],
            jump_strength: 10000.0,
            gravity: 0.0,
            grounded: false,
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
        // Here you can add any additional logic that needs to run every frame, such as collision detection or animations.
        self.direction = [0.0; 2]; // Reset velocity before applying input

        if self.state.up {
            //TODO add a check to see if the player is on the ground before allowing them to jump
            self.direction[1] = -1.0;
        }
        if self.state.down { // dont need to go down, gravity will do that for us
            self.direction[1] = 1.0;
        }
        if self.state.left {
            self.direction[0] = -1.0;
        }
        if self.state.right {
            self.direction[0] = 1.0;
        }

        let direction = normalize(self.direction);
        self.velocity = [
            direction[0] * self.speed * dt,
            direction[1] * self.speed * dt,
        ];

        self.velocity[1] += self.gravity * dt;
        self.rect.move_by(self.velocity)
    }
}
