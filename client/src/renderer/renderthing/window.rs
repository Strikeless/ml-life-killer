use std::{sync::Arc, time::Duration};

use pixels::{wgpu::TextureFormat, Pixels, PixelsBuilder, SurfaceTexture};
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalSize},
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowAttributes},
};

use super::{frame::RenderFrame, sleeper::Sleeper};

pub(super) struct RendererWindow {
    config: RendererWindowConfig,
    resumed_window: Option<ResumedWindow>,
    sleeper: Sleeper,
}

impl RendererWindow {
    pub fn new(config: RendererWindowConfig) -> Self {
        let sleeper = {
            let target_frame_time = Duration::from_micros(1_000_000 / config.target_fps);
            Sleeper::new(target_frame_time)
        };

        Self {
            config,
            resumed_window: None,
            sleeper,
        }
    }
}

pub struct RendererWindowConfig {
    pub title: String,
    pub width: usize,
    pub height: usize,
    pub target_fps: u64,
    pub draw_callback: Box<dyn FnMut(RenderFrame)>,
    pub event_callback: Option<Box<dyn FnMut(&WindowEvent)>>,
}

struct ResumedWindow {
    window: Arc<Window>,
    pixels: Pixels<'static>,
}

impl ApplicationHandler for RendererWindow {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new({
            let window_size = LogicalSize::new(self.config.width as f64, self.config.height as f64);

            event_loop
                .create_window(
                    WindowAttributes::default()
                        .with_title(self.config.title.clone())
                        .with_inner_size(window_size),
                )
                .expect("Creating window")
        });

        let pixels = {
            let window_size = window.inner_size();

            let surface_texture =
                SurfaceTexture::new(window_size.width, window_size.height, window.clone());

            PixelsBuilder::new(window_size.width, window_size.height, surface_texture)
                .texture_format(TextureFormat::Rgba8UnormSrgb)
                .build()
                .expect("Creating pixels buffer")
        };

        window.request_redraw();

        self.resumed_window = Some(ResumedWindow { window, pixels });
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        // SAFETY: I don't think winit will ever call window_event before resumed.
        let ResumedWindow { window, pixels } = self.resumed_window.as_mut().unwrap();

        match event {
            WindowEvent::RedrawRequested => {
                let PhysicalSize { width, height } = window.inner_size();

                let next_frame = RenderFrame {
                    width,
                    height,
                    buffer: pixels.frame_mut(),
                };

                (self.config.draw_callback)(next_frame);

                // Let pixels do the actual hard work
                pixels.render().expect("Rendering with pixels");

                // FIXME: It isn't ideal that we're hanging the entire event loop just for the throttled redraw loop.
                //        This can make things such as resizing less responsive, since we're waiting here instead of handling the resize.
                self.sleeper.sleep();
                window.request_redraw();
            }
            WindowEvent::Resized(PhysicalSize { width, height }) => {
                pixels.resize_surface(width, height).unwrap();
                pixels.resize_buffer(width, height).unwrap();
                window.request_redraw();
            }
            WindowEvent::CloseRequested => event_loop.exit(),
            _ => {}
        }

        if let Some(event_callback) = &mut self.config.event_callback {
            event_callback(&event);
        }
    }
}
