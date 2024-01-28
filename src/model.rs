use std::collections::HashMap;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, PartialEq, Default)]
pub struct ModelVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 3],
    pub tangent: [f32; 4],
    pub bone_ids: [f32; 4],
    pub bone_weights: [f32; 4],
}

impl crate::vertex::Vertex for ModelVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<ModelVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // tex_coords
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // normal
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // tangent
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // bone_ids
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // bone_weights
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}
#[derive(Debug, Default)]
pub struct Model {
    pub meshes: Vec<Mesh>,
}
#[derive(Debug, Default)]
pub struct Mesh {
    pub name: String,
    pub vertices: Vec<ModelVertex>,
    pub indices: Vec<u32>,
    pub skeleton: Option<Skeleton>,
}
#[derive(Debug, PartialEq, Default, Clone)]
pub struct Bone {
    pub id: u32,
    pub name: String,
    pub parent_id: Option<usize>,
    pub inverse_bind_matrix: [[f32; 4]; 4],
    pub index: usize,
}

#[derive(Debug, Default)]
pub struct Skeleton {
    pub name: String,
    pub bones: HashMap<usize, Bone>,
    pub bones_ordered: Vec<Bone>,
}
#[derive(Debug, Default, Clone)]
pub struct KeyTranslation {
    pub timestamp: f32,
    pub translation: [f32; 3],
}
#[derive(Debug, Default, Clone)]
pub struct KeyRotation {
    pub timestamp: f32,
    pub rotation: [f32; 4],
}
#[derive(Debug, Default, Clone)]
pub struct KeyScale {
    pub timestamp: f32,
    pub scale: [f32; 3],
}
#[derive(Debug, Default, Clone)]
pub struct AnimatedBone {
    pub bone_id: u32,
    pub bone_name: String,
    pub parent_index: Option<usize>,
    pub translation_keys: Vec<KeyTranslation>,
    pub rotation_keys: Vec<KeyRotation>,
    pub scale_keys: Vec<KeyScale>,
}
#[derive(Debug, Default)]
pub struct Animation {
    pub name: String,
    pub bone_keyframes: HashMap<usize, AnimatedBone>,
    pub bone_keyframes_name: HashMap<String, AnimatedBone>,
}

pub struct MeshLayout {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
}
impl MeshLayout {
    pub fn new(device: &wgpu::Device, mesh: &Mesh) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&mesh.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        // --- Index Buffer---
        let num_indices = mesh.indices.len() as u32;
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&mesh.indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        Self {
            vertex_buffer,
            index_buffer,
            num_indices,
        }
    }
}
pub const MAX_BONES: usize = 100;
use bytemuck::{Pod, Zeroable};
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BoneTransformsUniform {
    pub transforms: [[[f32; 4]; 4]; MAX_BONES],
}

impl BoneTransformsUniform {
    pub fn new() -> Self {
        let mut transforms: [[[f32; 4]; 4]; MAX_BONES] = [[[0.0; 4]; 4]; MAX_BONES];
        // fill transforms with identity matrix
        for i in 0..MAX_BONES {
            transforms[i] = cgmath::Matrix4::identity().into();
        }
        Self { transforms }
    }
}
unsafe impl Pod for BoneTransformsUniform {}
unsafe impl Zeroable for BoneTransformsUniform {}
pub struct BoneBufferHandler {
    pub buffer: wgpu::Buffer,
    pub buffer_bind_group: wgpu::BindGroup,
}
use cgmath::SquareMatrix;
impl BoneBufferHandler {
    pub fn new(device: &wgpu::Device, layout: &wgpu::BindGroupLayout) -> Self {
        // create color uniform placeholder value
        let bone_uniform = BoneTransformsUniform::new();
        // create buffer
        let bones_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Bones Buffer"),
            contents: bytemuck::cast_slice(&[bone_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let color_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: bones_buffer.as_entire_binding(),
            }],
            label: Some("bones bind group"),
        });

        Self {
            buffer: bones_buffer,
            buffer_bind_group: color_bind_group,
        }
    }

    pub fn change_transforms(
        &mut self,
        new_transforms: BoneTransformsUniform,
        queue: &wgpu::Queue,
    ) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[new_transforms]));
    }
}
