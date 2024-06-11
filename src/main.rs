use std::time::Instant;
use rand::prelude::*;
use std::{mem, slice};
use winit::keyboard::KeyCode;
use winit::{event::*, event_loop::EventLoop, window::WindowBuilder};
use wgpu::util::DeviceExt;
use glam::{Vec3, Vec4, Mat4};
use log::LevelFilter;

type Result<T = (), E = Box<dyn std::error::Error>> = std::result::Result<T, E>;
const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;

struct Camera {
    eye: Vec3,
    target: Vec3,
    up: Vec3,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

fn main() -> Result {
    // WINDOW
    env_logger::builder().filter_level(LevelFilter::Info).init();

    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new().with_title("Fur Shader").build(&event_loop)?;

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
    let surface = unsafe { instance.create_surface(&window)? };
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::default(),
        force_fallback_adapter: false,
        compatible_surface: Some(&surface),
    }))
    .unwrap();

    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            features: wgpu::Features::empty(),
            limits: wgpu::Limits::default(),
            label: None,
        },
        None,
    ))
    .unwrap();

    // TIME
    let start_time = Instant::now();

    let time_buf = device.create_buffer(&wgpu::BufferDescriptor {
        size: 4,
        mapped_at_creation: false,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        label: None,
    });
    let time_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
        label: None,
    });
    let time_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &time_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: time_buf.as_entire_binding(),
        }],
        label: None,
    });

    // CAMERA
    let mut camera = Camera {
        eye: (0.0, 1.0, 30.0).into(),
        target: (0.0, 0.0, 0.0).into(),
        up: Vec3::new(0.0, 1.0, 0.0),
        aspect: window.inner_size().width as f32 / window.inner_size().height as f32,
        fovy: 45.0,
        znear: 0.1,
        zfar: 100.0,
    };

    // CAMERA UNIFORM
    let view = Mat4::look_at_rh(camera.eye, camera.target, camera.up);
    let proj = Mat4::perspective_rh(
        camera.fovy.to_radians(),
        camera.aspect,
        camera.znear,
        camera.zfar,
    );
    let camera_uniform = proj * view;

    let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        contents: cast_slice(&[camera_uniform]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        label: None,
    });

    let camera_bind_group_layout =
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
        label: None,
    });

    let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &camera_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: camera_buffer.as_entire_binding(),
        }],
        label: None,
    });

    let mut vertices: Vec<Vec4> = vec![];
    let mut rng = rand::thread_rng();
    for _ in 0..500 {
        vertices.push( Vec4::new(
            rng.gen_range(0.0..1.0),
            rng.gen_range(0.0..1.0),
            rng.gen_range(0.0..1.0),
            1.0,
        ));
    }

    println!("{:?}", vertices);

    let vtx_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        contents: cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
        label: None,
    });

    // DEPTH BUFFER
    const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
    let size = wgpu::Extent3d {
        width: window.inner_size().width,
        height: window.inner_size().height,
        depth_or_array_layers: 1,
    };
    let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: DEPTH_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let mut depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

    // SHADERS
    let shader = device.create_shader_module(wgpu::include_spirv!(env!("shaders.spv")));

    // PIPELINE
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        bind_group_layouts: &[
            &camera_bind_group_layout,
            &time_bind_group_layout,
        ],
        push_constant_ranges: &[],
        label: None,
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "main_vs",
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: mem::size_of::<Vec4>() as _,
                step_mode: wgpu::VertexStepMode::Instance,
                attributes: &wgpu::vertex_attr_array![0 => Float32x4],
            }],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "main_fs",
            targets: &[Some(wgpu::ColorTargetState {
                format: FORMAT,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::PointList,
            ..Default::default()
        },  
        depth_stencil: Some(wgpu::DepthStencilState {
            format: DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        label: None,
    });

    // EVENT LOOP
    event_loop.run(move |event, elwt| match event {
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::CloseRequested => elwt.exit(),

            WindowEvent::KeyboardInput { event, .. } => {
                if event.physical_key == KeyCode::Escape {
                    elwt.exit();
                };
            },

            WindowEvent::RedrawRequested => {
                let duration = start_time.elapsed();
                queue.write_buffer(&time_buf, 0, cast_slice(&[duration.as_secs_f32()]));
                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

                let surface = surface.get_current_texture().unwrap();
                let surface_view = surface
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &surface_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        },
                    })],

                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &depth_view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    label: None,
                });

                render_pass.set_pipeline(&pipeline);
                render_pass.set_bind_group(0, &camera_bind_group, &[]);
                render_pass.set_bind_group(1, &time_bind_group, &[]);
                render_pass.set_vertex_buffer(0, vtx_buf.slice(..));

                render_pass.draw(0..vertices.len() as u32, 0..1);
                drop(render_pass);
                queue.submit([encoder.finish()]);
                surface.present();
            }

            WindowEvent::Resized(size) => {
                let config = wgpu::SurfaceConfiguration {
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    format: FORMAT,
                    width: size.width,
                    height: size.height,
                    present_mode: wgpu::PresentMode::Fifo,
                    alpha_mode: wgpu::CompositeAlphaMode::Opaque,
                    view_formats: vec![],
                };
                surface.configure(&device, &config);

                //FIXING FOV
                camera.aspect = window.inner_size().width as f32 / window.inner_size().height as f32;
                let proj = Mat4::perspective_rh(
                    camera.fovy.to_radians(),
                    camera.aspect,
                    camera.znear,
                    camera.zfar,
                );
                let camera_uniform = proj * view;
                queue.write_buffer(&camera_buffer, 0, cast_slice(&[camera_uniform]));

                // FIXING DEPTH BUFFER
                let size = wgpu::Extent3d {
                    width: window.inner_size().width,
                    height: window.inner_size().height,
                    depth_or_array_layers: 1,
                };
                let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
                    label: None,
                    size,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: DEPTH_FORMAT,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                });
                depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
            }
            _ => {}
        },
        Event::AboutToWait => window.request_redraw(),
        _ => {}
    })?;
    Ok(())
}

fn cast_slice<T>(fake: &[T]) -> &[u8] {
    unsafe { slice::from_raw_parts(fake.as_ptr() as _, mem::size_of_val(fake)) }
}
