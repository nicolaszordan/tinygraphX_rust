use crate::shapes::material::Material;
use crate::shapes::shape::{RayHit, Shape};

use cgmath::{InnerSpace, Vector3};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Polygon {
    vertex_0: Vector3<f32>,
    vertex_1: Vector3<f32>,
    vertex_2: Vector3<f32>,

    normal: Vector3<f32>,

    material: Material,
}

impl Polygon {
    pub fn new(
        vertex_0: Vector3<f32>,
        vertex_1: Vector3<f32>,
        vertex_2: Vector3<f32>,
        material: Material,
    ) -> Self {
        Self {
            vertex_0,
            vertex_1,
            vertex_2,
            normal: (vertex_0 - vertex_1).cross(vertex_2 - vertex_1),
            material,
        }
    }
}

impl Shape for Polygon {
    fn ray_intersect(&self, ray_orig: Vector3<f32>, ray_dir: Vector3<f32>) -> Option<RayHit> {
        let v0v1 = self.vertex_1 - self.vertex_0;
        let v0v2 = self.vertex_2 - self.vertex_0;
        let pvec = ray_dir.cross(v0v2);
        let det = cgmath::dot(v0v1, pvec);
        if det.abs() < 1e-3 {
            return None;
        }

        let inv_det = 1.0 / det;

        let tvec = ray_orig - self.vertex_0;

        let hit_dist = cgmath::dot(tvec, pvec) * inv_det;
        if hit_dist < 0.0 || hit_dist > 1.0 {
            return None;
        }

        let qvec = tvec.cross(v0v1);
        let v = ray_dir.dot(qvec) * inv_det;
        if v < 0.0 || v + hit_dist > 1.0 {
            return None;
        }

        Some(RayHit {
            hit_dist,
            hit_point: ray_orig + ray_dir * hit_dist,
            hit_normal: self.normal,
            material: self.material,
        })
    }
}
