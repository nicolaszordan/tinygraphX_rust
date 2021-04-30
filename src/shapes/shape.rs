use crate::shapes::material::Material;
use cgmath::Vector3;

pub trait Shape {
    // returns the distance from orig on ray_dir of the first intersection if any
    fn ray_intersect(&self, ray_orig: Vector3<f32>, ray_dir: Vector3<f32>) -> Option<f32>;

    fn get_material(&self) -> &Material;
}
