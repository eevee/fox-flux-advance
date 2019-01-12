pub mod shapes;

use arrayvec::ArrayVec;

use crate::geom::Vector;
use self::shapes::Collision;

const MAX_COLLISIONS: usize = 8;
pub type CollisionVec = ArrayVec<[Collision; MAX_COLLISIONS]>;

pub struct CollisionResult {
    pub allowed: Vector,
    pub collisions: CollisionVec,
}
