pub trait Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static>;
}
// used for simple geometry (quads, trinagles)
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SimpleVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
}

impl Vertex for SimpleVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            // vertex byte size
            array_stride: mem::size_of::<SimpleVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // vertex position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // vertex tex coords
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

// Define a quad using SimpleVertex
pub const QUAD_VERTICES: [SimpleVertex; 4] = [
    SimpleVertex {
        position: [-0.5, -0.5, 0.0],
        tex_coords: [0.0, 0.0],
    },
    SimpleVertex {
        position: [0.5, -0.5, 0.0],
        tex_coords: [1.0, 0.0],
    },
    SimpleVertex {
        position: [0.5, 0.5, 0.0],
        tex_coords: [1.0, 1.0],
    },
    SimpleVertex {
        position: [-0.5, 0.5, 0.0],
        tex_coords: [0.0, 1.0],
    },
];

// Define the index array to form two triangles (forming a quad)
pub const QUAD_INDICES: [u16; 6] = [0, 1, 2, 0, 2, 3];
