use anyhow::Result;
use wgpu::{util::DeviceExt, *};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ModifiersState, MouseScrollDelta},
};

use crate::{
    character::Character, character_buffer::CharacterBuffer, texture, vertex::Vertex, vertices,
};

#[rustfmt::skip]
const VERTICES: [Vertex; 6] = vertices!(
    (-1.0, -1.0,  0.0),
    ( 1.0, -1.0,  0.0),
    (-1.0,  1.0,  0.0),
    ( 1.0, -1.0,  0.0),
    ( 1.0,  1.0,  0.0),
    (-1.0,  1.0,  0.0)
);

pub struct State {
    size: PhysicalSize<u32>,

    surface: Surface,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    base_render_pipeline: RenderPipeline,
    final_render_pipeline: RenderPipeline,

    base_texture: Texture,
    base_texture_bind_group: BindGroup,

    vertex_buffer: Buffer,
    character_buffer: Buffer,
    scale_factor_uniform: Buffer,
    font_texture_bind_group: BindGroup,
    character_buffer_bind_group: BindGroup,

    characters: CharacterBuffer,
    scale_factor: f32,

    modifiers_state: ModifiersState,
}

impl State {
    pub async fn new(window: &winit::window::Window) -> Result<Self> {
        let size = window.inner_size();

        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::all(),
            dx12_shader_compiler: Dx12Compiler::Fxc,
        });

        let surface = unsafe { instance.create_surface(&window) }?;

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("no suitable adapter");

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: Some("Device"),
                    features: Features::default(),
                    limits: Limits::default(),
                },
                None,
            )
            .await?;

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: TextureFormat::Bgra8Unorm,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Fifo,
            alpha_mode: CompositeAlphaMode::Opaque,
            view_formats: vec![TextureFormat::Bgra8Unorm],
        };

        surface.configure(&device, &config);

        let vertex_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&VERTICES),
            usage: BufferUsages::VERTEX,
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("Texture Bind Group Layout"),
            });

        let font_texture = texture::Texture::from_bytes(
            &device,
            &queue,
            include_bytes!("../assets/font.png"),
            "Font Texture",
        )?;

        let font_texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&font_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&font_texture.sampler),
                },
            ],
            label: Some("Texture Bind Group"),
        });

        let base_texture_size = wgpu::Extent3d {
            width: 800,
            height: 600,
            depth_or_array_layers: 1,
        };

        let base_texture = device.create_texture(&TextureDescriptor {
            label: Some("Base Texture"),
            size: base_texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8Unorm,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let base_texture_view = base_texture.create_view(&TextureViewDescriptor::default());
        let base_texture_sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });

        let base_texture_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Base Texture Bind Group"),
            layout: &texture_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&base_texture_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&base_texture_sampler),
                },
            ],
        });

        let scale_factor = 0.5;

        let characters = CharacterBuffer::new((
            f32::floor((size.width / 10) as f32 * scale_factor) as u32,
            f32::floor((size.height / 10) as f32 * scale_factor) as u32,
        ));

        let character_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("Character Buffer"),
            contents: bytemuck::cast_slice(characters.buffer()),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        });

        let scale_factor_uniform = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("Scale Factor Uniform Buffer"),
            contents: bytemuck::bytes_of(&scale_factor),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let character_buffer_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Character Buffer Bind Group Layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let character_buffer_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Character Buffer Bind Group"),
            layout: &character_buffer_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Buffer(BufferBinding {
                        buffer: &character_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Buffer(BufferBinding {
                        buffer: &scale_factor_uniform,
                        offset: 0,
                        size: None,
                    }),
                },
            ],
        });

        let base_shader = device.create_shader_module(include_wgsl!("shader_base.wgsl"));
        let final_shader = device.create_shader_module(include_wgsl!("shader_final.wgsl"));

        let base_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"),
            bind_group_layouts: &[
                &texture_bind_group_layout,
                &character_buffer_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let base_render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&base_pipeline_layout),
            vertex: VertexState {
                module: &base_shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(FragmentState {
                module: &base_shader,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: TextureFormat::Bgra8Unorm,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });

        let final_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"),
            bind_group_layouts: &[&texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        let final_render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Final Render Pipeline"),
            layout: Some(&final_pipeline_layout),
            vertex: VertexState {
                module: &final_shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(FragmentState {
                module: &final_shader,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: TextureFormat::Bgra8Unorm,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });

        let new = Self {
            size,

            surface,
            device,
            queue,
            config,
            base_render_pipeline,
            final_render_pipeline,

            base_texture,
            base_texture_bind_group,

            vertex_buffer,
            character_buffer,
            scale_factor_uniform,
            font_texture_bind_group,
            character_buffer_bind_group,

            characters,
            scale_factor,

            modifiers_state: ModifiersState::empty(),
        };

        new.render_base_texture();

        Ok(new)
    }

    pub fn input(&mut self, event: winit::event::WindowEvent) -> bool {
        match event {
            winit::event::WindowEvent::MouseWheel { delta, .. } if self.modifiers_state.ctrl() => {
                self.scale_factor += f32::signum(match delta {
                    MouseScrollDelta::LineDelta(_, v) => v,
                    MouseScrollDelta::PixelDelta(PhysicalPosition { y, .. }) => y as f32,
                }) * -0.1;

                self.queue.write_buffer(
                    &self.scale_factor_uniform,
                    0,
                    bytemuck::bytes_of(&self.scale_factor),
                );

                true
            }
            winit::event::WindowEvent::ReceivedCharacter(c) => {
                self.characters.push_char(Character::new(
                    [0.0, 0.0, 0.0],
                    [1.0, 1.0, 1.0],
                    c as u32,
                ));

                self.characters
                    .write_changes(&self.queue, &self.character_buffer);

                self.render_base_texture();

                true
            }
            winit::event::WindowEvent::ModifiersChanged(new_state) => {
                self.modifiers_state = new_state;
                false
            }
            _ => false,
        }
    }

    pub fn render(&self) -> Result<(), SurfaceError> {
        let output_texture = self.surface.get_current_texture()?;
        let view = output_texture.texture.create_view(&TextureViewDescriptor {
            label: Some("Output Texture View"),
            ..Default::default()
        });

        let mut commands = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Command Encoder"),
            });

        {
            let mut render_pass = commands.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.final_render_pipeline);
            render_pass.set_bind_group(0, &self.base_texture_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..6, 0..1);
        }

        self.queue.submit(std::iter::once(commands.finish()));
        output_texture.present();

        Ok(())
    }

    fn render_base_texture(&self) {
        let view = self.base_texture.create_view(&TextureViewDescriptor {
            label: Some("Base Texture View"),
            ..Default::default()
        });

        let mut commands = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Base Texture Render Command Encoder"),
            });

        {
            let mut render_pass = commands.begin_render_pass(&RenderPassDescriptor {
                label: Some("Base Texture Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.base_render_pipeline);
            render_pass.set_bind_group(0, &self.font_texture_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_bind_group(1, &self.character_buffer_bind_group, &[]);
            render_pass.draw(0..6, 0..1);
        }

        self.queue.submit(std::iter::once(commands.finish()));
    }

    pub fn size(&self) -> PhysicalSize<u32> {
        self.size
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }
}
