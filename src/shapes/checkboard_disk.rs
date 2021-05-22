use crate::shapes::material::Material;
use crate::shapes::shape::{RayHit, Shape};

use cgmath::Vector3;

// squared chekboard
pub struct CheckBoardDisk {
    pub center: Vector3<f32>,
    pub normal: Vector3<f32>,
    pub radius: f32,
    pub dist_between_mats: f32,
    pub material1: Material,
    pub material2: Material,
}

impl CheckBoardDisk {
    pub fn new(
        center: Vector3<f32>,
        normal: Vector3<f32>,
        radius: f32,
        dist_between_mats: f32,
        material1: Material,
        material2: Material,
    ) -> Self {
        Self {
            center,
            normal,
            radius,
            dist_between_mats,
            material1,
            material2,
        }
    }
}

impl Shape for CheckBoardDisk {
    fn ray_intersect(&self, ray_orig: Vector3<f32>, ray_dir: Vector3<f32>) -> Option<RayHit> {
        let denom = cgmath::dot(self.normal, ray_dir);
        if denom.abs() < 1e-3 {
            // ray is considered parallel to the disk
            None
        } else {
            let point_to_ray = self.center - ray_orig;
            let hit_dist = cgmath::dot(point_to_ray, self.normal) / denom;
            if hit_dist < 0.0 {
                return None;
            }
            let hit_point = ray_orig + ray_dir * hit_dist;
            let hit_to_center = hit_point - self.center;
            let dist_hit_to_center = cgmath::dot(hit_to_center, hit_to_center).sqrt();
            if dist_hit_to_center <= self.radius {
                Some(RayHit {
                    hit_dist,
                    hit_point,
                    hit_normal: self.normal,
                    material: if dist_hit_to_center % self.dist_between_mats
                        > self.dist_between_mats / 2.0
                    {
                        self.material1
                    } else {
                        self.material2
                    },
                })
            } else {
                None
            }
        }
    }
}
