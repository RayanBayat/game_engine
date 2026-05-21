pub struct Camera {
    pub postion: [f32; 2],
}

impl Camera {
    pub fn new(position: [f32; 2]) -> Self {
        Self { postion: position }
    }

    pub fn update(&mut self, player_pos: [f32; 2], screen_size: [f32; 2]) {
        self.postion[0] = player_pos[0] - screen_size[0] / 2.0;
        self.postion[1] = player_pos[1] - screen_size[1] / 2.0;
    }

    pub fn position(&self) -> [f32; 2] {
        self.postion
    }
}