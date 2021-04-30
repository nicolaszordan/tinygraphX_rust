use cgmath::Vector3;

pub trait Shape {
    fn ray_intersect(&self, ray_orig: Vector3<f32>, ray_dir: Vector3<f32>, t0: f32) -> bool;
}
