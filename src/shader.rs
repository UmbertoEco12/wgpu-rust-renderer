use crate::camera::CameraBufferHandler;
use crate::camera::ModelMatrixBufferHandler;
use crate::renderer;
use crate::vertex;
use crate::vertex::SimpleVertex;
use crate::vertex::Vertex;

use wgpu::util::DeviceExt;
pub trait Render {
    fn render<'a, 'b: 'a>(&'b self, render_pass: &mut wgpu::RenderPass<'a>);
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ColorUniform {
    pub color: [f32; 4],
}
impl ColorUniform {
    pub fn new(color: [f32; 4]) -> Self {
        Self { color }
    }
}

pub struct ColorBufferHandler {
    pub buffer: wgpu::Buffer,
    pub buffer_bind_group: wgpu::BindGroup,
}

impl ColorBufferHandler {
    pub fn new(device: &wgpu::Device, layout: &wgpu::BindGroupLayout) -> Self {
        // create color uniform placeholder value
        let color_uniform = ColorUniform::new([1.0, 1.0, 1.0, 1.0]);
        // create buffer
        let color_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Color Uniform Buffer"),
            contents: bytemuck::cast_slice(&[color_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let color_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: color_buffer.as_entire_binding(),
            }],
            label: Some("color bind group"),
        });

        Self {
            buffer: color_buffer,
            buffer_bind_group: color_bind_group,
        }
    }

    pub fn change_buffer_color(&mut self, new_color: ColorUniform, queue: &wgpu::Queue) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[new_color]));
    }
}

pub struct SimpleVertexLayout {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
}

impl SimpleVertexLayout {
    pub fn new(device: &wgpu::Device, vertices: Vec<SimpleVertex>, indices: Vec<u16>) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        // --- Index Buffer---
        let num_indices = indices.len() as u32;
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        Self {
            vertex_buffer,
            index_buffer,
            num_indices,
        }
    }
}

pub struct Shader {
    pub render_pipeline: wgpu::RenderPipeline,
    pub color_buffer: ColorBufferHandler,
    pub vertex_layout: SimpleVertexLayout,
    pub camera_buffer: CameraBufferHandler,
    pub model_buffer: ModelMatrixBufferHandler,
}

impl Shader {
    pub fn new(path: &str, renderer: &renderer::Renderer) -> Self {
        let shader: wgpu::ShaderModule = load_shader(path, &renderer.device, Some("Shader"));
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
        // shader only
        let render_pipeline_layout =
            renderer
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[
                        &color_bind_group_layout,
                        &camera_bind_group_layout,
                        &model_bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                });
        let render_pipeline = create_render_pipeline(
            &renderer.device,
            &render_pipeline_layout,
            renderer.config.format,
            Some(crate::texture::Texture::DEPTH_FORMAT),
            &[vertex::SimpleVertex::desc()],
            shader,
            Some("Render pipeline"),
        );

        let vertex_layout = SimpleVertexLayout::new(
            &renderer.device,
            vertex::QUAD_VERTICES.to_vec(),
            vertex::QUAD_INDICES.to_vec(),
        );

        Self {
            render_pipeline,
            color_buffer,
            vertex_layout,
            camera_buffer,
            model_buffer,
        }
    }
}

impl Render for Shader {
    fn render<'a, 'b: 'a>(&'b self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.color_buffer.buffer_bind_group, &[]);
        render_pass.set_bind_group(1, &self.camera_buffer.buffer_bind_group, &[]);
        render_pass.set_bind_group(2, &self.model_buffer.buffer_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_layout.vertex_buffer.slice(..));
        render_pass.set_index_buffer(
            self.vertex_layout.index_buffer.slice(..),
            wgpu::IndexFormat::Uint16,
        );
        render_pass.draw_indexed(0..self.vertex_layout.num_indices, 0, 0..1);
    }
}

pub fn load_shader(path: &str, device: &wgpu::Device, label: Option<&str>) -> wgpu::ShaderModule {
    // Read the shader source from the file
    let shader_source =
        std::fs::read_to_string(path).expect("Failed to read shader source from file");
    // Create a shader module from the source
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: label,
        source: wgpu::ShaderSource::Wgsl(shader_source.into()),
    })
}
pub fn create_render_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    color_format: wgpu::TextureFormat,
    depth_format: Option<wgpu::TextureFormat>,
    vertex_layouts: &[wgpu::VertexBufferLayout],
    shader: wgpu::ShaderModule,
    pipeline_label: Option<&str>,
) -> wgpu::RenderPipeline {
    //let shader = device.create_shader_module(shader);
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: pipeline_label,
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: vertex_layouts,
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: color_format,
                blend: Some(wgpu::BlendState {
                    alpha: wgpu::BlendComponent::REPLACE,
                    color: wgpu::BlendComponent::REPLACE,
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
            polygon_mode: wgpu::PolygonMode::Fill,
            // Requires Features::DEPTH_CLIP_CONTROL
            unclipped_depth: false,
            // Requires Features::CONSERVATIVE_RASTERIZATION
            conservative: false,
        },
        depth_stencil: depth_format.map(|format| wgpu::DepthStencilState {
            format,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    })
}
