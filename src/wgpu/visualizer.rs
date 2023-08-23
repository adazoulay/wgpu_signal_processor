use crate::audio::audio_state::{AudioStateMetatada, SpectrumType};

use wgpu::util::DeviceExt;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use rustfft::{num_complex::Complex, FftPlanner};

struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    window: Window,
    slice_size: usize,
    vertex_buffer: wgpu::Buffer,
    audio_data_buffer: Vec<f32>,
    bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
}

impl State {
    async fn new(window: Window, audio_state: &AudioStateMetatada) -> Self {
        // --------- SETUP --------- //

        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });
        let surface = unsafe { instance.create_surface(&window) }.unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                },
                None,
            )
            .await
            .unwrap();

        // * Surface Config
        let surface_caps = surface.get_capabilities(&adapter);

        let srgb_formats = [
            wgpu::TextureFormat::Bgra8UnormSrgb,
            wgpu::TextureFormat::Rgba8UnormSrgb,
        ];
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| srgb_formats.contains(f))
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        // --------- DATA --------- //

        // ! Buffers

        // * Vertex Buffer

        let buffer_size = (std::mem::size_of::<f32>() * audio_state.slice_size) as u64;

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Vertex Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let vertex_buffer_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<f32>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32,
            }],
        };

        // * Uniform Buffer
        let uniform_array: [f32; 4] = [
            audio_state.get_max_amplitude() as f32,
            audio_state.slice_size as f32,
            0 as f32,
            0 as f32,
        ];

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&uniform_array),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // ! Shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        // ! Bind Group

        // * Bind Group Layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bindgroup layout"),
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
        });

        // * Bind Group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bindgrop"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        // ! Render Pipeline
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: match audio_state.spectrum_type {
                    SpectrumType::Time => "time_domain_main",
                    SpectrumType::Frequency => "freq_domain_main",
                },
                buffers: &[vertex_buffer_layout],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineStrip,
                strip_index_format: Some(wgpu::IndexFormat::Uint32),
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        Self {
            surface,
            device,
            queue,
            config,
            size,
            window,
            slice_size: audio_state.slice_size,
            audio_data_buffer: Vec::new(),
            vertex_buffer,
            bind_group,
            render_pipeline,
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
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
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[
                    // This is what @location(0) in the fragment shader targets
                    Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.0,
                                g: 0.0,
                                b: 0.0,
                                a: 1.0,
                            }),
                            store: true,
                        },
                    }),
                ],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.bind_group, &[]);

            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..(self.slice_size as u32), 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    fn update_vertex_buffer(&mut self, mut new_slice: Vec<f32>) {
        if new_slice.len() > self.slice_size {
            new_slice.truncate(self.slice_size);
        }
        self.audio_data_buffer.extend(new_slice);

        if self.audio_data_buffer.len() >= self.slice_size {
            self.queue.write_buffer(
                &self.vertex_buffer,
                0,
                bytemuck::cast_slice(&self.audio_data_buffer[0..self.slice_size]),
            );
            self.audio_data_buffer.drain(..self.slice_size);
        }
    }
}

pub async fn run_visualizer(
    audio_state: AudioStateMetatada,
    rx: std::sync::mpsc::Receiver<Vec<f32>>,
) {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        use winit::dpi::PhysicalSize;
        window.set_inner_size(PhysicalSize::new(450, 400));

        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("wasm-example")?;
                let canvas = web_sys::Element::from(window.canvas());
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }

    // ! STATE SETUP
    let mut state = State::new(window, &audio_state).await;

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window().id() => {
                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        // new_inner_size is &&mut so w have to dereference it twice
                        state.resize(**new_inner_size);
                    }
                    _ => {}
                }
            }
            // ! Redraw Request
            Event::RedrawRequested(window_id) if window_id == state.window().id() => {
                let mut chunks = Vec::new();
                while let Ok(chunk) = rx.try_recv() {
                    // println!("chunk {}", chunk.len());
                    chunks.push(chunk);
                }

                if chunks.is_empty() {
                    return;
                }

                let processed = match audio_state.spectrum_type {
                    SpectrumType::Frequency => {
                        let slice_size = audio_state.slice_size;
                        // println!("slice_size {}", slice_size);
                        let averaged_chunk = average_chunks(chunks, slice_size);
                        compute_fft(averaged_chunk, slice_size)
                    }
                    SpectrumType::Time => {
                        let slice_size = audio_state.slice_size;
                        // println!("slice_size {}", slice_size);
                        average_chunks(chunks, slice_size)
                    }
                };

                state.update_vertex_buffer(processed);

                match state.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if it's lost or outdated
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        state.resize(state.size)
                    }
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,

                    Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
                }
            }
            Event::RedrawEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                state.window().request_redraw();
            }
            _ => {}
        }
    });
}

impl State {
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn window(&self) -> &Window {
        &self.window
    }
}

fn average_chunks(chunks: Vec<Vec<f32>>, slice_size: usize) -> Vec<f32> {
    if chunks.is_empty() {
        return Vec::new();
    }

    let chunk_len = chunks.len();
    let mut averaged_chunk = vec![0.0; slice_size];

    for i in 0..slice_size {
        let sum: f32 = chunks
            .iter()
            .map(|chunk| chunk.get(i % chunk.len()).unwrap_or(&0.0))
            .sum();
        averaged_chunk[i] = sum / chunk_len as f32;
    }

    averaged_chunk
}

pub fn compute_fft(chunk: Vec<f32>, slice_size: usize) -> Vec<f32> {
    let fft = FftPlanner::new().plan_fft_forward(slice_size * 2); // double the FFT size

    let window: Vec<f32> = (0..slice_size)
        .map(|i| {
            0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / (slice_size - 1) as f32).cos())
        })
        .collect();

    // Apply FFT to the slice with zero padding
    let mut fft_input: Vec<Complex<f32>> = chunk
        .iter()
        .enumerate()
        .map(|(i, &x)| Complex::new(x * window[i], 0.0))
        .collect();

    // Zero padding
    fft_input.resize(slice_size * 2, Complex::new(0.0, 0.0));
    fft.process(&mut fft_input);

    // Convert to magnitude and normalize FFT output, return only first half of the data
    fft_input
        .iter()
        .take(slice_size) // Only take first half of the data
        .map(|x| (x.norm() / (slice_size * 2) as f32).sqrt()) // sqrt for perceptual scaling, divided by slice_size to normalize FFT output
        .collect()
}
