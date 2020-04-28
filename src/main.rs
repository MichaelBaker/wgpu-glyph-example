use std::io::*;

fn main() {
    futures::executor::block_on(render_thread());
}

async fn render_thread() {
    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::Window::new(&event_loop).unwrap();

    let width = window.inner_size().width;
    let window_size = window.inner_size();
    let surface = wgpu::Surface::create(&window);

    let adapter = wgpu::Adapter::request(
        &wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
        },
        wgpu::BackendBit::PRIMARY,
    )
    .await
    .unwrap();

    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            extensions: wgpu::Extensions {
                anisotropic_filtering: false,
            },
            limits: wgpu::Limits::default(),
        })
        .await;

    let swap_chain_descriptor = wgpu::SwapChainDescriptor {
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        format: wgpu::TextureFormat::Bgra8UnormSrgb,
        width: window_size.width,
        height: window_size.height,
        present_mode: wgpu::PresentMode::Fifo,
    };

    let mut swap_chain = device.create_swap_chain(&surface, &swap_chain_descriptor);

    let mut inconsolata = load_font(
        "assets/fonts/Inconsolata-Regular.ttf",
        &device,
        swap_chain_descriptor.format,
    );
    let height = window.inner_size().height;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = winit::event_loop::ControlFlow::Poll;

        match event {
            winit::event::Event::MainEventsCleared => {
                window.request_redraw();
            }

            winit::event::Event::WindowEvent {
                event: winit::event::WindowEvent::CloseRequested,
                ..
            } => *control_flow = winit::event_loop::ControlFlow::Exit,

            winit::event::Event::RedrawRequested(_) => {
                let frame = swap_chain
                    .get_next_texture()
                    .expect("Timed out acquiring next swap chain texture.");

                // -- This is the problematic function  call
                device.create_buffer_with_data(&[0 as u8], wgpu::BufferUsage::VERTEX);

                let mut encoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

                {
                    let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                            attachment: &frame.view,
                            resolve_target: None,
                            load_op: wgpu::LoadOp::Clear,
                            store_op: wgpu::StoreOp::Store,
                            clear_color: wgpu::Color::WHITE,
                        }],
                        depth_stencil_attachment: None,
                    });
                }

                let section = wgpu_glyph::Section {
                    text: "hello",
                    screen_position: (10.0, 10.0),
                    ..wgpu_glyph::Section::default()
                };

                inconsolata.queue(section);

                inconsolata
                    .draw_queued(&device, &mut encoder, &frame.view, width, height)
                    .unwrap();

                queue.submit(&[encoder.finish()]);
            }

            _ => {}
        }
    });
}

fn load_font<'a>(
    path: &str,
    device: &wgpu::Device,
    render_format: wgpu::TextureFormat,
) -> wgpu_glyph::GlyphBrush<'a, (), twox_hash::RandomXxHashBuilder64> {
    let mut file = std::fs::File::open(path).expect(&format!("Failed to open {:?}", path));
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();

    wgpu_glyph::GlyphBrushBuilder::using_font_bytes(buffer)
        .expect(&format!("Failed to generate glyph brush for {:?}", path))
        .cache_glyph_positioning(false)
        .build(device, render_format)
}
