use crate::shapes::shape::Shape;
use cgmath::Vector3;

pub struct Sphere {
    center: Vector3<f32>,
    radius: f32,
}

impl Sphere {
    pub fn new(center: Vector3<f32>, radius: f32) -> Self {
        Self { center, radius }
    }
}

impl Shape for Sphere {
    fn ray_intersect(&self, ray_orig: Vector3<f32>, ray_dir: Vector3<f32>, _t0: f32) -> bool {
        let vec_ray_to_center = self.center - ray_orig;
        let tca = cgmath::dot(vec_ray_to_center, ray_dir);
        let d2 = cgmath::dot(vec_ray_to_center, vec_ray_to_center) - tca * tca;
        if d2 > self.radius * self.radius {
            return false;
        }
        let thc = (self.radius * self.radius - d2).sqrt();
        let mut t0 = tca - thc;
        let t1 = tca + thc;
        if t0 < t1 {
            t0 = t1;
        }
        if t0 < 0.0 {
            false
        } else {
            true
        }
    }
}
