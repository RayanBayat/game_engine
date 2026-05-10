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
//!     https://sotrh.github.io/learn-wgpu/beginner/tutorial2-surface/
//!   - winit `ApplicationHandler` trait — modern entry-point shape
//!     https://docs.rs/winit/0.30.13/winit/application/trait.ApplicationHandler.html

use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

/// GPU + swapchain bundle.
///
/// Holds everything we need to issue a frame:
///   * `surface`  — the OS-window-backed swapchain wgpu draws into.
///   * `device`   — creates GPU resources (textures, buffers, pipelines)
///                  and records command encoders.
///   * `queue`    — submits recorded command buffers to the GPU.
///   * `config`   — the surface's current size / format / present mode;
///                  reapplied with `surface.configure(...)` on resize.
///   * `window`   — `Arc` so the surface can outlive any single owner;
///                  using `Arc<Window>` is what gives us `Surface<'static>`.
///
/// See: https://sotrh.github.io/learn-wgpu/beginner/tutorial2-surface/
struct State {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    window: Arc<Window>,
}

impl State {
    /// Initialize the GPU stack against an already-created window.
    ///
    /// `request_adapter` and `request_device` are async because some
    /// backends probe the system asynchronously. We don't want a real
    /// async runtime for a desktop binary, so the caller wraps this in
    /// `pollster::block_on`.
    async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();

        // The Instance is the wgpu entry point; defaults pick the
        // platform's preferred backend (DX12 on Windows, Metal on macOS,
        // Vulkan elsewhere).
        let instance = wgpu::Instance::default();

        // Surface = the swapchain target the OS will composite to screen.
        // `Arc<Window>` -> `Surface<'static>` (no lifetime juggling).
        let surface = instance
            .create_surface(window.clone())
            .expect("failed to create surface");

        // Adapter = chosen physical GPU + backend pairing. We pass the
        // surface so wgpu picks an adapter that can actually present to
        // it (avoids picking an iGPU when the surface is on the dGPU).
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("no compatible GPU adapter found");

        // Device + Queue. Default descriptor: no extra features, no
        // raised limits — we don't need them for clear-only and asking
        // for less means more machines can run the engine.
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .expect("failed to acquire device");

