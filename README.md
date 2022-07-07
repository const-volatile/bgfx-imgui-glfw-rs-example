# bgfx-imgui-glfw-rs-example
🦀 Example of bgfx-rs, imgui &amp; glfw in Rust

![screenshot](https://user-images.githubusercontent.com/9589896/177610510-586f0329-a105-4b02-adbe-dc22c2d7ad40.png)

### Prerequisites ###
- rust nightly (to use concat_idents)
- cmake (to build dependencies)

### How to build ###
Clone the repo and execute `cargo run --release` in the root directory.

### Shaders ###
In main.rs `#![feature(concat_idents)]` is enabled, which needs rust nightly.
The following macro then resolves to the correct shader, based on the renderer backend:
```
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
```

The macro can be used like this:
```
let vsh = bgfx::create_shader(&Memory::copy(get_shader_code!(VS_OCORNUT_IMGUI)));
let fsh = bgfx::create_shader(&Memory::copy(get_shader_code!(FS_OCORNUT_IMGUI)));
```

Shader code is generated using ![bgfx shaderc tool](https://github.com/bkaradzic/bgfx/tree/master/tools/shaderc) and manually modified to an rust array.
Example usage (Windows batch file):
```
shaderc -i . -f %%f -o %%~nf_glsl.h --bin2c %%~nf_glsl --type %%t --platform linux
shaderc -i . -f %%f -o %%~nf_spv.h --bin2c %%~nf_spv --type %%t --platform linux -p spirv
if %%t==v (
shaderc -i . -f %%f -o %%~nf_dx9.h --bin2c %%~nf_dx9 --type %%t --platform windows -p vs_3_0 -O 3
shaderc -i . -f %%f -o %%~nf_dx11.h --bin2c %%~nf_dx11 --type %%t --platform windows -p vs_4_0 -O 3
) else (
shaderc -i . -f %%f -o %%~nf_dx9.h --bin2c %%~nf_dx9 --type %%t --platform windows -p ps_3_0 -O 3
shaderc -i . -f %%f -o %%~nf_dx11.h --bin2c %%~nf_dx11 --type %%t --platform windows -p ps_4_0 -O 3
)
shaderc -i . -f %%f -o %%~nf_mtl.h --bin2c %%~nf_mtl --type %%t --platform ios -p metal  -O 3
```
