use std::ops::Mul;

use crate::geom::Pos;

use crate::geom::Curvature;

#[derive(Debug, Clone)]
pub(crate) struct MobiusTransform {
    transform: [[Pos; 2]; 2],
}
impl MobiusTransform {
    pub const IDENT: MobiusTransform = MobiusTransform {
        transform: [
            [Pos::new(1., 0.), Pos::new(0., 0.)],
            [Pos::new(0., 0.), Pos::new(1., 0.)],
        ],
    };

    pub fn new(transform: [[Pos; 2]; 2]) -> Self {
        Self { transform }
    }

    pub fn apply_to(&self, pos: Pos) -> Pos {
        let [[a, b], [c, d]] = self.transform;
        (a * pos + b) / (c * pos + d)
    }

    pub fn inverse(&self) -> Self {
        let [[a, b], [c, d]] = self.transform;
        Self::new([[d, -b], [-c, a]])
    }

    pub fn normalise(&mut self, curvature: Curvature) {
        let [[a, b], [c, d]] = &mut self.transform;
        match curvature {
            Curvature::Spherical => {
                let scale = 1. / (a.dist_sq(&Pos::ORIGIN) + b.dist_sq(&Pos::ORIGIN));
                *a = scale * *a;
                *b = scale * *b;
                *d = (a.dist(&Pos::ORIGIN) / d.dist(&Pos::ORIGIN)) * *d;
                *c = -1. * *d * b.conjugate() / a.conjugate();
            }
            Curvature::Euclidean => {
                *c = Pos::new(0., 0.);
                *a = (1. / a.dist(&Pos::ORIGIN)) * *a;
                *d = (1. / d.dist(&Pos::ORIGIN)) * *d;
            }
            Curvature::Hyperbolic => {
                let scale = 1. / (a.dist_sq(&Pos::ORIGIN) + b.dist_sq(&Pos::ORIGIN));
                *a = scale * *a;
                *b = scale * *b;
                *d = (a.dist(&Pos::ORIGIN) / d.dist(&Pos::ORIGIN)) * *d;
                *c = *d * b.conjugate() / a.conjugate();
            }
        }
    }
}
impl Mul for MobiusTransform {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let [[a, b], [c, d]] = self.transform;
        let [[p, q], [r, s]] = rhs.transform;

        Self::new([
            [a * p + b * r, a * q + b * s],
            [c * p + d * r, c * q + d * s],
        ])
    }
}
