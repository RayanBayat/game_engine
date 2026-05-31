use crate::config::*;

pub struct Animation {
    pub speed: f32, 
}

impl Animation {
    pub fn new() -> Self{
        return Self {
            speed: 0.05,
        }
    }

    pub fn spin(&mut self, grounded: bool, rotation: &mut f32) {
        if grounded {
            *rotation = 0.0;
        } else {
            *rotation += PLAYER_ANIMATION_SPEED;
        }
    }
}