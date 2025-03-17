use nalgebra::{Matrix4, Vector3};

use crate::dimension::Dimension;

#[derive(Debug, Clone, PartialEq)]
pub struct Transform {
    pub translation: Vector3<f32>,
    pub origin: Vector3<Dimension>,
    pub scale: Vector3<f32>,
    pub shear: Vector3<f32>,
    pub rotation: Vector3<f32>,
}

impl Transform {
    pub const fn new() -> Self {
        Self {
            translation: Vector3::new(0.0, 0.0, 0.0),
            origin: Vector3::new(
                Dimension::Absolute(0.0),
                Dimension::Absolute(0.0),
                Dimension::Absolute(0.0),
            ),
            scale: Vector3::new(1.0, 1.0, 1.0),
            shear: Vector3::new(0.0, 0.0, 0.0),
            rotation: Vector3::new(0.0, 0.0, 0.0),
        }
    }

    pub(crate) fn to_matrix(&self) -> Matrix4<f32> {
        Matrix4::new_rotation(self.rotation)
            * Matrix4::new_translation(&self.translation)
            * Matrix4::new_nonuniform_scaling(&self.scale)
    }

    pub(crate) fn to_inverse_matrix(&self) -> Matrix4<f32> {
        Matrix4::new_rotation(self.rotation)
            * Matrix4::new_translation(&self.translation)
            * Matrix4::new_nonuniform_scaling(&Vector3::new(
                1.0 / self.scale.x,
                1.0 / self.scale.y,
                1.0 / self.scale.z,
            ))
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::new()
    }
}
