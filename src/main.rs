#![feature(concat_idents)]
#![allow(dead_code)]

use bgfx::*;
use bgfx_rs::bgfx;
use glfw::{Action, Key, Window};
use imgui::Context;
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use crate::imgui_glfw_support::GlfwPlatform;

mod imgui_bgfx_renderer;
mod imgui_glfw_support;

const DEFAULT_WIDTH: u32 = 1920;
const DEFAULT_HEIGHT: u32 = 1080;

#[cfg(target_os = "linux")]
fn get_render_type() -> RendererType {
    RendererType::OpenGL
}

#[cfg(not(target_os = "linux"))]
fn get_render_type() -> RendererType {
    RendererType::Count
}

fn update_platform_handle(pd: &mut PlatformData, window: &Window) {
    match window.raw_window_handle() {
        #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
        ))]
        RawWindowHandle::Xlib(data) => {
            pd.nwh = data.window as *mut _;
            pd.ndt = data.display as *mut _;
        }
        #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
        ))]
        RawWindowHandle::Wayland(data) => {
            pd.ndt = data.surface; // same as window, on wayland there ins't a concept of windows
            pd.nwh = data.display;
        }
        #[cfg(target_os = "macos")]
        RawWindowHandle::MacOS(data) => {
            pd.nwh = data.ns_window;
        }
        #[cfg(target_os = "windows")]
        RawWindowHandle::Windows(data) => {
            pd.nwh = data.hwnd;
        }
        #[cfg(target_os = "android")]
        RawWindowHandle::Android(data) => {
            pd.nwh = data.hwnd;
        }
        _ => panic!("Unsupported Window Manager"),
    }
}

fn init_bgfx(window: &Window){
    let mut pd = bgfx::PlatformData::new();
    update_platform_handle(&mut pd, &window);
    bgfx::set_platform_data(&pd);
    let mut init = Init::new();
    init.type_r = get_render_type();
    init.resolution.width = DEFAULT_WIDTH as u32;
    init.resolution.height = DEFAULT_HEIGHT as u32;
    init.resolution.reset = ResetFlags::VSYNC.bits();
    init.platform_data = pd;
    if !bgfx::init(&init) {
        panic!("failed to init bgfx");
    }
}

fn load_fonts(imgui_context : &mut Context){
    let mut io = imgui_context.io_mut();
    io.font_global_scale = 1.0f32;
    let mut fonts = imgui_context.fonts();
    let font_size = 13.0 * 2.;
    fonts.add_font(&[
        imgui::FontSource::TtfData {
            data: include_bytes!("../Roboto-Regular.ttf"),
            size_pixels: font_size,
            config: Some(imgui::FontConfig {
                // As imgui-glium-renderer isn't gamma-correct with
                // it's font rendering, we apply an arbitrary
                // multiplier to make the font a bit "heavier". With
                // default imgui-glow-renderer this is unnecessary.
                rasterizer_multiply: 1.5,
                // Oversampling font helps improve text rendering at
                // expense of larger font atlas texture.
                oversample_h: 4,
                oversample_v: 4,
                ..imgui::FontConfig::default()
            }),
        },
        imgui::FontSource::TtfData {
            data: include_bytes!("../mplus-1p-regular.ttf"),
            size_pixels: font_size,
            config: Some(imgui::FontConfig {
                // Oversampling font helps improve text rendering at
                // expense of larger font atlas texture.
                oversample_h: 4,
                oversample_v: 4,
                // Range of glyphs to rasterize
                glyph_ranges: imgui::FontGlyphRanges::japanese(),
                ..imgui::FontConfig::default()
            }),
        },
    ]);
}

fn toggle_fullscreen(glfw : &mut glfw::Glfw, window : &mut glfw::Window){
    let mut is_fullscreen = false;
    window.with_window_mode_mut(|mode| {
        match mode {
            glfw::WindowMode::Windowed => { is_fullscreen = false; },
            glfw::WindowMode::FullScreen(_m) => { is_fullscreen = true; },
        }
    });
    match is_fullscreen {
        false => {
            glfw.with_primary_monitor_mut(|_: &mut _, m: Option<&glfw::Monitor>| {
                let monitor = m.unwrap();
                let mode: glfw::VidMode = monitor.get_video_mode().unwrap();
                window.set_monitor(glfw::WindowMode::FullScreen(&monitor), 0, 0, mode.width, mode.height, Some(mode.refresh_rate));
            });
        },
        true => {
            glfw.with_primary_monitor_mut(|_: &mut _, m: Option<&glfw::Monitor>| {
                let monitor = m.unwrap();
                let mode: glfw::VidMode = monitor.get_video_mode().unwrap();
                window.set_monitor(glfw::WindowMode::Windowed, 0, 0, DEFAULT_WIDTH as u32, DEFAULT_HEIGHT as u32, Some(mode.refresh_rate));
            });
        }
    }
}

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));
    let (mut window, events) = glfw
        .create_window(
            DEFAULT_WIDTH as _,
            DEFAULT_HEIGHT as _,
            "App - ESC to close",
            glfw::WindowMode::Windowed,
        )
        .expect("Failed to create GLFW window.");
    window.set_key_polling(true);
    window.set_scroll_polling(true);
    window.set_char_polling(true);

    init_bgfx(&window);
    {
        let mut imgui_context = imgui::Context::create();
        load_fonts(&mut imgui_context);
        let mut glfw_platform = GlfwPlatform::init(&mut imgui_context);
        let mut imgui_renderer = imgui_bgfx_renderer::Renderer::init(&mut imgui_context);
        bgfx::set_debug(DebugFlags::TEXT.bits());
        let mut old_size = (0, 0);
        {
            while !window.should_close() {
                glfw.poll_events();
                glfw_platform.reset();
                for (_, event) in glfw::flush_messages(&events) {
                    glfw_platform.handle_event(&mut imgui_context, &event);
                    if let glfw::WindowEvent::Key(key, _, action, _) = event {
                        if action == Action::Press {
                            if key == Key::Escape {
                                window.set_should_close(true)
                            }else if imgui_context.io().key_alt && key == Key::Enter {
                                toggle_fullscreen(&mut glfw, &mut window);
                            }
                        }
                    }
                }
                let size = window.get_framebuffer_size();
                if old_size != size {
                    //bgfx::reset(size.0 as _, size.1 as _, ResetArgs::default());
                    bgfx::reset(size.0 as _, size.1 as _, ResetArgs{ flags: ResetFlags::VSYNC.bits(), format: TextureFormat::Count });
                    old_size = size;
                }
                bgfx::set_view_clear(
                    0,
                    ClearFlags::COLOR.bits() | ClearFlags::DEPTH.bits(),
                    SetViewClearArgs {
                        rgba: 0x103030ff,
                        ..Default::default()
                    },
                );
                for view_id in 0..2 {
                    bgfx::set_view_rect(view_id, 0, 0, size.0 as _, size.1 as _);
                }
                bgfx::touch(0);
                let (mouse_x, mouse_y) = window.get_cursor_pos();
                {
                    imgui_renderer.begin_frame(&mut imgui_context, [mouse_x as f32, mouse_y as f32], GlfwPlatform::translate_glfw_mouse_buttons_for_imgui(&window), glfw_platform.get_mouse_wheel(), size, glfw_platform.get_last_character(), 0xFF);

                    let mut test: bool = true;
                    let ui = imgui_context.frame();
                    ui.show_demo_window(&mut test);

                    imgui_renderer.render(ui.render());
                }
                bgfx::frame(false);
            }
        }
    }
    bgfx::shutdown();
}