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

pub mod animation;
pub mod camera;
pub mod config;
pub mod player;
pub mod rect;
pub mod util;
pub mod vertex;
pub mod world;

use crate::rect::{INDICES, RectUniform, VERTICES};
use crate::util::lerp;
use crate::vertex::Vertex;
use crate::world::World;

use std::sync::Arc;
use std::time::{Duration, Instant};

use wgpu::util::DeviceExt;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::PhysicalKey;
use winit::window::{Window, WindowId};

// ---------------------------------------------------------------------------
// Fixed-timestep simulation clock — Glenn Fiedler, "Fix Your Timestep!".
//   https://gafferongames.com/post/fix_your_timestep/
//
// A platformer's jump arcs, gravity, and collision response are mathematically
// sensitive to `dt`. If we feed the variable wall-clock frame time directly
// into the update step, behavior changes with the player's hardware: springs
// explode, bullets tunnel through walls, jumps land differently at 60 vs 144 Hz.
//
// Solution: render at whatever rate the display gives us; simulate at a fixed
// rate. The `Clock` accumulates wall-clock time and the inner while-loop
// drains it in `FIXED_DT` chunks. Whatever's left (< FIXED_DT) becomes
// `alpha` — the fraction of a tick the render is "ahead" of the most recent
// simulation state. We'll pass `alpha` to the renderer later to interpolate
// between previous and current sim states for sub-tick visual smoothness.
//
// This is "stage 4" in Fiedler's article — the canonical answer. Adopting it
// from day one is barely more code than the naive variable-dt loop and we
// never have to rewrite the loop later.
// ---------------------------------------------------------------------------

/// Simulation tick rate: 60 Hz (≈ 16.667 ms per step). Conventional and
/// matches typical monitor refresh, so the accumulator stays small.
// const FIXED_DT: Duration = Duration::from_nanos(16_666_667);
const FIXED_DT: Duration = Duration::from_nanos(16_666_667);
const TARGET_FPS: f32 = 60.0;

/// Maximum wall-clock time the loop will absorb in one frame. Without
/// this clamp, a debugger pause or OS sleep can push the elapsed time
/// to seconds; the inner while-loop would then step thousands of times
/// trying to "catch up", freezing the app — Fiedler calls this the
/// "spiral of death". 250 ms is the canonical value from the article.
const MAX_FRAME_TIME: Duration = Duration::from_millis(250);

/// Fixed-timestep clock state.
///
/// `last_instant` is `Option` because `Instant::now()` is not `const`,
/// so the struct can't be initialized eagerly with a sane "previous"
/// time. The first `tick()` records the first timestamp and reports
/// zero elapsed for that frame — there's nothing to simulate before
/// the first frame anyway.
#[derive(Default)]
struct Clock {
    last_instant: Option<Instant>,
    accumulator: Duration,
}

/// Result of advancing the clock by one redraw's worth of wall time.
struct Tick {
    /// How many `FIXED_DT` simulation steps the caller should run now.
    /// May be zero (if the redraw fired faster than `FIXED_DT`) or
    /// several (if a frame ran long).
    steps: u32,
    /// Fraction of the next tick already accumulated, in [0.0, 1.0).
    /// Renderers use this to interpolate `prev_state -> curr_state`
    /// for sub-tick smoothness. Unused while we have no simulation.
    alpha: f32,
}

impl Clock {
    fn tick(&mut self) -> Tick {
        let now = Instant::now();
        let frame_time = match self.last_instant {
            Some(prev) => (now - prev).min(MAX_FRAME_TIME),
            None => Duration::ZERO,
        };
        self.last_instant = Some(now);
        self.accumulator += frame_time;

        let mut steps = 0u32;
        while self.accumulator >= FIXED_DT {
            steps += 1;
            self.accumulator -= FIXED_DT;
        }
        let dt_seconds = frame_time.as_secs_f32();
        if dt_seconds > 0.0 {
            // Prevent dividing by zero on the very first frame!
            let fps = 1.0 / dt_seconds;
            println!("FPS: {:.0}", fps); // {:.0} rounds it to a whole number
        }

        Tick {
            steps,
            alpha: self.accumulator.as_secs_f32() / FIXED_DT.as_secs_f32(),
        }
    }
}
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
    num_indices: u32,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    window: Arc<Window>,
    render_pipeline: wgpu::RenderPipeline,

    // Vertex and index buffers for our rectangle geometry
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,

    world: World,
}

