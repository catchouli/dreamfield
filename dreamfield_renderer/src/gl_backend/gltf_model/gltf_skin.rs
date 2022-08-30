use std::sync::{Arc, Mutex};
use cgmath::{SquareMatrix, Matrix4};
use byteorder::{LittleEndian, ReadBytesExt};

use super::gltf_transform::{GltfTransformHierarchy, GltfTransform};

/// A gltf skin
pub struct GltfSkin {
    joints: Vec<GltfJoint>
}

/// One joint in a skin
pub struct GltfJoint {
    joint_transform: Arc<Mutex<GltfTransform>>,
    inverse_bind_matrix: Matrix4<f32>
}

impl GltfSkin {
    /// Load a skin
    pub fn load(skin: &gltf::Skin, buffer_data: &[gltf::buffer::Data], hierarchy: &GltfTransformHierarchy) -> GltfSkin
    {
        const F32_SIZE: usize = std::mem::size_of::<f32>();

        // Get joint indices
        let joint_transforms = skin.joints().map(|joint| {
            hierarchy
                .node_by_index(joint.index())
                .as_ref()
                .expect("Joint node was not found")
                .clone()
        });

        // Get inverse bind matrices
        let joint_count = skin.joints().count();
        let inverse_bind_matrices = skin.inverse_bind_matrices().map(|accessor| {
            // Get view and buffer data
            let view = accessor.view().unwrap();
            let buffer_data = &buffer_data[view.buffer().index()];

            // Read matrices
            let expected_length = 16 * joint_count * F32_SIZE;
            assert!(accessor.data_type().size() == F32_SIZE);
            assert!(view.length() == expected_length);

            let start = view.offset();
            let end = start + expected_length;
            let mut slice = buffer_data.get(start..end).unwrap();

            let matrices = (0..joint_count).map(|_| {
                let m00 = slice.read_f32::<LittleEndian>().unwrap();
                let m01 = slice.read_f32::<LittleEndian>().unwrap();
                let m02 = slice.read_f32::<LittleEndian>().unwrap();
                let m03 = slice.read_f32::<LittleEndian>().unwrap();
                let m10 = slice.read_f32::<LittleEndian>().unwrap();
                let m11 = slice.read_f32::<LittleEndian>().unwrap();
                let m12 = slice.read_f32::<LittleEndian>().unwrap();
                let m13 = slice.read_f32::<LittleEndian>().unwrap();
                let m20 = slice.read_f32::<LittleEndian>().unwrap();
                let m21 = slice.read_f32::<LittleEndian>().unwrap();
                let m22 = slice.read_f32::<LittleEndian>().unwrap();
                let m23 = slice.read_f32::<LittleEndian>().unwrap();
                let m30 = slice.read_f32::<LittleEndian>().unwrap();
                let m31 = slice.read_f32::<LittleEndian>().unwrap();
                let m32 = slice.read_f32::<LittleEndian>().unwrap();
                let m33 = slice.read_f32::<LittleEndian>().unwrap();
                Matrix4::new(m00, m01, m02, m03, m10, m11, m12, m13, m20, m21, m22, m23, m30, m31, m32, m33)
            }).collect();

            // Check data is fully consumed
            assert!(slice.len() == 0);

            matrices
        })
        .unwrap_or(vec![SquareMatrix::identity(); joint_count]);

        // Get joints
        let joints = joint_transforms.zip(inverse_bind_matrices)
            .map(|(joint_transform, inverse_bind_matrix)| {
                GltfJoint {
                    joint_transform,
                    inverse_bind_matrix
                }
            })
            .collect();

        GltfSkin {
            joints
        }
    }

    /// Get the joints
    pub fn joints(&self) -> &[GltfJoint] {
        &self.joints
    }
}

impl GltfJoint {
    /// Get the joint transform
    pub fn transform(&self) -> &Arc<Mutex<GltfTransform>> {
        &self.joint_transform
    }

    /// Get the inverse bind matrix
    pub fn inverse_bind_matrix(&self) -> &Matrix4<f32> {
        &self.inverse_bind_matrix
    }
}
