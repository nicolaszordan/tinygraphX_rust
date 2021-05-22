use crate::shapes::material::Material;
use crate::shapes::shape::{RayHit, Shape};

use cgmath::Vector3;

pub struct Plane {
    pub point: Vector3<f32>,
    pub normal: Vector3<f32>,
    pub material: Material,
}

impl Plane {
    pub fn new(point: Vector3<f32>, normal: Vector3<f32>, material: Material) -> Self {
        Self {
            point,
            normal,
            material,
        }
    }
}

impl Shape for Plane {
    fn ray_intersect(&self, ray_orig: Vector3<f32>, ray_dir: Vector3<f32>) -> Option<RayHit> {
        let denom = cgmath::dot(self.normal, ray_dir);
        if denom.abs() < 1e-3 {
            // ray is considered parallel to the plane
            None
        } else {
            let point_to_ray = self.point - ray_orig;
            let hit_dist = cgmath::dot(point_to_ray, self.normal) / denom;
            let hit_point = ray_orig + ray_dir * hit_dist;
            if hit_dist >= 0.0 {
                Some(RayHit {
                    hit_dist,
                    hit_point,
                    hit_normal: self.normal,
                    material: self.material,
                })
            } else {
                // ray goes away for the plane
                None
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::shapes::material::{Albedo, Color};
    use num::Zero;

    #[test]
    fn test_plane_ray_intersect() {
        let material = Material::new(Albedo::zero(), Color::zero(), 0.0, 0.0);

        let plane = Plane::new(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
            material,
        );
        let ray_dir = Vector3::new(0.0, 1.0, 0.0);
        let ray_orig = Vector3::new(0.0, -1.0, 0.0);

        assert!(plane.ray_intersect(ray_orig, ray_dir).is_some());

        let ray_dir = Vector3::new(0.0, 0.5, 0.0);
        let ray_orig = Vector3::new(0.0, -1.0, 0.0);

        assert!(plane.ray_intersect(ray_orig, ray_dir).is_some());

        let ray_dir = Vector3::new(0.0, 1.0, 1.0);
        let ray_orig = Vector3::new(0.0, -1.0, 0.0);

        assert!(plane.ray_intersect(ray_orig, ray_dir).is_some());

        let ray_dir = Vector3::new(-1.0, -1.0, -1.0);
        let ray_orig = Vector3::new(0.0, 1.0, 0.0);

        assert!(plane.ray_intersect(ray_orig, ray_dir).is_some());

        let ray_dir = Vector3::new(0.0, -1.0, 0.0);
        let ray_orig = Vector3::new(0.0, 1.0, 0.0);

        assert!(plane.ray_intersect(ray_orig, ray_dir).is_some());
    }

    #[test]
    fn test_plane_parralel_ray() {
        let material = Material::new(Albedo::zero(), Color::zero(), 0.0, 0.0);

        let plane = Plane::new(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
            material,
        );
        let ray_dir = Vector3::new(1.0, 0.0, 0.0);
        let ray_orig = Vector3::new(0.0, 1.0, 0.0);

        assert!(plane.ray_intersect(ray_orig, ray_dir).is_none());
    }

    #[test]
    fn test_plane_ray_miss() {
        let material = Material::new(Albedo::zero(), Color::zero(), 0.0, 0.0);

        let plane = Plane::new(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
            material,
        );
        let ray_dir = Vector3::new(0.0, 1.0, 0.0);
        let ray_orig = Vector3::new(0.0, 1.0, 0.0);

        assert!(plane.ray_intersect(ray_orig, ray_dir).is_none());
    }
}
