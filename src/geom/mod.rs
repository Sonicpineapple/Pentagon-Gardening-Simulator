mod circles;
mod points;

pub(crate) use circles::{Circle, GraphicsCircle, RotCircle};
pub(crate) use points::Pos;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub enum Curvature {
    Spherical,
    #[default]
    Euclidean,
    Hyperbolic,
}
