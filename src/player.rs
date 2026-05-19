use winit::keyboard::KeyCode;

use crate::rect;


pub struct Player {
    pub rect: rect::Rect,
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
        }
    }

    pub fn handle_key(&mut self, code: KeyCode) -> bool {
        match code {
            KeyCode::KeyW | KeyCode::ArrowUp => {
                self.rect.move_by([0.0, -2.0]); // Move up by 0.1 units
                true
            }
            KeyCode::KeyA | KeyCode::ArrowLeft => {
                self.rect.move_by([-2.0, 0.0]); // Move left by 0.1 units
                true
            }
            KeyCode::KeyS | KeyCode::ArrowDown => {
                self.rect.move_by([0.0, 2.0]); // Move down by 0.1 units
                true
            }
            KeyCode::KeyD | KeyCode::ArrowRight => {
                self.rect.move_by([2.0, 0.0]); // Move right by 0.1 units
                true
            }
            _ => false,
        }
    }
}
