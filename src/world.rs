use crate::camera::Camera;
use crate::config::*;
use crate::player::Player;
use crate::rect::Rect;

use std::fs::File;
use std::io::{BufRead, BufReader};

/// Represents the current game world.
///
/// The world owns:
/// - the player
/// - all placed rectangle objects
/// - the camera
/// - screen information
///
/// The world is also responsible for:
/// - updating entities
/// - collision handling
/// - camera updates
/// - loading maps from files
///
pub struct World {
    pub player: Player,
    pub items: Vec<Rect>,
    pub screen_size: [f32; 2],
    pub dimensions: [i32; 2],
    pub camera: Camera,
}

impl World {
    pub fn new(
        device: &wgpu::Device,
        rect_bind_group_layout: &wgpu::BindGroupLayout,
        screen_size: [f32; 2],
    ) -> Self {
        let camera = Camera::new(CAMERA_STARTING_POSITION);
        let player = Player::new(
            PLAYER_STARTING_POSITION,
            PLAYER_SIZE,
            PLAYER_COLOR,
            camera.position(),
            screen_size,
            device,
            rect_bind_group_layout,
        );

        let items = vec![
            // Rect::new([300.0, 300.0], [100.0, 150.0], [0.0, 1.0, 0.0, 1.0], device, rect_bind_group_layout, screen_size), // Green color
            // Rect::new([400.0, 300.0], [100.0, 150.0], [1.0, 1.0, 1.0, 1.0], device, rect_bind_group_layout, screen_size),
            // Rect::new([550.0, 150.0], [100.0, 150.0], [1.0, 1.0, 1.0, 1.0], device, rect_bind_group_layout, screen_size),
        ];

        let camera = Camera::new(player.rect.position());

        Self {
            player,
            items,
            screen_size,
            dimensions: [10, 10],
            camera,
        }
    }

    pub fn wall_collision(&mut self) {
        if self.player.rect.position()[0] < 0.0 {
            self.player
                .rect
                .move_to([0.0, self.player.rect.position()[1]]);
            self.player.stop_horizontal_velocity();
        }
        if self.player.rect.position()[0] + self.player.rect.size()[0] > self.screen_size[0] {
            self.player.rect.move_to([
                self.screen_size[0] - self.player.rect.size()[0],
                self.player.rect.position()[1],
            ]);
            self.player.stop_horizontal_velocity();
        }
        if self.player.rect.position()[1] < 0.0 {
            self.player
                .rect
                .move_to([self.player.rect.position()[0], 0.0]);
            self.player.stop_vertical_velocity();
        }
        if self.player.rect.position()[1] + self.player.rect.size()[1] > self.screen_size[1] {
            self.player.rect.move_to([
                self.player.rect.position()[0],
                self.screen_size[1] - self.player.rect.size()[1],
            ]);
            self.player.grounded = true;
            self.player.stop_vertical_velocity();
        }
    }

    pub fn object_to_player_collision(&mut self) {
        for wall in self.items.iter() {
            if !self.player.rect.intersects(wall) {
                continue;
            }

            let overlap_left =
                (self.player.rect.position()[0] + self.player.rect.size()[0]) - wall.position()[0];
            let overlap_right =
                (wall.position()[0] + wall.size()[0]) - self.player.rect.position()[0];
            let overlap_bottom =
                (wall.position()[1] + wall.size()[1]) - self.player.rect.position()[1];
            let overlap_top =
                (self.player.rect.position()[1] + self.player.rect.size()[1]) - wall.position()[1];

            let overlap_x = if overlap_left < overlap_right {
                -overlap_left
            } else {
                overlap_right
            };
            let overlap_y = if overlap_top < overlap_bottom {
                -overlap_top
            } else {
                overlap_bottom
            };

            let player_velocity = self.player.get_velocity();

            if overlap_x.abs() < overlap_y.abs() {
                if overlap_x < 0.0 && player_velocity[0] > 0.0 {
                    self.player.stop_horizontal_velocity();
                }
                if overlap_x > 0.0 && player_velocity[0] < 0.0 {
                    self.player.stop_horizontal_velocity();
                }
                self.player.rect.move_by([overlap_x, 0.0]);
            } else {
                if overlap_y < 0.0 && player_velocity[1] > 0.0 {
                    self.player.stop_vertical_velocity();
                    self.player.grounded = true;
                }
                if overlap_y > 0.0 && player_velocity[1] < 0.0 {
                    self.player.stop_vertical_velocity();
                }
                self.player.rect.move_by([0.0, overlap_y]);
            }
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.player.update(dt);

        self.player.grounded = false;

        // self.wall_collision();
        self.object_to_player_collision();
        self.camera
            .update(self.player.rect.position(), self.screen_size);
    }

    /// Reads a world map from a text file.
    ///
    /// Supported characters:
    /// - '0' = empty space
    /// - '#' = wall block
    /// - 'P' = player spawn
    ///
    /// World positions are converted from grid coordinates
    /// into screen-space positions.
    pub fn read_world(
        &mut self,
        device: &wgpu::Device,
        rect_bind_group_layout: &wgpu::BindGroupLayout,
    ) {
        let file = File::open("map.txt").expect("Unable to open world state file");
        let reader = BufReader::new(file);
        let x_step = self.screen_size[0] / self.dimensions[0] as f32;
        let y_step = self.screen_size[1] / self.dimensions[1] as f32;

        let mut y = 0;
        for line in reader.lines() {
            let mut x = 0;
            for char in line.unwrap().chars() {
                match char {
                    '0' => {
                        // Empty space, do nothing
                    }
                    '#' => {
                        self.items.push(Rect::new(
                            [x as f32 * x_step, y as f32 * y_step],
                            [x_step, y_step],
                            WALL_COLOR, // Green color
                            self.camera.position(),
                            self.screen_size,
                            device,
                            rect_bind_group_layout,
                        ));
                    }
                    'P' => {
                        self.player = Player::new(
                            [x as f32 * x_step, y as f32 * y_step],
                            PLAYER_SIZE,
                            PLAYER_COLOR, // Red color
                            self.camera.position(),
                            self.screen_size,
                            device,
                            rect_bind_group_layout,
                        );
                    }
                    _ => println!("Unknown character in world state file: {}", char),
                }
                x += 1;
            }
            y += 1;
        }
    }
}
