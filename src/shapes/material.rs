use cgmath::{Vector3, Vector4};
use serde::{Deserialize, Serialize};

pub type Color = Vector3<f32>;
pub type Albedo = Vector4<f32>;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Material {
    pub albedo: Albedo,
    pub diffuse_color: Color,
    pub specular_exponent: f32,
    pub refractive_index: f32,
}

impl Material {
    pub fn new(
        albedo: Albedo,
        diffuse_color: Color,
        specular_exponent: f32,
        refractive_index: f32,
    ) -> Self {
        Self {
            albedo,
            diffuse_color,
            specular_exponent,
            refractive_index,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_deserialize() {
        let material_json = r#"
        {
            "albedo": [1, 2, 3, 4],
            "diffuse_color": [5, 6, 7],
            "specular_exponent": 8,
            "refractive_index": 9
        }"#;

        let material: Material =
            serde_json::from_str(material_json).expect("failed to deserialize");

        assert_eq!(material.albedo, Albedo::new(1.0, 2.0, 3.0, 4.0));
        assert_eq!(material.diffuse_color, Color::new(5.0, 6.0, 7.0));
        assert_eq!(material.specular_exponent, 8.0);
        assert_eq!(material.refractive_index, 9.0);
    }
}
