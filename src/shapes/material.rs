use cgmath::Vector3;

pub type Color = Vector3<f32>;

#[derive(Debug, Copy, Clone)]
pub struct Material {
    pub diffuse_color: Color,
}

impl Material {
    pub fn new(diffuse_color: Color) -> Self {
        Self { diffuse_color }
    }
}
