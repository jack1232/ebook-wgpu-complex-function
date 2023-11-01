use bytemuck::cast_slice;
use cgmath::Matrix4;
use std::{iter, borrow::Cow};
use wgpu::{util::DeviceExt, VertexBufferLayout};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};
use wgpu_simplified as ws;
use wgpu_complex_function::colormap;

#[include_wgsl_oil::include_wgsl_oil("complex3d_comp.wgsl")]
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
    uniform_buffer: wgpu::Buffer,

    cs_pipelines: Vec<wgpu::ComputePipeline>,
    cs_vertex_buffer: wgpu::Buffer,
    cs_index_buffer: wgpu::Buffer,
    cs_uniform_buffers: Vec<wgpu::Buffer>,
    cs_bind_groups: Vec<wgpu::BindGroup>,

    view_mat: Matrix4<f32>,
    project_mat: Matrix4<f32>,
    msaa_texture_view: wgpu::TextureView,
    depth_texture_view: wgpu::TextureView,
    animation_speed: f32,
    rotation_speed: f32,

    resolution: u32,
    triangles_count: u32,
    surface_type: u32,
    scale: f32,
    aspect_ratio: f32,
    fps_counter: ws::FpsCounter,
}

impl State {
    async fn new(window: &Window, sample_count: u32, resolution: u32, colormap_name: &str) -> Self {
        let init = ws::IWgpuInit::new(&window, sample_count, None).await;

        let resol = ws::round_to_multiple(resolution, 8);
        let vertices_count = resol * resol;
        let triangles_count = 6 * (resol - 1) * (resol - 1);
        println!("resolution = {}", resol);

        let shader = init.device.create_shader_module(wgpu::include_wgsl!("../ch01/shader_unlit.wgsl"));
        let cs_indices = init.device.create_shader_module(wgpu::include_wgsl!("indices_comp.wgsl"));
        let cs_comp = init.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Compute Shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::from(cs_shader_mod::SOURCE)),
        });

        // uniform data
        let camera_position = (2.0, 2.0, 2.0).into();
        let look_direction = (0.0, 0.0, 0.0).into();
        let up_direction = cgmath::Vector3::unit_y();

        let (view_mat, project_mat, _) = ws::create_vp_mat(
            camera_position,
            look_direction,
            up_direction,
            init.config.width as f32 / init.config.height as f32,
        );

        // create vertex uniform buffers
        let vert_uniform_buffer = init.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Vertex Uniform Buffer"),
            size: 64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // uniform bind group for vertex shader
        let (vert_bind_group_layout, vert_bind_group) = ws::create_bind_group(
            &init.device,
            vec![wgpu::ShaderStages::VERTEX],
            &[vert_uniform_buffer.as_entire_binding()],
        );

        let vertex_buffer_layout = VertexBufferLayout {
            array_stride: 32,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x4], // pos, col
        };

        let pipeline_layout = init
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&vert_bind_group_layout],
                push_constant_ranges: &[],
            });

        let mut ppl = ws::IRenderPipeline {
            shader: Some(&shader),
            pipeline_layout: Some(&pipeline_layout),
            vertex_buffer_layout: &[vertex_buffer_layout],
            ..Default::default()
        };
        let pipeline = ppl.new(&init);

        let msaa_texture_view = ws::create_msaa_texture_view(&init);
        let depth_texture_view = ws::create_depth_view(&init);

        // create compute pipeline for indices
        let cs_index_buffer = init.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Index Buffer"),
            size: 4 * triangles_count as u64,
            usage: wgpu::BufferUsages::INDEX
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let cs_index_uniform_buffer = init.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Index Uniform Buffer"),
            size: 4,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        init.queue
            .write_buffer(&cs_index_uniform_buffer, 0, cast_slice(&[resol]));

        let (cs_index_bind_group_layout, cs_index_bind_group) = ws::create_bind_group_storage(
            &init.device,
            vec![wgpu::ShaderStages::COMPUTE, wgpu::ShaderStages::COMPUTE],
            vec![
                wgpu::BufferBindingType::Storage { read_only: false },
                wgpu::BufferBindingType::Uniform,
            ],
            &[
                cs_index_buffer.as_entire_binding(),
                cs_index_uniform_buffer.as_entire_binding(),
            ],
        );

        let cs_index_pipeline_layout =
            init.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Compute Index Pipeline Layout"),
                    bind_group_layouts: &[&cs_index_bind_group_layout],
                    push_constant_ranges: &[],
                });

        let cs_index_pipeline =
            init.device
                .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some("Compute Index Pipeline"),
                    layout: Some(&cs_index_pipeline_layout),
                    module: &cs_indices,
                    entry_point: "cs_main",
                });

        // create compute pipeline for complex3d surface
        let cs_vertex_buffer = init.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Vertex Buffer"),
            size: 32 * vertices_count as u64,
            usage: wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let cdata = create_color_data(colormap_name);
        let cs_colormap_buffer =
            init.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Colormap Uniform Buffer"),
                    contents: bytemuck::cast_slice(&cdata),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

        let cs_vertex_uniform_buffer = init.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Vertex Uniform Buffer"),
            size: 32,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let (cs_vertex_bind_group_layout, cs_vertex_bind_group) = ws::create_bind_group_storage(
            &init.device,
            vec![
                wgpu::ShaderStages::COMPUTE,
                wgpu::ShaderStages::COMPUTE,
                wgpu::ShaderStages::COMPUTE,
            ],
            vec![
                wgpu::BufferBindingType::Storage { read_only: false },
                wgpu::BufferBindingType::Uniform,
                wgpu::BufferBindingType::Uniform,
            ],
            &[
                cs_vertex_buffer.as_entire_binding(),
                cs_colormap_buffer.as_entire_binding(),
                cs_vertex_uniform_buffer.as_entire_binding(),
            ],
        );

        let cs_pipeline_layout =
            init.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Compute Pipeline Layout"),
                    bind_group_layouts: &[&cs_vertex_bind_group_layout],
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
            uniform_bind_group: vert_bind_group,
            uniform_buffer: vert_uniform_buffer,

            cs_pipelines: vec![cs_index_pipeline, cs_pipeline],
            cs_vertex_buffer,
            cs_index_buffer,
            cs_uniform_buffers: vec![cs_index_uniform_buffer, cs_vertex_uniform_buffer],
            cs_bind_groups: vec![cs_index_bind_group, cs_vertex_bind_group],

            view_mat,
            project_mat,
            msaa_texture_view,
            depth_texture_view,
            animation_speed: 1.0,
            rotation_speed: 1.0,

            resolution: resol,
            triangles_count,
            surface_type: 0,
            scale: 1.5,
            aspect_ratio: 0.8,
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

            self.project_mat =
                ws::create_projection_mat(new_size.width as f32 / new_size.height as f32, true);
            self.depth_texture_view = ws::create_depth_view(&self.init);
            if self.init.sample_count > 1 {
                self.msaa_texture_view = ws::create_msaa_texture_view(&self.init);
            }
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
                    self.surface_type = (self.surface_type + 1) % 11;
                    println!("function = {}", self.surface_type);
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
                VirtualKeyCode::W => {
                    self.rotation_speed += 0.1;
                    true
                }
                VirtualKeyCode::S => {
                    self.rotation_speed -= 0.1;
                    if self.rotation_speed < 0.0 {
                        self.rotation_speed = 0.0;
                    }
                    true
                }
                _ => false,
            },
            _ => false,
        }
    }

    fn update(&mut self, dt: std::time::Duration) {
        // update uniform buffer
        let dt1 = self.rotation_speed * dt.as_secs_f32();

        let model_mat = ws::create_model_mat(
            [0.0, 0.5, 0.0],
            [dt1.sin(), dt1.cos(), 0.0],
            [1.0, 1.0, 1.0],
        );
        let mvp_mat = self.project_mat * self.view_mat * model_mat;
        self.init.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(mvp_mat.as_ref() as &[f32; 16]),
        );

        // update uniform buffer for compute pipeline
        let params = [
            self.resolution as f32,
            self.surface_type as f32,
            0.5 * (1.0 + (self.animation_speed * 2.0 * dt.as_secs_f32()).cos()),
            self.scale,
            self.aspect_ratio,
        ];
        self.init
            .queue
            .write_buffer(&self.cs_uniform_buffers[1], 0, cast_slice(&params));
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

        // compute pass for indices
        {
            let mut cs_index_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Compute Index Pass"),
            });
            cs_index_pass.set_pipeline(&self.cs_pipelines[0]);
            cs_index_pass.set_bind_group(0, &self.cs_bind_groups[0], &[]);
            cs_index_pass.dispatch_workgroups(self.resolution / 8, self.resolution / 8, 1);
        }

        // compute pass for vertices
        {
            let mut cs_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Compute Pass"),
            });
            cs_pass.set_pipeline(&self.cs_pipelines[1]);
            cs_pass.set_bind_group(0, &self.cs_bind_groups[1], &[]);
            cs_pass.dispatch_workgroups(self.resolution / 8, self.resolution / 8, 1);
        }

        // render pass
        {
            let color_attach = ws::create_color_attachment(&view);
            let msaa_attach = ws::create_msaa_color_attachment(&view, &self.msaa_texture_view);
            let color_attachment = if self.init.sample_count == 1 {
                color_attach
            } else {
                msaa_attach
            };
            let depth_attachment = ws::create_depth_stencil_attachment(&self.depth_texture_view);

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(color_attachment)],
                depth_stencil_attachment: Some(depth_attachment),
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_vertex_buffer(0, self.cs_vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.cs_index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            render_pass.draw_indexed(0..self.triangles_count, 0, 0..1);
        }
        self.fps_counter.print_fps(5);
        self.init.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

fn main() {
    let mut sample_count = 1 as u32;
    let mut resolution = 1024u32;
    let mut colormap_name = "jet";

    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        sample_count = args[1].parse::<u32>().unwrap();
    }
    if args.len() > 2 {
        resolution = args[2].parse::<u32>().unwrap();
    }
    if args.len() > 3 {
        colormap_name = &args[3];
    }

    env_logger::init();
    let event_loop = EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .build(&event_loop)
        .unwrap();
    window.set_title(&*format!("ch10_{}", "complex3d_gpu"));

    let mut state =
        pollster::block_on(State::new(&window, sample_count, resolution, colormap_name));
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