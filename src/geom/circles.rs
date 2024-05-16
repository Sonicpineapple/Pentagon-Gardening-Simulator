use hypermath::collections::approx_hashmap::{ApproxHashMapKey, FloatHash};

use crate::gfx::CircleInstance;

pub(crate) struct Circle {
    pub centre: [f32; 2],
    pub radius: f32,
    pub col: [f32; 4],
}
impl Circle {
    pub fn get_instance(&self, scale: [f32; 2]) -> CircleInstance {
        CircleInstance {
            col: self.col,
            centre: [self.centre[0] * scale[0], self.centre[1] * scale[1]],
            scale: [scale[0] * self.radius, scale[1] * self.radius],
        }
    }
}

use crate::Pos;
#[derive(Debug, Clone)]
pub(crate) struct RotCircle {
    pub cen: Pos,
    pub rad: f64,
    pub step: u32,
    pub inverted: bool,
}
impl RotCircle {
    pub fn new(cen: Pos, rad: f64, step: u32, inverted: bool) -> Self {
        Self {
            cen,
            rad,
            step,
            inverted,
        }
    }

    pub fn rotate_point(&self, point: Pos) -> Pos {
        let theta = std::f64::consts::TAU / self.step as f64;
        let x = point.x - self.cen.x;
        let y = point.y - self.cen.y;
        let x2 = theta.cos() * x + theta.sin() * y;
        let y2 = theta.cos() * y - theta.sin() * x;
        let x = x2 + self.cen.x;
        let y = y2 + self.cen.y;

        Pos { x, y }
    }

    pub fn rotate_circle(&self, circle: &Self) -> Self {
        Self {
            cen: self.rotate_point(circle.cen),
            ..circle.clone()
        }
    }

    pub fn contains(&self, point: &Pos) -> bool {
        (self.cen.dist_sq(point) < self.rad * self.rad) ^ self.inverted
    }
}

impl ApproxHashMapKey for RotCircle {
    type Hash = (<Pos as ApproxHashMapKey>::Hash, FloatHash, u32);

    fn approx_hash(
        &self,
        mut float_hash_fn: impl FnMut(
            hypermath::prelude::Float,
        ) -> hypermath::collections::approx_hashmap::FloatHash,
    ) -> Self::Hash {
        (
            self.cen.approx_hash(&mut float_hash_fn),
            float_hash_fn(self.rad),
            self.step,
        )
    }
}
