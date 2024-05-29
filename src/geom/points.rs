use std::ops::{Add, Div, Mul, Neg, Sub};

use eframe::egui::Pos2;

use super::Curvature;

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
    pub const ORIGIN: Self = Pos { x: 0., y: 0. };
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
    pub fn dist_sq(self, other: &Pos) -> f64 {
        (self.x - other.x).powi(2) + (self.y - other.y).powi(2)
    }
    pub fn dist(self, other: &Pos) -> f64 {
        self.dist_sq(other).sqrt()
    }

    pub fn dist_in_space(self, other: &Pos, curvature: Curvature) -> f64 {
        match curvature {
            Curvature::Spherical => {
                let a = (*other - self) / (self.conjugate() * *other + Pos::new(1., 0.));
                2. * a.dist(&Pos::ORIGIN).atan()
            }
            Curvature::Euclidean => self.dist(other),
            Curvature::Hyperbolic => {
                let a = (*other - self) / (-self.conjugate() * *other + Pos::new(1., 0.));
                2. * a.dist(&Pos::ORIGIN).atanh()
            }
        }
    }
    /// Only for spherical inverted circles
    pub fn dist_to_inf(&self, curvature: Curvature) -> f64 {
        match curvature {
            Curvature::Spherical => {
                let a = 1. / self.dist(&Pos::ORIGIN);
                2. * a.atan() * 0.8
            }
            _ => f64::INFINITY,
        }
    }

    pub fn conjugate(self) -> Self {
        Pos {
            x: self.x,
            y: -self.y,
        }
    }
}
impl Add for Pos {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Pos {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}
impl Mul<Pos> for f64 {
    type Output = Pos;

    fn mul(self, rhs: Pos) -> Self::Output {
        Pos {
            x: self * rhs.x,
            y: self * rhs.y,
        }
    }
}
impl Neg for Pos {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Pos {
            x: -self.x,
            y: -self.y,
        }
    }
}
impl Sub for Pos {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self + -rhs
    }
}

/// Complex multiplication
impl Mul for Pos {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Pos {
            x: self.x * rhs.x - self.y * rhs.y,
            y: self.x * rhs.y + self.y * rhs.x,
        }
    }
}
impl Div for Pos {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        (1. / rhs.dist_sq(&Pos::ORIGIN)) * self * rhs.conjugate()
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
