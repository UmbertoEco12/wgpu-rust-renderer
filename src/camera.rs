use crate::transform::Transform;
use cgmath::SquareMatrix;
use cgmath::{Matrix4, PerspectiveFov, Quaternion, Rad, Vector3};
// --- Camera ---
#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);
#[derive(Debug, Clone, Copy)]
pub struct Camera {
    pub transform: Transform,
    pub fov: f32,
    pub aspect_ratio: f32,
    pub near: f32,
    pub far: f32,
}

const DEFAULT_LEFT: f32 = -1.0;
const DEFAULT_RIGHT: f32 = 1.0;
const DEFAULT_BOTTOM: f32 = -1.0;
const DEFAULT_TOP: f32 = 1.0;
const DEFAULT_NEAR: f32 = -1.0;
const DEFAULT_FAR: f32 = 1.0;

impl Camera {
    pub fn new(transform: Transform, fov: f32, aspect_ratio: f32, near: f32, far: f32) -> Self {
        Camera {
            transform,
            fov,
            aspect_ratio,
            near,
            far,
        }
    }
    pub fn default(aspect_ratio: f32) -> Self {
        let mut transform = Transform::identity();
        transform.translate(cgmath::Vector3::from([0.0, 1.0, 5.5]));
        Self {
            transform: transform,
            fov: 45.0,
            near: 0.1,
            far: 100.0,
            aspect_ratio: aspect_ratio,
        }
    }

    pub fn view_matrix(&self) -> [[f32; 4]; 4] {
        let translation_matrix = Matrix4::from_translation(Vector3::new(
            -self.transform.position[0],
            -self.transform.position[1],
            -self.transform.position[2],
        ));

        let rotation_matrix = Matrix4::from(Quaternion::new(
            self.transform.rotation[3],
            self.transform.rotation[0],
            self.transform.rotation[1],
            self.transform.rotation[2],
        ));

        let transformation_matrix = translation_matrix * rotation_matrix;
        let array: [[f32; 4]; 4] = transformation_matrix.into();
        array
    }

    pub fn projection_matrix(&self) -> [[f32; 4]; 4] {
        let perspective = PerspectiveFov {
            fovy: Rad(self.fov),
            aspect: self.aspect_ratio,
            near: self.near,
            far: self.far,
        };

        let projection_matrix: Matrix4<f32> = perspective.into();
        let array: [[f32; 4]; 4] = projection_matrix.into();
        array
    }

    fn build_view_projection_matrix(&self) -> Matrix4<f32> {
        let view_matrix = cgmath::Matrix4::from(self.view_matrix());
        let perspective = cgmath::perspective(
            cgmath::Deg(self.fov),
            self.aspect_ratio,
            self.near,
            self.far,
        );

        let ortho = cgmath::ortho(
            DEFAULT_LEFT,
            DEFAULT_RIGHT,
            DEFAULT_BOTTOM,
            DEFAULT_TOP,
            DEFAULT_NEAR,
            DEFAULT_FAR,
        );

        let projection_matrix: Matrix4<f32> = perspective;

        OPENGL_TO_WGPU_MATRIX * projection_matrix * view_matrix
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub matrix: [[f32; 4]; 4],
    pub proj_matrix: [[f32; 4]; 4],
}
impl CameraUniform {
    pub fn new() -> Self {
        Self {
            matrix: Transform::identity().matrix().into(),
            proj_matrix: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        //println!("self matrix {:#?}", self.matrix);
        self.matrix = camera.transform.matrix().into();
        //println!("after matrix {:#?}", self.matrix);
        self.proj_matrix = (camera.build_view_projection_matrix()).into();
        //println!("proj matrix {:#?}", self.proj_matrix);
    }
}
use wgpu::util::DeviceExt;
pub struct CameraBufferHandler {
    pub buffer: wgpu::Buffer,
    pub buffer_bind_group: wgpu::BindGroup,
    pub camera_uniform: CameraUniform,
}
impl CameraBufferHandler {
    pub fn new(device: &wgpu::Device, layout: &wgpu::BindGroupLayout) -> Self {
        // create color uniform placeholder value
        let camera_uniform = CameraUniform::new();
        // create buffer
        let color_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Uniform Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let color_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: color_buffer.as_entire_binding(),
            }],
            label: Some("Camera bind group"),
        });

        Self {
            buffer: color_buffer,
            buffer_bind_group: color_bind_group,
            camera_uniform,
        }
    }

    pub fn update_camera(&mut self, camera: &Camera, queue: &wgpu::Queue) {
        self.camera_uniform.update_view_proj(camera);
        queue.write_buffer(
            &self.buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelMatrixUniform {
    pub matrix: [[f32; 4]; 4],
}

// model matrix uniform
pub struct ModelMatrixBufferHandler {
    pub buffer: wgpu::Buffer,
    pub buffer_bind_group: wgpu::BindGroup,
    pub matrix_uniform: ModelMatrixUniform,
}

impl ModelMatrixBufferHandler {
    pub fn new(device: &wgpu::Device, layout: &wgpu::BindGroupLayout) -> Self {
        // create color uniform placeholder value
        let matrix: [[f32; 4]; 4] = cgmath::Matrix4::identity().into();
        let matrix_uniform = ModelMatrixUniform { matrix };
        // create buffer
        let color_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Model Matrix Uniform Buffer"),
            contents: bytemuck::cast_slice(&[matrix_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let color_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: color_buffer.as_entire_binding(),
            }],
            label: Some("Model Matrix bind group"),
        });
        Self {
            buffer: color_buffer,
            buffer_bind_group: color_bind_group,
            matrix_uniform,
        }
    }

    pub fn update_matrix(&mut self, matrix: ModelMatrixUniform, queue: &wgpu::Queue) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[matrix]));
    }
}
