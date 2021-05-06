use cgmath::Vector3;

pub type Color = Vector3<f32>;
pub type Albedo = Vector3<f32>;

#[derive(Debug, Copy, Clone)]
pub struct Material {
    pub albedo: Albedo,
    pub diffuse_color: Color,
    pub specular_exponent: f32,
}

impl Material {
    pub fn new(albedo: Albedo, diffuse_color: Color, specular_exponent: f32) -> Self {
        Self {
            albedo,
            diffuse_color,
            specular_exponent,
        }
    }
}
