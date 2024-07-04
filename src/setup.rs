use futures::{Future, StreamExt};
use glutin::context::{NotCurrentGlContext, PossiblyCurrentGlContext};
use imgui::{BackendFlags, ClipboardBackend, Condition, FontConfig, FontSource, StyleColor, TableFlags, Ui};
use copypasta::{ClipboardContext, ClipboardProvider};
use winit::event::Event;
use std::path::Path;
use std::sync::mpsc::{channel, Sender};
use std::time::Instant;
use std::{ffi::CString, num::NonZeroU32};

use glow::{Context, HasContext};
use glutin::{
    config::ConfigTemplateBuilder,
    context::ContextAttributesBuilder,
    display::GetGlDisplay,
    prelude::GlDisplay,
    surface::{GlSurface, SurfaceAttributesBuilder, WindowSurface},
};
use glutin_winit::DisplayBuilder;
use imgui::ConfigFlags;
use imgui_winit_glow_renderer_viewports::Renderer;
use raw_window_handle::HasRawWindowHandle;
use winit::{dpi::LogicalSize, event::WindowEvent, event_loop::EventLoop, window::WindowBuilder};
use revolt_database::events::client::EventV1;

use crate::state::GlobalState;
use crate::websocket;


pub const FONT_SIZE: f32 = 13.0;

pub struct ClipboardSupport(pub ClipboardContext);

pub fn init_clipboard() -> Option<ClipboardSupport> {
    ClipboardContext::new().ok().map(ClipboardSupport)
}

impl ClipboardBackend for ClipboardSupport {
    fn get(&mut self) -> Option<String> {
        self.0.get_contents().ok()
    }
    fn set(&mut self, text: &str) {
        // ignore errors?
        let _ = self.0.set_contents(text.to_owned());
    }
}

