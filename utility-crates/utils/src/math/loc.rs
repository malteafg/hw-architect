use glam::Vec3;
use serde::{Deserialize, Serialize};

use super::dir::DirXZ;

/// Represents a position in xyz and a direction in xz. Maybe rename to Loc2 to reflect dir only
/// being in xz
#[derive(Copy, Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct Loc {
    pub pos: Vec3,
    pub dir: DirXZ,
}

impl Loc {
    pub fn new(pos: Vec3, dir: DirXZ) -> Self {
        Self { pos, dir }
    }

    pub fn flip(self, flip: bool) -> Self {
        Loc::new(self.pos, self.dir.flip(flip))
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum PosOrLoc {
    Pos(Vec3),
    Loc(Loc),
}

impl PosOrLoc {
    pub fn flip(self, flip: bool) -> Self {
        match self {
            PosOrLoc::Pos(_) => self,
            PosOrLoc::Loc(loc) => PosOrLoc::Loc(loc.flip(flip)),
        }
    }

    pub fn pos(self) -> Vec3 {
        match self {
            PosOrLoc::Pos(pos) => pos,
            PosOrLoc::Loc(loc) => loc.pos,
        }
    }

    pub fn is_pos(self) -> bool {
        match self {
            PosOrLoc::Pos(_) => true,
            PosOrLoc::Loc(_) => false,
        }
    }

    pub fn is_loc(self) -> bool {
        match self {
            PosOrLoc::Pos(_) => false,
            PosOrLoc::Loc(_) => true,
        }
    }

    pub fn to_pos(self) -> Self {
        match self {
            PosOrLoc::Pos(_) => self,
            PosOrLoc::Loc(loc) => PosOrLoc::Pos(loc.pos),
        }
    }
}

impl From<Vec3> for PosOrLoc {
    fn from(value: Vec3) -> Self {
        PosOrLoc::Pos(value)
    }
}

impl From<Loc> for PosOrLoc {
    fn from(value: Loc) -> Self {
        PosOrLoc::Loc(value)
    }
}
