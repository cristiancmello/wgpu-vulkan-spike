use std::sync::{Arc, Mutex};
use std::thread;

use wgpu::util::DeviceExt;
use winit::window::Window;

use crate::draw_command::DrawCommand;
use crate::renderer;
use crate::transform::TransformBuffer;
use crate::primitive::PrimitiveBuffer;

pub struct State {
    instance: wgpu::Instance,
    window: Arc<Window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    size: winit::dpi::PhysicalSize<u32>,
    surface: wgpu::Surface<'static>,
    surface_format: wgpu::TextureFormat,
    command_buffer: Arc<Mutex<Vec<DrawCommand>>>,
    pipeline: wgpu::RenderPipeline,
    transforms: TransformBuffer,
    primitives: PrimitiveBuffer,
    clear_color: wgpu::Color,
}

impl State {
    pub async fn new(
        display: winit::event_loop::OwnedDisplayHandle,
        window: Arc<Window>,
    ) -> State {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_with_display_handle(
            Box::new(display),
        ));
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .unwrap();

        let size = window.inner_size();

        let surface = instance.create_surface(window.clone()).unwrap();
        let cap = surface.get_capabilities(&adapter);
        let surface_format = cap.formats[0];

        let command_buffer = Arc::new(Mutex::new(Vec::new()));
        let buffer_clone = Arc::clone(&command_buffer);

        thread::spawn(move || {
            crate::socket_listener::spawn_socket_listener("/tmp/wgpu-draw.sock", buffer_clone)
                .expect("failed to spawn socket listener");
        });

        let transform_layout = renderer::create_transform_bind_group_layout(&device);
        let pipeline = renderer::build_pipeline(&device, surface_format, &transform_layout);

        let state = State {
            instance,
            window,
            device,
            queue,
            size,
            surface,
            surface_format,
            command_buffer,
            pipeline,
            transforms: TransformBuffer::new(),
            primitives: PrimitiveBuffer::new(),
            clear_color: wgpu::Color::BLACK,
        };

        state.configure_surface();

        state
    }

    pub fn get_window(&self) -> &Window {
        &self.window
    }

    fn configure_surface(&self) {
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: self.surface_format,
            view_formats: vec![self.surface_format.add_srgb_suffix()],
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            width: self.size.width,
            height: self.size.height,
            desired_maximum_frame_latency: 2,
            present_mode: wgpu::PresentMode::AutoVsync,
        };
        self.surface.configure(&self.device, &surface_config);
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.configure_surface();
    }

    fn process_commands(&mut self, commands: &[DrawCommand]) {
        for cmd in commands {
            match cmd {
                DrawCommand::Clear { r, g, b, a } => {
                    self.clear_color = wgpu::Color {
                        r: *r as f64,
                        g: *g as f64,
                        b: *b as f64,
                        a: *a as f64,
                    };
                }
                DrawCommand::DrawTriangle { id, x1, y1, x2, y2, x3, y3, r, g, b, a } => {
                    let color = [*r, *g, *b, *a];
                    let mut vertices = Vec::new();
                    vertices.push(crate::renderer::Vertex { position: [*x1, *y1], color });
                    vertices.push(crate::renderer::Vertex { position: [*x2, *y2], color });
                    vertices.push(crate::renderer::Vertex { position: [*x3, *y3], color });
                    self.primitives.add(crate::primitive::Primitive::new(*id, vertices));
                }
                DrawCommand::DrawRect { id, x, y, w, h, r, g, b, a } => {
                    let color = [*r, *g, *b, *a];
                    let x1 = *x;
                    let y1 = *y;
                    let x2 = *x + *w;
                    let y2 = *y - *h;

                    let mut vertices = Vec::new();
                    vertices.push(crate::renderer::Vertex { position: [x1, y1], color });
                    vertices.push(crate::renderer::Vertex { position: [x2, y1], color });
                    vertices.push(crate::renderer::Vertex { position: [x1, y2], color });

                    vertices.push(crate::renderer::Vertex { position: [x2, y1], color });
                    vertices.push(crate::renderer::Vertex { position: [x2, y2], color });
                    vertices.push(crate::renderer::Vertex { position: [x1, y2], color });

                    self.primitives.add(crate::primitive::Primitive::new(*id, vertices));
                }
                DrawCommand::SetTransform { id, tx, ty, sx, sy } => {
                    self.transforms.set(*id, crate::transform::Transform::new(*tx, *ty, *sx, *sy, 0.0));
                }
                DrawCommand::Reset => {
                    self.primitives.clear();
                    self.transforms.clear();
                }
                DrawCommand::Present => {}
            }
        }
    }


    pub fn render(&mut self) {
        let commands = {
            let locked = self.command_buffer.lock().unwrap();
            locked.clone()
        };

        self.process_commands(&commands);

        let surface_texture = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture) => texture,
            wgpu::CurrentSurfaceTexture::Occluded | wgpu::CurrentSurfaceTexture::Timeout => return,
            wgpu::CurrentSurfaceTexture::Suboptimal(_) | wgpu::CurrentSurfaceTexture::Outdated => {
                self.configure_surface();
                return;
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                unreachable!("No error scope registered, so validation errors will panic")
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                self.surface = self.instance.create_surface(self.window.clone()).unwrap();
                self.configure_surface();
                return;
            }
        };

        let texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor {
                format: Some(self.surface_format.add_srgb_suffix()),
                ..Default::default()
            });

        let mut encoder = self.device.create_command_encoder(&Default::default());

        let mut renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &texture_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(self.clear_color),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        let transform_layout = renderer::create_transform_bind_group_layout(&self.device);

        let draw_calls: Vec<_> = self.primitives
            .get_all()
            .iter()
            .map(|p| {
                let vertex_buffer = self.device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("vertex_buffer"),
                        contents: bytemuck::cast_slice(&p.vertices),
                        usage: wgpu::BufferUsages::VERTEX,
                    },
                );

                let transform = self.transforms.get(p.id).unwrap_or_else(renderer::Transform::identity);
                let uniform = renderer::TransformUniform {
                    matrix: transform.to_matrix(),
                };

                let transform_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("transform_buffer_per_primitive"),
                    contents: bytemuck::cast_slice(&[uniform]),
                    usage: wgpu::BufferUsages::UNIFORM,
                });

                let transform_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &transform_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: transform_buffer.as_entire_binding(),
                    }],
                    label: Some("transform_bind_group_per_primitive"),
                });

                (vertex_buffer, transform_bind_group, p.vertices.len())
            })
            .collect();

        renderpass.set_pipeline(&self.pipeline);

        for (_i, (vertex_buffer, transform_bind_group, vertex_count)) in draw_calls.iter().enumerate() {
            renderpass.set_bind_group(0, transform_bind_group, &[]);
            renderpass.set_vertex_buffer(0, vertex_buffer.slice(..));
            renderpass.draw(0..*vertex_count as u32, 0..1);
        }

        drop(renderpass);

        self.queue.submit([encoder.finish()]);
        self.window.pre_present_notify();
        surface_texture.present();
    }
}
