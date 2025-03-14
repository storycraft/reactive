use nalgebra::{Isometry3, Matrix4, Scale3, Vector3};

#[derive(Debug, Clone, PartialEq)]
pub struct Transform {
    pub translation: Vector3<f32>,
    pub scale: Vector3<f32>,
    pub shear: Vector3<f32>,
    pub rotation: Vector3<f32>,
}

impl Transform {
    pub const fn new() -> Self {
        Self {
            translation: Vector3::new(0.0, 0.0, 0.0),
            scale: Vector3::new(1.0, 1.0, 1.0),
            shear: Vector3::new(0.0, 0.0, 0.0),
            rotation: Vector3::new(0.0, 0.0, 0.0),
        }
    }

    pub(crate) fn to_matrix(&self) -> Matrix4<f32> {
        Isometry3::new(self.translation, self.rotation).to_matrix()
            * Scale3::from(self.scale).to_homogeneous()
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::new()
    }
}
