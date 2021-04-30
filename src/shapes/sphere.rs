use crate::shapes::material::Material;
use crate::shapes::shape::Shape;

use cgmath::Vector3;

pub struct Sphere {
    pub center: Vector3<f32>,
    pub radius: f32,
    pub material: Material,
}

impl Sphere {
    pub fn new(center: Vector3<f32>, radius: f32, material: Material) -> Self {
        Self {
            center,
            radius,
            material,
        }
    }
}

impl Shape for Sphere {
    fn ray_intersect(&self, ray_orig: Vector3<f32>, ray_dir: Vector3<f32>) -> Option<f32> {
        // calc vector sphere_center -> ray_orig
        let vec_center_to_ray = self.center - ray_orig;

        // project sphere center on ray
        let tca = cgmath::dot(vec_center_to_ray, ray_dir);
        let d2 = cgmath::dot(vec_center_to_ray, vec_center_to_ray) - tca * tca;
        if d2 > self.radius * self.radius {
            return None;
        }
        let thc = (self.radius * self.radius - d2).sqrt();
        let mut t0 = tca - thc;
        let t1 = tca + thc;
        if t0 < t1 {
            t0 = t1;
        }
        if t0 < 0.0 {
            None
        } else {
            Some(t0)
        }
    }

    fn get_material(&self) -> &Material {
        &self.material
    }
}
