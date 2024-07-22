use glam::Vec3;

/// Defines utility functions intended for vector types
pub trait VecUtils {
    /// Projects self on to target
    fn proj(self, target: Self) -> Self;

    /// Anti projects self on to target
    fn anti_proj(self, target: Self) -> Self;

    /// Normalizes self and gives it the specified length
    fn rescale(self, length: f32) -> Self;

    /// Mirrors self on the given normal
    fn mirror(self, normal: Vec3) -> Self;

    fn ndot(self, other: Self) -> f32;

    // perhaps move these to Vec3Utils
    fn intersects_in_xz(self, other: Self) -> bool;
    fn intersection_in_xz(self, self_dir: Self, other: Self, other_dir: Self) -> Self;
    fn side(self, other: Self) -> f32;
    fn right_hand(self) -> Self;
    fn left_hand(self) -> Self;
    fn flip(self, flip: bool) -> Self;
}

impl VecUtils for Vec3 {
    fn proj(self, target: Self) -> Self {
        target * (self.dot(target) / target.length_squared())
    }

    fn anti_proj(self, target: Self) -> Self {
        self - self.proj(target)
    }

    fn rescale(self, length: f32) -> Self {
        self.normalize() * length
    }

    fn mirror(self, normal: Self) -> Self {
        self - self.proj(normal) * 2.0
    }

    fn ndot(self, other: Self) -> f32 {
        self.normalize().dot(other.normalize())
    }

    fn intersects_in_xz(self, other: Self) -> bool {
        // TODO use .xz()? and dot?
        other.x * self.z - other.z * self.x != 0.0
    }

    fn side(self, other: Self) -> f32 {
        (self.z * other.x - self.x * other.z).signum()
    }

    fn intersection_in_xz(self, self_dir: Self, other: Self, other_dir: Self) -> Self {
        other
            + (other_dir * ((other.z - self.z) * self_dir.x - (other.x - self.x) * self_dir.z)
                / (other_dir.x * self_dir.z - other_dir.z * self_dir.x))
    }

    /// Should be removed and only be in dir2
    fn left_hand(self) -> Self {
        Self::new(self.z, self.y, -self.x)
    }

    /// Should be removed and only be in dir2
    fn right_hand(self) -> Self {
        Self::new(-self.z, self.y, self.x)
    }

    fn flip(self, flip: bool) -> Self {
        if flip {
            self * -1.
        } else {
            self
        }
    }
}
