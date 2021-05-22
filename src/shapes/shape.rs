use crate::shapes::material::Material;
use cgmath::Vector3;

pub struct RayHit {
    pub hit_dist: f32,
    pub hit_point: Vector3<f32>,
    pub hit_normal: Vector3<f32>,
    pub material: Material,
}

pub trait Shape {
    // returns the distance from orig on ray_dir of the first intersection if any
    fn ray_intersect(&self, ray_orig: Vector3<f32>, ray_dir: Vector3<f32>) -> Option<RayHit>;
}