impl State {
    /// Initialize the GPU stack against an already-created window.
    ///
    /// `request_adapter` and `request_device` are async because some
    /// backends probe the system asynchronously. We don't want a real
    /// async runtime for a desktop binary, so the caller wraps this in
    /// `pollster::block_on`.
    async fn new(window: Arc<Window>) -> Self {
        let num_indices = INDICES.len() as u32;
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
            .unwrap();

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
            // Method-reference form (clippy `redundant_closure_for_method_calls`):
            // `.find(|f| f.is_srgb())` and `.find(TextureFormat::is_srgb)` do
            // the same thing; the latter avoids constructing a tiny closure.
            .find(wgpu::TextureFormat::is_srgb)
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

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let rect_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Rect Bind Group Layout"),
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
            });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Camera Bind Group Layout"),
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
            });

        let mut world = World::new(
            &device,
            &rect_bind_group_layout,
            &camera_bind_group_layout,
            [config.width as f32, config.height as f32],
        );
        
        world.read_world(&device, &rect_bind_group_layout);

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    Some(&rect_bind_group_layout),
                    Some(&camera_bind_group_layout),
                ],
                immediate_size: 0,
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"), // 1.
                buffers: &[Vertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                // 3.
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    // 4.
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None, // 1.
            multisample: wgpu::MultisampleState {
                count: 1,                         // 2.
                mask: !0,                         // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
            multiview_mask: None, // 5.
            cache: None,          // 6.
        });

        Self {
            num_indices,
            surface,
            device,
            queue,
            config,
            window,
            render_pipeline,

            ///////////////////////////
            vertex_buffer,
            index_buffer,

            ///////////////////////////
            world,
        }
    }

    /// Reconfigure the surface for a new window size.
    ///
    /// Two important behaviors:
    ///   * Zero-size guard. On Windows, minimizing the window fires
    ///     `Resized(0, 0)` — a zero-dimension `configure` call panics
    ///     inside wgpu. We bail early; the next non-zero resize after
    ///     restore re-validates the surface.
    ///   * Reuse `self.config`. We only mutate the dimensions and re-apply,
    ///     keeping the format / present mode / view formats stable.
    /// See: https://sotrh.github.io/learn-wgpu/beginner/tutorial2-surface/
    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 {
            return;
        }
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);
    }

    fn update(&mut self) {
        // Simulation update would go here once we have a `World` and
        // `Player` to update.
        self.world.update(FIXED_DT.as_secs_f32());
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
    fn render(&mut self, alpha: f32) {
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

            // Occluded: window minimized or hidden — drop the frame, the
            //   OS isn't showing it anyway. Sim keeps ticking.
            // Timeout: rare, can't acquire an image fast enough — skip.
            // (Merged because the recovery is identical; clippy's
            //  `match_same_arms` lint flagged the duplicated bodies.)
            wgpu::CurrentSurfaceTexture::Occluded | wgpu::CurrentSurfaceTexture::Timeout => return,
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
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("frame"),
            });

        {
            // Render pass with one color attachment (the swapchain view)
            // and no draw calls — `LoadOp::Clear` writes the clear color
            // into every pixel and `StoreOp::Store` keeps it. When we
            // start drawing sprites, they'll go inside this same scope.
            let mut _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
            _pass.set_pipeline(&self.render_pipeline); // 2.
            _pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            _pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            let player_visual = lerp(
                self.world.player.rect.rect_object.previous_position,
                self.world.player.rect.rect_object.position,
                alpha,
            );

            let camera_pos = [
                player_visual[0] - self.config.width as f32 / 2.0,
                player_visual[1] - self.config.height as f32 / 2.0,
            ];

            let camera_uniform = crate::camera::CameraUniform::new(camera_pos);

            self.queue.write_buffer(
                &self.world.camera.render_camera.uniform_buffer,
                0,
                bytemuck::bytes_of(&camera_uniform),
            );

            _pass.set_bind_group(1, &self.world.camera.render_camera.bind_group, &[]);

            for item in self.world.items.iter() {
                let visual_pos = lerp(
                    item.rect_object.previous_position,
                    item.rect_object.position,
                    alpha,
                );

                let item_uniform = RectUniform::new(
                    visual_pos,
                    [self.config.width as f32, self.config.height as f32],
                    item.size(),
                    item.color(),
                    item.rotation(),
                );

                self.queue.write_buffer(
                    &item.render_rect.uniform_buffer,
                    0,
                    bytemuck::bytes_of(&item_uniform),
                );

                _pass.set_bind_group(0, &item.render_rect.bind_group, &[]);
                _pass.draw_indexed(0..self.num_indices, 0, 0..1);
            }

            let player_uniform = RectUniform::new(
                player_visual,
                [self.config.width as f32, self.config.height as f32],
                self.world.player.rect.size(),
                self.world.player.rect.color(),
                self.world.player.rect.rotation(),
            );

            self.queue.write_buffer(
                &self.world.player.rect.render_rect.uniform_buffer,
                0,
                bytemuck::bytes_of(&player_uniform),
            );

            _pass.set_bind_group(0, &self.world.player.rect.render_rect.bind_group, &[]);
            _pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
    }
}

