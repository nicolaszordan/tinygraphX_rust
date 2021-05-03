use cgmath::Vector3;

pub struct Light {
    pub position: Vector3<f32>,
    pub intensity: f32,
}

impl Light {
    pub fn new(position: Vector3<f32>, intensity: f32) -> Self {
        Self {
            position,
            intensity,
        }
    }
}