pub fn init<S, E, FState, FBg, FBgF, FUpdate, FUi>(
    title: &str,
    create_state: FState,
    background_task: FBg,
    update_state: FUpdate,
    mut run_ui: FUi
) where
    FState: FnOnce() -> S,
    FBg: FnOnce(Sender<E>) -> FBgF + Send + Sync + 'static,
    FBgF: Future + Send + Sync + 'static,
    <FBgF as Future>::Output: Send + Sync + 'static,
    FUpdate: Fn(E, &mut S) -> (),
    FUi: FnMut(&mut bool, &mut Ui, &mut S) + 'static,
{
    let mut imgui = create_context();

    let title = match Path::new(&title).file_name() {
        Some(file_name) => file_name.to_str().unwrap(),
        None => title,
    };
    let event_loop = EventLoop::new().unwrap();

    let window_builder = WindowBuilder::new()
        .with_title(title)
        .with_inner_size(LogicalSize::new(1024, 768));

    let template_builder = ConfigTemplateBuilder::new();

    let (window, gl_config) = DisplayBuilder::new()
        .with_window_builder(Some(window_builder))
        .build(&event_loop, template_builder, |mut configs| {
            configs.next().unwrap()
        })
        .unwrap();

    let window = window.unwrap();

    let context_attribs = ContextAttributesBuilder::new().build(Some(window.raw_window_handle()));
    let context = unsafe {
        gl_config
            .display()
            .create_context(&gl_config, &context_attribs)
            .expect("Failed to create main context")
    };

    let size = window.inner_size();
    let surface_attribs = SurfaceAttributesBuilder::<WindowSurface>::new().build(
        window.raw_window_handle(),
        NonZeroU32::new(size.width).unwrap(),
        NonZeroU32::new(size.height).unwrap(),
    );
    let surface = unsafe {
        gl_config
            .display()
            .create_window_surface(&gl_config, &surface_attribs)
            .expect("Failed to create main surface")
    };

    let context = context
        .make_current(&surface)
        .expect("Failed to make current");

    let glow = unsafe {
        Context::from_loader_function(|name| {
            let name = CString::new(name).unwrap();
            context.display().get_proc_address(&name)
        })
    };

    if let Some(backend) = init_clipboard() {
        imgui.set_clipboard_backend(backend);
    } else {
        eprintln!("Failed to initialize clipboard");
    }

    let (event_sender, event_receiver) = channel::<E>();

    tokio::spawn(background_task(event_sender));

    let mut renderer = Renderer::new(&mut imgui, &window, &glow).unwrap();
    let mut last_frame = Instant::now();
    let mut state = create_state();

    event_loop
        .run(move |event, window_target| {
            window_target.set_control_flow(winit::event_loop::ControlFlow::Poll);
            renderer.handle_event(&mut imgui, &window, &event);

            while let Some(user_event) = event_receiver.try_iter().next() {
                update_state(user_event, &mut state);
            }

            match event {
                Event::NewEvents(_) => {
                    let now = Instant::now();
                    imgui.io_mut().update_delta_time(now - last_frame);
                    last_frame = now;
                }
                Event::AboutToWait => {
                    window.request_redraw();
                }
                Event::WindowEvent {
                    event: WindowEvent::RedrawRequested,
                    ..
                } => {
                    let ui = imgui.frame();

                    ui.dockspace_over_main_viewport();

                    let mut run = true;
                    run_ui(&mut run, ui, &mut state);
                    if !run {
                        window_target.exit();
                    }

                    ui.end_frame_early();

                    renderer.prepare_render(&mut imgui, &window);

                    imgui.update_platform_windows();
                    renderer
                        .update_viewports(&mut imgui, window_target, &glow)
                        .expect("Failed to update viewports");

                    let draw_data = imgui.render();

                    if let Err(e) = context.make_current(&surface) {
                        eprintln!("Failed to make current: {e}");
                    }

                    unsafe {
                        glow.disable(glow::SCISSOR_TEST);
                        glow.clear(glow::COLOR_BUFFER_BIT);
                    }

                    renderer
                        .render(&window, &glow, draw_data)
                        .expect("Failed to render main viewport");

                    surface
                        .swap_buffers(&context)
                        .expect("Failed to swap buffers");

                    renderer
                        .render_viewports(&glow, &mut imgui)
                        .expect("Failed to render viewports");

                }
                Event::WindowEvent {
                    window_id,
                    event: WindowEvent::Resized(new_size),
                    ..
                } if window_id == window.id() => {
                    surface.resize(
                        &context,
                        NonZeroU32::new(new_size.width).unwrap(),
                        NonZeroU32::new(new_size.height).unwrap(),
                    );
            }
                Event::WindowEvent {
                    window_id,
                    event: WindowEvent::CloseRequested,
                    ..
                } if window_id == window.id() => {
                    window_target.exit()
                }
                _ => {}
            }
        })
        .expect("EventLoop error");
}

/// Creates the imgui context
pub fn create_context() -> imgui::Context {
    let mut imgui = imgui::Context::create();
    imgui.io_mut().config_flags |= ConfigFlags::VIEWPORTS_ENABLE;
    imgui.io_mut().config_flags |= ConfigFlags::DOCKING_ENABLE;
    imgui.io_mut().backend_flags |= BackendFlags::PLATFORM_HAS_VIEWPORTS;

    // imgui.fonts().add_font(&[
    //     FontSource::TtfData {
    //         data: include_bytes!("../resources/InterVariable.ttf"),
    //         size_pixels: FONT_SIZE,
    //         config: Some(FontConfig {
    //             // As imgui-glium-renderer isn't gamma-correct with
    //             // it's font rendering, we apply an arbitrary
    //             // multiplier to make the font a bit "heavier". With
    //             // default imgui-glow-renderer this is unnecessary.
    //             rasterizer_multiply: 1.5,
    //             // Oversampling font helps improve text rendering at
    //             // expense of larger font atlas texture.
    //             oversample_h: 4,
    //             oversample_v: 4,
    //             ..FontConfig::default()
    //         }),
    //     }
    // ]);
    imgui.set_ini_filename(Some(Path::new("Revolt.ini").to_path_buf()));

    imgui
}