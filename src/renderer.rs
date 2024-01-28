use crate::{
    shader::{self, Render, Shader},
    window::WindowSize,
};
use std::rc::Rc;
use std::{borrow::Borrow, cell::RefCell};

pub struct Renderer {
    pub size: WindowSize,
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub clear_color: wgpu::Color,
    can_render: bool,
    pub render_objects: Vec<Rc<RefCell<dyn Render>>>,
    depth_texture: crate::texture::Texture,
}

impl Renderer {
    pub async fn new<W>(window: &W, size: WindowSize) -> Self
    where
        W: raw_window_handle::HasRawWindowHandle + raw_window_handle::HasRawDisplayHandle,
    {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        let surface = unsafe { instance.create_surface(window) }.unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        // device and queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None, // Trace path
            )
            .await
            .unwrap();
        // config surface
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .filter(|f| f.is_srgb())
            .next()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Immediate, //surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);
        let clear_color = wgpu::Color::BLACK;
        let depth_texture =
            crate::texture::Texture::create_depth_texture(&device, &config, "depth_texture");
        Self {
            size,
            surface,
            device,
            queue,
            config,
            clear_color,
            can_render: true,
            render_objects: Vec::new(),
            depth_texture,
        }
    }
    pub fn resize(&mut self, new_size: WindowSize) {
        if new_size.width > 0 && new_size.height > 0 {
            // enable render
            self.can_render = true;
            // set size
            self.size = new_size;
            // recreate surface configuration
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            // reacreate depth texture
            self.depth_texture = crate::texture::Texture::create_depth_texture(
                &self.device,
                &self.config,
                "depth_texture",
            );
            // // camera
            // // Update camera aspect ratio
            // self.camera.aspect = new_size.width as f32 / new_size.height as f32;
            // // Update and recreate camera uniform
            // self.camera_uniform.update_view_proj(&self.camera);
            // self.queue.write_buffer(
            //     &self.camera_buffer,
            //     0,
            //     bytemuck::cast_slice(&[self.camera_uniform]),
            // );
        } else {
            self.can_render = false;
        }
    }
    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        if !self.can_render {
            return Ok(());
        }
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        {
            // get all the shaders ref in a vector
            // to fix doesn t live long enough
            // (more than render_pass)
            let mut ref_vec = Vec::new();
            for shader in &self.render_objects {
                //let val: &RefCell<shader::Shader> = Rc::borrow(shader);
                //let val = val.borrow_mut();
                let val = (*shader).borrow_mut();
                ref_vec.push(val);
            }
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            // render pipeline work
            for shader in &ref_vec {
                shader.render(&mut render_pass);
            }
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn add_shader(&mut self, shader: Rc<RefCell<dyn Render>>) {
        self.render_objects.push(shader);
    }
}
