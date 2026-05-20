use winit::keyboard::KeyCode;

use crate::rect;


pub struct Player {
    pub rect: rect::Rect,
    speed: f32,
}

impl Player {
    pub fn new( position: [f32; 2], size: [f32; 2], device: &wgpu::Device, bind_group_layout: &wgpu::BindGroupLayout, screen_size: [f32; 2],) -> Self {
        Player {
            rect: rect::Rect {
                rect_object: rect::RectObject { position, size },
                render_rect: rect::create_render_rect(
                    device,
                    bind_group_layout,
                    position,
                    size,
                    screen_size,
                ),
            },
            speed: 5.0,
        }
    }

    pub fn handle_key(&mut self, code: KeyCode) -> bool {
        match code {
            KeyCode::KeyW | KeyCode::ArrowUp => {
                self.rect.move_by([0.0, -self.speed]); // Move up by the player's speed
                true
            }
            KeyCode::KeyA | KeyCode::ArrowLeft => {
                self.rect.move_by([-self.speed, 0.0]); // Move left by the player's speed
                true
            }
            KeyCode::KeyS | KeyCode::ArrowDown => {
                self.rect.move_by([0.0, self.speed]); // Move down by the player's speed
                true
            }
            KeyCode::KeyD | KeyCode::ArrowRight => {
                self.rect.move_by([self.speed, 0.0]); // Move right by the player's speed
                true
            }
            _ => false,
        }
    }
}
