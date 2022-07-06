//! Renderer for [`imgui-rs`][imgui] using the [`bgfx`] library
//!
//! This is heavily influenced by the
//! [example from upstream](https://github.com/bkaradzic/bgfx/blob/master/examples/common/imgui/imgui.cpp).
//!
use std::time::Instant;
use imgui::{internal::RawWrapper, DrawData};
use bgfx_rs::bgfx;
use bgfx::*;

mod vs_ocornut_imgui;
mod fs_ocornut_imgui;

use vs_ocornut_imgui::*;
use fs_ocornut_imgui::*;

macro_rules! get_shader_code {
    ($name:ident) => {
        match bgfx::get_renderer_type() {
            RendererType::Direct3D9 => &concat_idents!($name, _DX9),
            RendererType::Direct3D11 => &concat_idents!($name, _DX11),
            RendererType::OpenGL => &concat_idents!($name, _GLSL),
            RendererType::Metal => &concat_idents!($name, _MTL),
            RendererType::OpenGLES => &concat_idents!($name, _ESSL),
            RendererType::Vulkan => &concat_idents!($name, _SPV),
            e => panic!("Unsupported renderer type {:#?}", e),
        }
    };
}
/*macro_rules! include_shader_code {
    ($name:ident) => {
        pub const $name = include_bytes!(stringify!($name));
    };
}*/
pub struct Renderer{
    shader_program : bgfx::Program,
    vertex_layout : bgfx::VertexLayoutBuilder,
    font_atlas : bgfx::Texture,
    sampler_uniform : bgfx::Uniform,
    view_id : bgfx::ViewId,
    frame_time : Instant
}

impl Renderer {
    pub fn init(imgui_context: &mut imgui::Context) -> Self {
        imgui_context.set_ini_filename(None);
        let mut io = imgui_context.io_mut();
        io.backend_flags = imgui::BackendFlags::RENDERER_HAS_VTX_OFFSET;
        let texture = {
            let mut fonts = imgui_context.fonts();
            let font_atlas = fonts.build_rgba32_texture();
            bgfx::create_texture_2d(font_atlas.width as u16, font_atlas.height as u16, false, 1, bgfx::TextureFormat::BGRA8, 0, &Memory::copy(&font_atlas.data))
        };
        Self {
            shader_program: {
                let vsh = bgfx::create_shader(&Memory::copy(get_shader_code!(VS_OCORNUT_IMGUI)));
                let fsh = bgfx::create_shader(&Memory::copy(get_shader_code!(FS_OCORNUT_IMGUI)));
                //let vsh = bgfx::create_shader(&Memory::copy(&vs_ocornut_imgui::vs_ocornut_imgui_glsl));
                //let fsh = bgfx::create_shader(&Memory::copy(&fs_ocornut_imgui::fs_ocornut_imgui_glsl));
                bgfx::create_program(&vsh, &fsh, false)
                //bgfx::create_program(&vsh, &fsh, true) //TODO: Why Segmentation fault if we destroy the shaders?
            },
            vertex_layout: {
                let layout = bgfx::VertexLayoutBuilder::new();
                layout.begin(bgfx::RendererType::Noop);
                layout.add(bgfx::Attrib::Position, 2, bgfx::AttribType::Float, AddArgs{ normalized: true, as_int: false });
                layout.add(bgfx::Attrib::TexCoord0, 2, bgfx::AttribType::Float, AddArgs{ normalized: true, as_int: false });
                layout.add(bgfx::Attrib::Color0, 4, bgfx::AttribType::Uint8, AddArgs{ normalized: true, as_int: true });
                layout.end();
                layout
            },
            sampler_uniform: {
                bgfx::Uniform::create("s_tex", bgfx::UniformType::Sampler, 1)
            },
            font_atlas: texture,
            view_id: 0xFF,
            frame_time: Instant::now()
        }
    }
    pub fn begin_frame(&mut self, imgui_context: &mut imgui::Context, mouse_position : [f32; 2], mouse_button : u8, mouse_scroll : f32, size : (i32, i32), input_char : char, view_id : bgfx::ViewId)  {
        self.view_id = view_id;
        let mut io = imgui_context.io_mut();
        io.mouse_pos = mouse_position;
        for i in 0..5 {
            io.mouse_down[i] = (mouse_button & (0x01 << i)) != 0x00;
        }
        io.mouse_wheel = mouse_scroll;
        io.display_size = [size.0 as f32, size.1 as f32];
        if input_char != '\0' {
            io.add_input_character(input_char);
        }
        io.delta_time = self.frame_time.elapsed().as_secs_f32();
        self.frame_time = Instant::now();
    }