/// App now owns a `State` instead of a bare window — the window lives
/// inside `State` so the GPU stack and the surface have a shared owner.
/// `clock` drives the fixed-timestep simulation loop independent of
/// the redraw rate (see `Clock` doc above for the algorithm).
#[derive(Default)]
struct App {
    state: Option<State>,
    clock: Clock,
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
        let Some(state) = self.state.as_mut() else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            // Fires on every drag-corner resize, DPI change, and on the
            // initial show. Without this handler, the swapchain stays
            // sized to the window's first dimensions and presents
            // garbage / crashes after the first resize.
            WindowEvent::KeyboardInput { event, .. } => {
                // Handle keyboard input here, e.g. by updating the player's velocity
                // physical keyboard key
                match event.physical_key {
                    PhysicalKey::Code(code) => {
                        state.world.player.handle_key(code, event);
                    }

                    _ => {}
                }
            }

            WindowEvent::Resized(new_size) => state.resize(new_size),
            WindowEvent::RedrawRequested => {
                let frame_start = Instant::now();
                // Advance the fixed-timestep clock first, BEFORE render.
                // `tick.steps` says how many simulation updates to run
                // this frame (0..N); `tick.alpha` is the render
                // interpolation factor. With no sim state yet the loop
                // body is empty — when we add a `World`, this is where
                //   world.update(FIXED_DT)
                // goes. See Glenn Fiedler, "Fix Your Timestep!".
                let tick = self.clock.tick();

                for _ in 0..tick.steps {
                    state.update();
                    // Simulation step would happen here.
                }
                // `_alpha` will be passed to `render` once we have
                // prev/curr simulation states to interpolate between.
                let alpha = tick.alpha;
                // println!("{:?} {}", {}, _alpha);
                // All surface error recovery lives inside `render`
                // (wgpu 29 collapsed the old SurfaceError into the
                // CurrentSurfaceTexture enum).

                state.render(alpha);

                let target_duration = Duration::from_secs_f32(1.0 / TARGET_FPS);
                let time_spent = frame_start.elapsed();
                if time_spent < target_duration {
                    std::thread::sleep(target_duration - time_spent);
                }
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
