use glam::Vec3;
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Tree {
    pos: Vec3,
    yrot: f32,
}

impl Tree {
    pub fn new(pos: Vec3) -> Self {
        let mut rng = rand::thread_rng();
        Self {
            pos,
            yrot: rng.gen_range(0.0..3.14),
        }
    }

    pub fn pos(&self) -> Vec3 {
        self.pos
    }

    pub fn yrot(&self) -> f32 {
        self.yrot
    }
}
