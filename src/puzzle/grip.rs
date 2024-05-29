use crate::geom::Pos;

#[derive(Debug, Clone)]
pub(crate) struct Grip {
    pub pos: Pos,
    pub id: usize,
}
impl Grip {
    pub fn new(pos: Pos, id: usize) -> Self {
        Self { pos, id }
    }
}
