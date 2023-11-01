use bytemuck::cast_slice;
use std::{iter, borrow::Cow};
use wgpu::util::DeviceExt;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};
use wgpu_simplified as ws;
use wgpu_simplified::texture_data as td;
use wgpu_complex_function::colormap;

#[include_wgsl_oil::include_wgsl_oil("domain_color_comp.wgsl")]
mod cs_shader_mod {}

fn create_color_data(colormap_name: &str) -> Vec<[f32; 4]> {
    let cdata = colormap::colormap_data(colormap_name);
    let mut data: Vec<[f32; 4]> = vec![];
    for i in 0..cdata.len() {
        data.push([cdata[i][0], cdata[i][1], cdata[i][2], 1.0]);
    }
    data
}
struct State {
    init: ws::IWgpuInit,
    pipeline: wgpu::RenderPipeline,
    uniform_bind_group: wgpu::BindGroup,

    cs_pipeline: wgpu::ComputePipeline,
    cs_uniform_buffers: Vec<wgpu::Buffer>,
    cs_bind_groups: Vec<wgpu::BindGroup>,

    animation_speed: f32,
    function_type: u32,
    colormap_type: u32,
    scale: f32,
    fps_counter: ws::FpsCounter,
}

impl State {
    async fn new(window: &Window, colormap_name: &str) -> Self {
        let init = ws::IWgpuInit::new(&window, 1, None).await;

        let shader = init.device.create_shader_module(wgpu::include_wgsl!("render_shader.wgsl"));

        let cs_comp = init.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Compute Shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::from(cs_shader_mod::SOURCE)),
        });

        let tex = td::ITexture::create_texture_store_data(
            &init.device,
            init.size.width,
            init.size.height,
        )
        .unwrap();
        let (texture_bind_group_layout, texture_bind_group) =
            ws::create_texture_store_bind_group(&init.device, &tex);

        let pipeline_layout = init
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        let mut ppl = ws::IRenderPipeline {
            shader: Some(&shader),
            pipeline_layout: Some(&pipeline_layout),
            vertex_buffer_layout: &[],
            is_depth_stencil: false,
            ..Default::default()
        };
        let pipeline = ppl.new(&init);

        // create compute pipeline for domain coloring
        let cdata = create_color_data(colormap_name);
        let cs_colormap_buffer =
            init.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Colormap Uniform Buffer"),
                    contents: bytemuck::cast_slice(&cdata),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

        let cs_int_uniform_buffer = init.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Compute Integer Uniform Buffer"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let cs_float_uniform_buffer = init.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Compute float Uniform Buffer"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let (cs_bind_group_layout, cs_bind_group) = ws::create_bind_group_storage(
            &init.device,
            vec![
                wgpu::ShaderStages::COMPUTE,
                wgpu::ShaderStages::COMPUTE,
                wgpu::ShaderStages::COMPUTE,
            ],
            vec![
                wgpu::BufferBindingType::Uniform,
                wgpu::BufferBindingType::Uniform,
                wgpu::BufferBindingType::Uniform,
            ],
            &[
                cs_colormap_buffer.as_entire_binding(),
                cs_int_uniform_buffer.as_entire_binding(),
                cs_float_uniform_buffer.as_entire_binding(),
            ],
        );

        let (cs_texture_bind_group_layout, cs_texture_bind_group) =
            ws::create_compute_texture_bind_group(&init.device, &tex.view);

        let cs_pipeline_layout =
            init.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Compute Pipeline Layout"),
                    bind_group_layouts: &[&cs_bind_group_layout, &cs_texture_bind_group_layout],
                    push_constant_ranges: &[],
                });

        let cs_pipeline = init
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Compute Pipeline"),
                layout: Some(&cs_pipeline_layout),
                module: &cs_comp,
                entry_point: "cs_main",
            });

        Self {
            init,
            pipeline,
            uniform_bind_group: texture_bind_group,

            cs_pipeline,
            cs_uniform_buffers: vec![
                cs_colormap_buffer,
                cs_int_uniform_buffer,
                cs_float_uniform_buffer,
            ],
            cs_bind_groups: vec![cs_bind_group, cs_texture_bind_group],

            animation_speed: 1.0,
            function_type: 0,
            colormap_type: 0,
            scale: 5.0,
            fps_counter: ws::FpsCounter::default(),
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.init.size = new_size;
            self.init.config.width = new_size.width;
            self.init.config.height = new_size.height;
            self.init
                .surface
                .configure(&self.init.device, &self.init.config);

            // update texture bind groups for both render and compute pipelines when resizing
            let tex = td::ITexture::create_texture_store_data(
                &self.init.device,
                self.init.size.width,
                self.init.size.height,
            )
            .unwrap();
            let (_, texture_bind_group) =
                ws::create_texture_store_bind_group(&self.init.device, &tex);
            self.uniform_bind_group = texture_bind_group;

            let (_, cs_texture_bind_group) =
                ws::create_compute_texture_bind_group(&self.init.device, &tex.view);

            self.cs_bind_groups[1] = cs_texture_bind_group;
        }
    }

    #[allow(unused_variables)]
    fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        virtual_keycode: Some(keycode),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => match keycode {
                VirtualKeyCode::Space => {
                    self.function_type = (self.function_type + 1) % 13;
                    println!("function = {}", self.function_type);
                    true
                }
                VirtualKeyCode::LControl => {
                    self.colormap_type = (self.colormap_type + 1) % 2;
                    true
                }
                VirtualKeyCode::Q => {
                    self.animation_speed += 0.1;
                    true
                }
                VirtualKeyCode::A => {
                    self.animation_speed -= 0.1;
                    if self.animation_speed < 0.0 {
                        self.animation_speed = 0.0;
                    }
                    true
                }
                _ => false,
            },
            _ => false,
        }
    }

    fn update(&mut self, dt: std::time::Duration) {
        // update uniform buffer for compute pipeline
        let int_params = [self.function_type, self.colormap_type];
        self.init
            .queue
            .write_buffer(&self.cs_uniform_buffers[1], 0, cast_slice(&int_params));

        let dt1 = self.animation_speed * dt.as_secs_f32();
        let float_params = [
            0.5 * (1.0 + dt1.cos()),
            self.init.size.width as f32,
            self.init.size.height as f32,
            self.scale,
        ];
        self.init
            .queue
            .write_buffer(&self.cs_uniform_buffers[2], 0, cast_slice(&float_params));
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.init.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder =
            self.init
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        // compute pass for vertices
        {
            let mut cs_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Compute Pass"),
            });
            cs_pass.set_pipeline(&self.cs_pipeline);
            cs_pass.set_bind_group(0, &self.cs_bind_groups[0], &[]);
            cs_pass.set_bind_group(1, &self.cs_bind_groups[1], &[]);
            cs_pass.dispatch_workgroups(self.init.size.width / 8, self.init.size.height / 8, 1);
        }

        // render pass
        {
            let color_attachment = ws::create_color_attachment(&view);
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(color_attachment)],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            render_pass.draw(0..6, 0..1);
        }
        self.fps_counter.print_fps(5);
        self.init.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

fn main() {
    let mut colormap_name = "jet";
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        colormap_name = &args[1];
    }

    env_logger::init();
    let event_loop = EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .build(&event_loop)
        .unwrap();
    window.set_title(&*format!("{}", "domain_color"));

    let mut state = pollster::block_on(State::new(&window, colormap_name));
    let render_start_time = std::time::Instant::now();

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => {
            if !state.input(event) {
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
                        state.resize(**new_inner_size);
                    }
                    _ => {}
                }
            }
        }
        Event::RedrawRequested(_) => {
            let now = std::time::Instant::now();
            let dt = now - render_start_time;
            state.update(dt);

            match state.render() {
                Ok(_) => {}
                Err(wgpu::SurfaceError::Lost) => state.resize(state.init.size),
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                Err(e) => eprintln!("{:?}", e),
            }
        }
        Event::MainEventsCleared => {
            window.request_redraw();
        }
        _ => {}
    });
}