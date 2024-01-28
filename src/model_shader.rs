use crate::camera::CameraBufferHandler;
use crate::camera::ModelMatrixBufferHandler;
use crate::light::LightBufferHandler;
use crate::model::BoneBufferHandler;
use crate::model::MeshLayout;
use crate::renderer;
use crate::shader::{self, ColorBufferHandler, Render};
use crate::vertex::Vertex;
pub struct ModelShader {
    pub render_pipeline: wgpu::RenderPipeline,
    pub color_buffer: ColorBufferHandler,
    pub vertex_layouts: Vec<MeshLayout>,
    pub camera_buffer: CameraBufferHandler,
    pub bone_transform_buffer: BoneBufferHandler,
    pub light_buffer: LightBufferHandler,
    pub model_buffer: ModelMatrixBufferHandler,
}
impl ModelShader {
    pub fn new(path: &str, renderer: &renderer::Renderer, model: &crate::model::Model) -> Self {
        let shader: wgpu::ShaderModule =
            shader::load_shader(path, &renderer.device, Some("Shader"));
        // color
        let color_bind_group_layout =
            renderer
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                    label: Some("Color bind group layout"),
                });
        // render only
        let color_buffer = ColorBufferHandler::new(&renderer.device, &color_bind_group_layout);

        // camera
        let camera_bind_group_layout =
            renderer
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                    label: Some("camera_bind_group_layout"),
                });
        let camera_buffer =
            crate::camera::CameraBufferHandler::new(&renderer.device, &camera_bind_group_layout);

        // model matrix buffer
        let model_bind_group_layout =
            renderer
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                    label: Some("model_bind_group_layout"),
                });
        let model_buffer =
            ModelMatrixBufferHandler::new(&renderer.device, &model_bind_group_layout);

        // bone buffer
        let bones_bind_group_layout =
            renderer
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                    label: Some("bones_bind_group_layout"),
                });
        let bones_buffer = BoneBufferHandler::new(&renderer.device, &bones_bind_group_layout);

        // light buffer
        let light_bind_group_layout =
            renderer
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                    label: Some("Light bind group layout"),
                });
        let light_buffer = LightBufferHandler::new(&renderer.device, &light_bind_group_layout);
        // shader only
        let render_pipeline_layout =
            renderer
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[
                        //&color_bind_group_layout,
                        &light_bind_group_layout,
                        &camera_bind_group_layout,
                        &model_bind_group_layout,
                        &bones_bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                });
        let render_pipeline = shader::create_render_pipeline(
            &renderer.device,
            &render_pipeline_layout,
            renderer.config.format,
            Some(crate::texture::Texture::DEPTH_FORMAT),
            &[crate::model::ModelVertex::desc()],
            shader,
            Some("Render pipeline"),
        );
        let mut vertices = Vec::new();
        for mesh in &model.meshes {
            vertices.push(MeshLayout::new(&renderer.device, &mesh));
        }
        Self {
            render_pipeline,
            color_buffer,
            vertex_layouts: vertices,
            camera_buffer,
            model_buffer,
            bone_transform_buffer: bones_buffer,
            light_buffer,
        }
    }
}
impl Render for ModelShader {
    fn render<'a, 'b: 'a>(&'b self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.light_buffer.buffer_bind_group, &[]);
        render_pass.set_bind_group(1, &self.camera_buffer.buffer_bind_group, &[]);
        render_pass.set_bind_group(2, &self.model_buffer.buffer_bind_group, &[]);
        render_pass.set_bind_group(3, &self.bone_transform_buffer.buffer_bind_group, &[]);
        //render_pass.set_bind_group(0, &self.color_buffer.buffer_bind_group, &[]);
        for vertex_layout in &self.vertex_layouts {
            render_pass.set_vertex_buffer(0, vertex_layout.vertex_buffer.slice(..));
            render_pass.set_index_buffer(
                vertex_layout.index_buffer.slice(..),
                wgpu::IndexFormat::Uint32,
            );
            render_pass.draw_indexed(0..vertex_layout.num_indices, 0, 0..1);
        }
    }
}
