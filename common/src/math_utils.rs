use cgmath::{Point2, Point3, Vector2, Vector3, Vector4};

pub trait VecPoint<A> {
    fn to_vec2(&self) -> Vector2<A>;
    fn to_vec3(&self) -> Vector3<A>;
    fn to_vec4(&self) -> Vector4<A>;
    fn to_point2(&self) -> Point2<A>;
    fn to_point3(&self) -> Point3<A>;
}

impl<A: Default + Copy> VecPoint<A> for Vector2<A> {
    fn to_vec2(&self) -> Vector2<A> {
        *self
    }
    fn to_vec3(&self) -> Vector3<A> {
        Vector3::new(self.x, self.y, A::default())
    }
    fn to_vec4(&self) -> Vector4<A> {
        Vector4::new(self.x, self.y, A::default(), A::default())
    }
    fn to_point2(&self) -> Point2<A> {
        Point2::new(self.x, self.y)
    }
    fn to_point3(&self) -> Point3<A> {
        Point3::new(self.x, self.y, A::default())
    }
}

impl<A: Default + Copy> VecPoint<A> for Vector3<A> {
    fn to_vec2(&self) -> Vector2<A> {
        Vector2::new(self.x, self.y)
    }
    fn to_vec3(&self) -> Vector3<A> {
        *self
    }
    fn to_vec4(&self) -> Vector4<A> {
        Vector4::new(self.x, self.y, self.z, A::default())
    }
    fn to_point2(&self) -> Point2<A> {
        Point2::new(self.x, self.y)
    }
    fn to_point3(&self) -> Point3<A> {
        Point3::new(self.x, self.y, self.z)
    }
}

impl<A: Default + Copy> VecPoint<A> for Vector4<A> {
    fn to_vec2(&self) -> Vector2<A> {
        Vector2::new(self.x, self.y)
    }
    fn to_vec3(&self) -> Vector3<A> {
        Vector3::new(self.x, self.y, self.z)
    }
    fn to_vec4(&self) -> Vector4<A> {
        *self
    }
    fn to_point2(&self) -> Point2<A> {
        Point2::new(self.x, self.y)
    }
    fn to_point3(&self) -> Point3<A> {
        Point3::new(self.x, self.y, self.z)
    }
}

impl<A: Default + Copy + One<A>> VecPoint<A> for Point2<A> {
    fn to_vec2(&self) -> Vector2<A> {
        Vector2::new(self.x, self.y)
    }
    fn to_vec3(&self) -> Vector3<A> {
        Vector3::new(self.x, self.y, A::default())
    }
    fn to_vec4(&self) -> Vector4<A> {
        Vector4::new(self.x, self.y, A::default(), A::one())
    }
    fn to_point2(&self) -> Point2<A> {
        *self
    }
    fn to_point3(&self) -> Point3<A> {
        Point3::new(self.x, self.y, A::default())
    }
}

impl<A: Default + Copy + One<A>> VecPoint<A> for Point3<A> {
    fn to_vec2(&self) -> Vector2<A> {
        Vector2::new(self.x, self.y)
    }
    fn to_vec3(&self) -> Vector3<A> {
        Vector3::new(self.x, self.y, self.z)
    }
    fn to_vec4(&self) -> Vector4<A> {
        Vector4::new(self.x, self.y, self.z, A::one())
    }
    fn to_point2(&self) -> Point2<A> {
        Point2::new(self.x, self.y)
    }
    fn to_point3(&self) -> Point3<A> {
        *self
    }
}

trait One<A> {
    fn one() -> A;
}

impl One<f32> for f32 {
    fn one() -> f32 {
        1.0
    }
}

impl One<f64> for f64 {
    fn one() -> f64 {
        1.0
    }
}