        // Surface format selection. Most platforms expose both linear
        // and sRGB formats; we prefer sRGB so colors painted in 0..1
        // match what designers see in tools that assume sRGB output.
        // On Windows the first format is typically `Bgra8UnormSrgb`.
        // Hardcoding `Rgba8UnormSrgb` like some older tutorials do can
        // wash out colors on platforms that don't list it first.
        //   https://sotrh.github.io/learn-wgpu/beginner/tutorial2-surface/
        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            // `.max(1)` — wgpu rejects a zero-dimension surface; cheap
            // belt-and-braces guard against weird startup states.
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: caps.present_modes[0],
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            // 2 frames in flight is the typical sweet spot for desktop:
            // enough to keep the GPU busy without adding visible input
            // lag. Increase only if profiling shows GPU stalls.
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        Self { surface, device, queue, config, window }
    }

    /// Draw a single frame. For now: clear the swapchain image to a dark
    /// color and present. This is the GPU "hello world" — once it shows
    /// a colored window, the entire winit↔wgpu seam is verified end to
    /// end and we can build sprites/tilemaps on top.
    ///   https://sotrh.github.io/learn-wgpu/beginner/tutorial2-surface/
    ///
    /// In wgpu 29 `get_current_texture` no longer returns a `Result`; it
    /// returns a `CurrentSurfaceTexture` enum we must match on. The old
    /// `wgpu::SurfaceError` type from older tutorials no longer exists.
    /// Recovery (re-configure on Outdated/Lost/Suboptimal) lives inline
    /// here so render() has a single integration point with the surface.
    ///   https://docs.rs/wgpu/29.0.3/wgpu/enum.CurrentSurfaceTexture.html
    fn render(&mut self) {
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(t) => t,
            // Suboptimal: texture is usable, but the surface no longer
            // matches its config (e.g. just-finished resize). Present
            // it AND reconfigure so the next frame is optimal.
            wgpu::CurrentSurfaceTexture::Suboptimal(t) => {
                self.surface.configure(&self.device, &self.config);
                t
            }
            // Outdated / Lost: surface fell out of sync (monitor change,
            // alt-tab, device reset). Reconfigure and skip this frame —
            // the next redraw will paint successfully.
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.surface.configure(&self.device, &self.config);
                return;
            }
            // Window minimized or hidden — drop the frame; the OS isn't
            // showing it anyway. We'll keep ticking (sim still runs).
            wgpu::CurrentSurfaceTexture::Occluded => return,
            // Timeout while acquiring an image (rare). Skip the frame.
            wgpu::CurrentSurfaceTexture::Timeout => return,
            // Validation error already surfaced via the wgpu error scope
            // / uncaptured-error callback; nothing more to do here.
            wgpu::CurrentSurfaceTexture::Validation => {
                eprintln!("surface texture validation error");
                return;
            }
        };

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Encoder records a sequence of GPU commands; nothing executes
        // until `queue.submit(encoder.finish())`.
        let mut encoder =
            self.device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("frame"),
                });

        {
            // Render pass with one color attachment (the swapchain view)
            // and no draw calls — `LoadOp::Clear` writes the clear color
            // into every pixel and `StoreOp::Store` keeps it. When we
            // start drawing sprites, they'll go inside this same scope.
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("clear"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    // `depth_slice: None` — only relevant for 3D texture
                    // attachments; we're rendering to a 2D swapchain.
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.05,
                            g: 0.07,
                            b: 0.10,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                // wgpu 29 added multiview support; `None` = single-view
                // rendering (the only thing a 2D platformer ever needs).
                //   https://docs.rs/wgpu/29.0.3/wgpu/struct.RenderPassDescriptor.html
                multiview_mask: None,
            });
            // _pass is dropped here, ending the render pass. wgpu records
            // an `EndRenderPass` command at this point.
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
    }
}

/// App now owns a `State` instead of a bare window — the window lives
/// inside `State` so the GPU stack and the surface have a shared owner.
#[derive(Default)]
struct App {
    state: Option<State>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }
        let attrs = Window::default_attributes().with_title("game_engine");
        let window = Arc::new(
            event_loop
                .create_window(attrs)
                .expect("failed to create window"),
        );
        // pollster blocks the current thread on the async wgpu init.
        // Two seconds-ish max in practice; happens once at startup.
        let state = pollster::block_on(State::new(window.clone()));
        // Kick the redraw loop. From here, every `RedrawRequested`
        // handler ends with `request_redraw()` to keep frames flowing.
        window.request_redraw();
        self.state = Some(state);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(state) = self.state.as_mut() else { return };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                // All surface error recovery now lives inside `render`
                // (wgpu 29 collapsed the old SurfaceError into the
                // CurrentSurfaceTexture enum). Caller just kicks the
                // next frame.
                state.render();
                // Continuous redraw — see Poll-mode rationale in `main`.
                state.window.request_redraw();
            }
            _ => {}
        }
    }
}

fn main() {
    // wgpu logs internally via the `log` facade; install env_logger so
    // warnings reach the terminal. Run with `RUST_LOG=wgpu=warn,info`
    // to see GPU-side diagnostics.
    //   https://github.com/gfx-rs/wgpu/wiki/Debugging-wgpu-Applications
    env_logger::init();

    let event_loop = EventLoop::new().expect("failed to create event loop");

    // `Poll` keeps the loop spinning; a fixed-timestep accumulator
    // (next commit) controls actual simulation rate. `Wait` would
    // suspend until input — wrong for a game.
    //   https://docs.rs/winit/0.30.13/winit/event_loop/enum.ControlFlow.html
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    event_loop
        .run_app(&mut app)
        .expect("event loop terminated with error");
}
