use eframe::egui::Pos2;

#[derive(Debug, Default, Copy, Clone)]
pub(crate) struct Pos {
    pub x: f64,
    pub y: f64,
}
impl From<Pos2> for Pos {
    fn from(value: Pos2) -> Self {
        Self {
            x: value.x as f64,
            y: value.y as f64,
        }
    }
}
impl From<Pos> for Pos2 {
    fn from(value: Pos) -> Self {
        Self {
            x: value.x as f32,
            y: value.y as f32,
        }
    }
}
impl From<Pos> for [f32; 2] {
    fn from(value: Pos) -> Self {
        [value.x as f32, value.y as f32]
    }
}
impl Pos {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
    pub fn dist_sq(self, other: &Pos) -> f64 {
        (self.x - other.x).powi(2) + (self.y - other.y).powi(2)
    }
    pub fn dist(self, other: &Pos) -> f64 {
        self.dist_sq(other).sqrt()
    }
}

impl hypermath::collections::approx_hashmap::ApproxHashMapKey for Pos {
    type Hash = [hypermath::collections::approx_hashmap::FloatHash; 2];

    fn approx_hash(
        &self,
        float_hash_fn: impl FnMut(
            hypermath::prelude::Float,
        ) -> hypermath::collections::approx_hashmap::FloatHash,
    ) -> Self::Hash {
        [self.x as f64, self.y as f64].map(float_hash_fn)
    }
}
