mod circles;
mod mobius;
mod points;

pub(crate) use circles::{Circle, GraphicsCircle, RotCircle};
pub(crate) use mobius::MobiusTransform;
pub(crate) use points::Pos;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub enum Curvature {
    Spherical,
    #[default]
    Euclidean,
    Hyperbolic,
}
