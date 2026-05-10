//! Minimal game engine — entry point.
//!
//! Project goal: build the smallest engine that can ship a 2D platformer,
//! adding pieces only when a real game forces them. This file is the
//! single-binary scaffold; we'll split into modules when duplication
//! demands it, not before.
//!
//! Source material:
//!   - lisyarus, "So you want to make a game engine?" — philosophy
//!     https://lisyarus.github.io/blog/posts/so-you-want-to-make-a-game-engine.html
//!   - sotrh, learn-wgpu — concrete winit + wgpu walkthroughs
//!     https://sotrh.github.io/learn-wgpu/beginner/tutorial1-window/
//!   - winit `ApplicationHandler` trait — modern entry-point shape
//!     https://docs.rs/winit/0.30.13/winit/application/trait.ApplicationHandler.html

use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

/// Top-level application state owned by the winit event loop.
///
/// `window` starts as `None` because — under winit 0.30+ — the window
/// must be created inside `resumed()`, not before `run_app()`. That
/// matches mobile lifecycles (Android/iOS can suspend and recreate the
/// window) and is now the recommended pattern on desktop too. The old
/// closure-based `EventLoop::run(|event, target| { ... })` form is
/// deprecated; treat any tutorial showing it as outdated.
///   https://docs.rs/winit/0.30.13/winit/application/trait.ApplicationHandler.html
///   https://rust-windowing.github.io/winit/winit/changelog/v0_30/index.html
#[derive(Default)]
struct App {
    /// `Arc` because the window will later be shared with `wgpu::Surface`,
    /// which needs to keep it alive for as long as the surface exists.
    /// Using `Arc<Window>` from day one avoids a refactor when wgpu lands
    /// in the next commit.
    window: Option<Arc<Window>>,
}

impl ApplicationHandler for App {
    /// Called once at startup on desktop, and again after the OS has
    /// recreated the window on mobile suspend/resume cycles.
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Guard against re-entry: if we already have a window we don't
        // want to leak the old one by overwriting it.
        if self.window.is_some() {
            return;
        }
        let attrs = Window::default_attributes().with_title("game_engine");
        let window = event_loop
            .create_window(attrs)
            .expect("failed to create window");
        self.window = Some(Arc::new(window));
    }

    /// All input + lifecycle messages targeted at our window land here.
    /// We currently only care about close requests; a future commit will
    /// add `RedrawRequested` (render hook) and `Resized` (reconfigure GPU
    /// surface).
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            _ => {}
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().expect("failed to create event loop");

    // `Poll` keeps the loop spinning so we can drive a fixed-timestep
    // simulation in a later commit. `Wait` would suspend the loop until
    // the next OS event, which is correct for an editor but wrong for
    // a game where physics must keep ticking.
    //   https://docs.rs/winit/0.30.13/winit/event_loop/enum.ControlFlow.html
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    event_loop
        .run_app(&mut app)
        .expect("event loop terminated with error");
}
