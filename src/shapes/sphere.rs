use crate::shapes::material::Material;
use crate::shapes::shape::{Ray, RayHit, Shape};

use cgmath::{InnerSpace, Vector3};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
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
    fn ray_intersect(&self, ray: &Ray) -> Option<RayHit> {
        // calc vector sphere_center -> ray_orig
        let vec_center_to_ray = self.center - ray.origin;

        // project sphere center on ray
        let tca = cgmath::dot(vec_center_to_ray, ray.direction);
        let d2 = cgmath::dot(vec_center_to_ray, vec_center_to_ray) - tca * tca;
        if d2 > self.radius * self.radius {
            return None;
        }
        let thc = (self.radius * self.radius - d2).sqrt();
        let mut t0 = tca - thc;
        let t1 = tca + thc;
        if t0 < 0.0 {
            t0 = t1;
        }
        if t0 < 0.0 {
            None
        } else {
            let hit_point = ray.origin + ray.direction * t0;
            Some(RayHit {
                hit_dist: t0,
                hit_point,
                hit_normal: (hit_point - self.center).normalize(),
                material: self.material,
            })
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::shapes::material::{Albedo, Color};

    #[test]
    fn test_deserialize() {
        let shpere_json = r#"
        {
            "center": [1, 2, 3],
            "radius": 4,
            "material": {
                "albedo": [1, 2, 3, 4],
                "diffuse_color": [5, 6, 7],
                "specular_exponent": 8,
                "refractive_index": 9
            }
        }"#;

        let sphere: Sphere = serde_json::from_str(shpere_json).expect("failed to deserialize");
        assert_eq!(sphere.center, Vector3::new(1.0, 2.0, 3.0));
        assert_eq!(sphere.radius, 4.0);
        assert_eq!(sphere.material.albedo, Albedo::new(1.0, 2.0, 3.0, 4.0));
        assert_eq!(sphere.material.diffuse_color, Color::new(5.0, 6.0, 7.0));
        assert_eq!(sphere.material.specular_exponent, 8.0);
        assert_eq!(sphere.material.refractive_index, 9.0);
    }
}
