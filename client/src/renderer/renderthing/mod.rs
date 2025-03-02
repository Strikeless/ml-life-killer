use anyhow::Context;
use window::{RendererWindow, RendererWindowConfig};
use winit::event_loop::EventLoop;

pub mod frame;
pub mod sleeper;
pub mod window;

pub struct Renderer {
    event_loop: EventLoop<()>,
    window: RendererWindow,
}

impl Renderer {
    pub fn new(config: RendererWindowConfig) -> anyhow::Result<Self> {
        Ok(Self {
            event_loop: EventLoop::new().context("Creating event loop")?,
            window: RendererWindow::new(config),
        })
    }

    pub fn run(mut self) -> anyhow::Result<()> {
        self.event_loop.run_app(&mut self.window)?;
        Ok(())
    }
}
