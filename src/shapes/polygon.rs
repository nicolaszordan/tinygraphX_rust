use crate::shapes::material::Material;
use crate::shapes::shape::{Ray, RayHit, Shape};

use cgmath::{InnerSpace, Vector3};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Polygon {
    vertex_0: Vector3<f32>,
    vertex_1: Vector3<f32>,
    vertex_2: Vector3<f32>,

    normal: Vector3<f32>,
    v0v1: Vector3<f32>,
    v0v2: Vector3<f32>,

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
            v0v1: vertex_1 - vertex_0,
            v0v2: vertex_2 - vertex_0,
            material,
        }
    }
}

impl Shape for Polygon {
    fn ray_intersect(&self, ray: &Ray) -> Option<RayHit> {
        let pvec = ray.direction.cross(self.v0v2);
        let det = self.v0v1.dot(pvec);
        if det.abs() < 1e-3 {
            return None;
        }

        let inv_det = 1.0 / det;

        let tvec = ray.origin - self.vertex_0;

        let hit_dist = cgmath::dot(tvec, pvec) * inv_det;
        if !(0.0..=1.0).contains(&hit_dist) {
            return None;
        }

        let qvec = tvec.cross(self.v0v1);
        let v = ray.direction.dot(qvec) * inv_det;
        if v < 0.0 || v + hit_dist > 1.0 {
            return None;
        }

        Some(RayHit {
            hit_dist,
            hit_point: ray.origin + ray.direction * hit_dist,
            hit_normal: self.normal,
            material: self.material,
        })
    }
}
