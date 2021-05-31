use crate::shapes::material::Material;
use cgmath::Vector3;

pub struct Ray {
    pub origin: Vector3<f32>,
    pub direction: Vector3<f32>,
    pub inv_direction: Vector3<f32>,
}

pub struct RayHit {
    pub hit_dist: f32,
    pub hit_point: Vector3<f32>,
    pub hit_normal: Vector3<f32>,
    pub material: Material,
}

pub trait Shape {
    // returns the distance from orig on ray_dir of the first intersection if any
    fn ray_intersect(&self, ray: &Ray) -> Option<RayHit>;
}

impl Ray {
    pub fn new(origin: Vector3<f32>, direction: Vector3<f32>) -> Ray {
        Ray {
            origin,
            direction,
            inv_direction: 1.0 / direction,
        }
    }
}
