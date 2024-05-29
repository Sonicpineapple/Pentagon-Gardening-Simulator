use hypermath::collections::approx_hashmap::{ApproxHashMapKey, FloatHash};

use crate::gfx::CircleInstance;

pub(crate) struct GraphicsCircle {
    pub centre: [f32; 2],
    pub radius: f32,
    pub col: [f32; 4],
}
impl GraphicsCircle {
    pub fn get_instance(&self, scale: [f32; 2]) -> CircleInstance {
        CircleInstance {
            col: self.col,
            centre: [self.centre[0] * scale[0], self.centre[1] * scale[1]],
            scale: [scale[0] * self.radius, scale[1] * self.radius],
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Circle {
    pub cen: Pos,
    pub rad: f64,
    pub curvature: Curvature,
}
impl Circle {
    pub fn new(cen: Pos, rad: f64, curvature: Curvature) -> Self {
        Self {
            cen,
            rad,
            curvature,
        }
    }

    pub fn contains(&self, point: &Pos) -> bool {
        self.cen.dist_in_space(point, self.curvature) < self.rad
    }

    pub fn euclidean_centre_radius(&self, transform: &MobiusTransform) -> (Pos, f64) {
        let transformed_cen = transform.apply_to(self.cen);
        match self.curvature {
            Curvature::Spherical => {
                // let t = (self.rad.tan().powi(2) + 1.).sqrt();
                // let ap = (t - 1.) / self.rad.tan();
                // let qp =
                //     (ap + self.cen.dist(&Pos::ORIGIN)) / (1. - ap * self.cen.dist(&Pos::ORIGIN));
                // let am = (-t - 1.) / self.rad.tan();
                // let qm =
                // (am + self.cen.dist(&Pos::ORIGIN)) / (1. - am * self.cen.dist(&Pos::ORIGIN));

                let p_mod = transformed_cen.dist(&Pos::ORIGIN);
                let p_mod_sq = transformed_cen.dist_sq(&Pos::ORIGIN);

                let (s, c) = self.rad.sin_cos();
                let qp = (2. * c * p_mod - (p_mod_sq - 1.) * s)
                    / (1. + c + p_mod_sq * (1. - c) - 2. * p_mod * s);
                let qm = (2. * c * p_mod + (p_mod_sq - 1.) * s)
                    / (1. + c + p_mod_sq * (1. - c) + 2. * p_mod * s);

                let cen = if transformed_cen.dist(&Pos::ORIGIN) == 0. {
                    Pos::ORIGIN
                } else {
                    (qp + qm) / (2. * transformed_cen.dist(&Pos::ORIGIN)) * transformed_cen
                };
                let rad = (qp - qm) / 2.;
                (cen, rad.abs())
            }
            Curvature::Euclidean => (transformed_cen, self.rad),
            Curvature::Hyperbolic => {
                let p_mod = transformed_cen.dist(&Pos::ORIGIN);
                let p_mod_sq = transformed_cen.dist_sq(&Pos::ORIGIN);

                let (s, c) = (self.rad.sinh(), self.rad.cosh());
                let qp = -(-2. * c * p_mod + p_mod_sq * s + s)
                    / (c * p_mod_sq + c - p_mod_sq - 2. * p_mod * s + 1.);
                let qm = -(-2. * c * p_mod - p_mod_sq * s - s)
                    / (c * p_mod_sq + c - p_mod_sq + 2. * p_mod * s + 1.);

                let cen = if transformed_cen.dist(&Pos::ORIGIN) == 0. {
                    Pos::ORIGIN
                } else {
                    (qp + qm) / (2. * transformed_cen.dist(&Pos::ORIGIN)) * transformed_cen
                };
                let rad = (qp - qm) / 2.;
                (cen, rad.abs())
            }
        }
    }
}

use crate::Pos;

use super::{Curvature, MobiusTransform};
#[derive(Debug, Clone)]
pub(crate) struct RotCircle {
    pub circle: Circle,
    pub step: u32,
    pub inverted: bool,
}
impl RotCircle {
    pub fn new(cen: Pos, rad: f64, step: u32, curvature: Curvature, inverted: bool) -> Self {
        let circle = Circle::new(cen, rad, curvature);
        Self {
            circle,
            step,
            inverted,
        }
    }

    pub fn rotate_point(&self, point: Pos) -> Pos {
        let theta = std::f64::consts::TAU / self.step as f64;
        match self.circle.curvature {
            Curvature::Spherical => {
                // φ = exp(iθ/2)
                // p = centre

                // M = [ φ+φ¯|p|^2   p¯(-φ+φ¯) ]
                //     [ p(-φ+φ¯)    φ¯+φ|p|^2 ]

                // M * (z,1) = ( φz+φ¯z|p|^2 + p¯(-φ+φ¯) , p(-φ+φ¯)z + φ¯+φ|p|^2 )
                //           = ( φz+φ¯z|p|^2 + p¯(-φ+φ¯) , p(-φ+φ¯)z + φ¯+φ|p|^2 )

                // = (az-b¯ , bz+a¯)

                // out = az-b¯ / bz+a¯

                let phi = Pos::new((theta / 2.).cos(), (theta / 2.).sin());
                let a = phi + self.circle.cen.dist_sq(&Pos::ORIGIN) * phi.conjugate();
                let b = Pos::new(0., -2. * phi.y) * self.circle.cen.conjugate();

                let az = a * point;
                let bz = b * point;

                let num = az - b.conjugate();
                let den = bz + a.conjugate();

                num / den
            }
            Curvature::Euclidean => {
                let x = point.x - self.circle.cen.x;
                let y = point.y - self.circle.cen.y;
                let x2 = theta.cos() * x + theta.sin() * y;
                let y2 = theta.cos() * y - theta.sin() * x;
                let x = x2 + self.circle.cen.x;
                let y = y2 + self.circle.cen.y;

                Pos { x, y }
            }
            Curvature::Hyperbolic => {
                let phi = Pos::new((theta / 2.).cos(), (theta / 2.).sin());
                let a = phi - self.circle.cen.dist_sq(&Pos::ORIGIN) * phi.conjugate();
                let b = Pos::new(0., 2. * phi.y) * self.circle.cen.conjugate();

                let az = a * point;
                let bz = b * point;

                let num = az + b.conjugate();
                let den = bz + a.conjugate();

                num / den
            }
        }
    }

    pub fn rotate_circle(&self, circle: &Self) -> Self {
        Self {
            circle: Circle {
                cen: self.rotate_point(circle.circle.cen),
                ..circle.circle.clone()
            },
            ..circle.clone()
        }
    }

    pub fn contains(&self, point: &Pos) -> bool {
        self.circle.contains(point) ^ self.inverted
    }

    pub fn euclidean_centre_radius(&self, transform: &MobiusTransform) -> (Pos, f64) {
        self.circle.euclidean_centre_radius(transform)
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
            self.circle.cen.approx_hash(&mut float_hash_fn),
            float_hash_fn(self.circle.rad),
            self.step,
        )
    }
}
