use super::grip::Grip;

#[derive(Debug, Clone)]
pub(crate) struct Piece {
    grips: Vec<Grip>,
}
impl Piece {
    pub fn new(grips: Vec<Grip>) -> Self {
        Self { grips }
    }
    pub fn grips(&self) -> &Vec<Grip> {
        &self.grips
    }
}
