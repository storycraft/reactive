use nalgebra::Matrix4;

use crate::{
    ElementId,
    tree::{TreeVisitorMut, visitor},
};

use super::split::{Elements, Relations};

pub fn update_matrix(id: ElementId, elements: &mut Elements, relations: Relations) {
    pub struct UpdateMatrix {
        matrix: Matrix4<f32>,
        inverse_matrix: Matrix4<f32>,
    }

    impl TreeVisitorMut for UpdateMatrix {
        fn visit_mut(&mut self, id: ElementId, elements: &mut Elements, relations: Relations) {
            let mut element = elements[id].as_mut();
            let matrix = self.matrix * element.transform.to_matrix();
            let inverse_matrix = self.inverse_matrix * element.transform.to_inverse_matrix();

            let node = element.as_mut().project().node;
            if node.matrix_outdated {
                node.matrix_outdated = false;
            }
            node.matrix = matrix;
            node.inverse_matrix = inverse_matrix;

            visitor::visit_mut(
                &mut Self {
                    matrix,
                    inverse_matrix,
                },
                id,
                elements,
                relations,
            );
        }
    }

    let mut update = if let Some(parent) = relations.parent(id) {
        let node = &elements[parent].node;
        UpdateMatrix {
            matrix: node.matrix,
            inverse_matrix: node.inverse_matrix,
        }
    } else {
        UpdateMatrix {
            matrix: Matrix4::identity(),
            inverse_matrix: Matrix4::identity(),
        }
    };

    update.visit_mut(id, elements, relations);
}
