use crate::shapes::material::Material;
use crate::shapes::shape::{Ray, RayHit, Shape};

use cgmath::Vector3;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Disk {
    pub center: Vector3<f32>,
    pub normal: Vector3<f32>,
    pub radius: f32,
    pub radius_squared: f32,
    pub material: Material,
}

impl Disk {
    pub fn new(
        center: Vector3<f32>,
        normal: Vector3<f32>,
        radius: f32,
        material: Material,
    ) -> Self {
        Self {
            center,
            normal,
            radius,
            radius_squared: radius * radius,
            material,
        }
    }
}

impl Shape for Disk {
    fn ray_intersect(&self, ray: &Ray) -> Option<RayHit> {
        let denom = cgmath::dot(self.normal, ray.direction);
        if denom.abs() < 1e-3 {
            // ray is considered parallel to the disk
            None
        } else {
            let point_to_ray = self.center - ray.origin;
            let hit_dist = cgmath::dot(point_to_ray, self.normal) / denom;
            if hit_dist < 0.0 {
                return None;
            }
            let hit_point = ray.origin + ray.direction * hit_dist;
            let hit_to_center = hit_point - self.center;
            let d2 = cgmath::dot(hit_to_center, hit_to_center);
            if d2 <= self.radius_squared {
                Some(RayHit {
                    hit_dist,
                    hit_point,
                    hit_normal: self.normal,
                    material: self.material,
                })
            } else {
                None
            }
        }
    }
}