    pub fn render(&mut self, draw_data: &DrawData) {
        //let view_id : bgfx::ViewId = 255;
        let index_32 = std::mem::size_of::<imgui::DrawIdx>() == 4;
        let fb_width = draw_data.display_size[0] * draw_data.framebuffer_scale[0];
        let fb_height = draw_data.display_size[1] * draw_data.framebuffer_scale[1];
        if fb_width <= 0.0 || fb_height <= 0.0 {
            return;
        }
        bgfx::set_view_mode(self.view_id, bgfx::ViewMode::Sequential);
        //let caps = bgfx::get_caps();
        {
            let x = draw_data.display_pos[0];
            let y = draw_data.display_pos[1];
            let width = draw_data.display_size[0];
            let height = draw_data.display_size[1];
            let projection = glam::Mat4::orthographic_lh(x, x + width, y + height, y, 0.0f32, 1000.0f32);
            bgfx::set_view_transform(self.view_id, &glam::Mat4::IDENTITY.as_ref(), &projection.as_ref());
            bgfx::set_view_rect(self.view_id, 0, 0, width as u16, height as u16);
        }

        let clip_pos   = draw_data.display_pos;       // (0,0) unless using multi-viewports
        let clip_scale = draw_data.framebuffer_scale; // (1,1) unless using retina display which are often (2,2)

        for draw_list in draw_data.draw_lists() {
            let vertices_count=  draw_list.vtx_buffer().len() as u32;
            let indices_count = draw_list.idx_buffer().len() as u32;
            if bgfx::get_avail_transient_vertex_buffer(vertices_count, &self.vertex_layout) != vertices_count || bgfx::get_avail_transient_index_buffer(indices_count, index_32) != indices_count {
                break;
            }
            let mut tvb = bgfx::TransientVertexBuffer::new();
            let mut tib = bgfx::TransientIndexBuffer::new();

            bgfx::alloc_transient_vertex_buffer(&mut tvb, vertices_count, &self.vertex_layout);
            bgfx::alloc_transient_index_buffer(&mut tib, indices_count, index_32);

            unsafe {  std::ptr::copy_nonoverlapping(draw_list.vtx_buffer().as_ptr() as *const u8, tvb.data as *mut u8, std::mem::size_of::<imgui::DrawVert>() * vertices_count as usize); }
            unsafe {  std::ptr::copy_nonoverlapping(draw_list.idx_buffer().as_ptr() as *const u8, tib.data as *mut u8, std::mem::size_of::<imgui::DrawIdx>() * indices_count as usize); }

            let encoder = bgfx::encoder_begin(false);
            for command in draw_list.commands() {
                match command {
                    imgui::DrawCmd::Elements { count, cmd_params } => {
                        let state = StateWriteFlags::RGB.bits() | StateWriteFlags::A.bits() | StateFlags::MSAA.bits() | StateBlendFlags::SRC_ALPHA.bits() | (StateBlendFlags::INV_SRC_ALPHA.bits() << 4) | (StateBlendFlags::SRC_ALPHA.bits() << 8) | (StateBlendFlags::INV_SRC_ALPHA.bits() << 12);
                        let clip_rect = [
                            (cmd_params.clip_rect[0] - clip_pos[0]) * clip_scale[0],
                            (cmd_params.clip_rect[1] - clip_pos[1]) * clip_scale[1],
                            (cmd_params.clip_rect[2] - clip_pos[0]) * clip_scale[0],
                            (cmd_params.clip_rect[3] - clip_pos[1]) * clip_scale[1]
                        ];
                        if clip_rect[0] <  fb_width && clip_rect[1] < fb_height &&  clip_rect[2] >= 0.0f32 &&  clip_rect[3] >= 0.0f32 {
                            let xx = clip_rect[0].max(0.0f32) as u16;
                            let yy = clip_rect[1].max(0.0f32) as u16;
                            encoder.set_scissor(xx, yy, (clip_rect[2].min(f32::MAX) as u16) -xx, (clip_rect[3].min(f32::MAX) as u16) - yy);
                            encoder.set_state(state, 0);
                            encoder.set_texture(0, &self.sampler_uniform, &self.font_atlas, u32::MAX);
                            encoder.set_transient_vertex_buffer(0, &tvb, cmd_params.vtx_offset as u32, vertices_count);
                            encoder.set_transient_index_buffer(&tib, cmd_params.idx_offset as u32, count as u32);
                            encoder.submit(self.view_id, &self.shader_program, SubmitArgs::default());
                        }
                    },
                    imgui::DrawCmd::RawCallback { callback, raw_cmd } => unsafe {
                        callback(draw_list.raw(), raw_cmd);
                    },
                    imgui::DrawCmd::ResetRenderState => {
                        bgfx::reset(fb_width as u32, fb_height as u32, ResetArgs::default());
                    }
                }
            }
            bgfx::encoder_end(&encoder);
        }
    }
    pub fn get_shader(&self) -> &bgfx::Program {
        &self.shader_program
    }
}
