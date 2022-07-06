pub struct GlfwPlatform {
    mouse_wheel : f32,
    last_character : char
}

impl GlfwPlatform {
    pub fn init(imgui_context : &mut imgui::Context) -> GlfwPlatform {
        let mut io = imgui_context.io_mut();
        io.key_map[imgui::Key::Tab as usize] = glfw::Key::Tab as u32;
        io.key_map[imgui::Key::LeftArrow as usize] = glfw::Key::Left as u32;
        io.key_map[imgui::Key::RightArrow as usize] = glfw::Key::Right as u32;
        io.key_map[imgui::Key::UpArrow as usize] = glfw::Key::Up as u32;
        io.key_map[imgui::Key::DownArrow as usize] = glfw::Key::Down as u32;
        io.key_map[imgui::Key::PageUp as usize] = glfw::Key::PageUp as u32;
        io.key_map[imgui::Key::PageDown as usize] = glfw::Key::PageDown as u32;
        io.key_map[imgui::Key::Home as usize] = glfw::Key::Home as u32;
        io.key_map[imgui::Key::End as usize] = glfw::Key::End as u32;
        io.key_map[imgui::Key::Insert as usize] = glfw::Key::Insert as u32;
        io.key_map[imgui::Key::Delete as usize] = glfw::Key::Delete as u32;
        io.key_map[imgui::Key::Backspace as usize] = glfw::Key::Backspace as u32;
        io.key_map[imgui::Key::Space as usize] = glfw::Key::Space as u32;
        io.key_map[imgui::Key::Enter as usize] = glfw::Key::Enter as u32;
        io.key_map[imgui::Key::Escape as usize] = glfw::Key::Escape as u32;
        io.key_map[imgui::Key::KeyPadEnter as usize] = glfw::Key::KpEnter as u32;
        io.key_map[imgui::Key::A as usize] = glfw::Key::A as u32;
        io.key_map[imgui::Key::C as usize] = glfw::Key::C as u32;
        io.key_map[imgui::Key::V as usize] = glfw::Key::V as u32;
        io.key_map[imgui::Key::X as usize] = glfw::Key::X as u32;
        io.key_map[imgui::Key::Y as usize] = glfw::Key::Y as u32;
        io.key_map[imgui::Key::Z as usize] = glfw::Key::Z as u32;
        GlfwPlatform { mouse_wheel: 0.0, last_character: '\0' }
    }
    pub fn translate_glfw_mouse_buttons_for_imgui(glfw_window : &glfw::Window) -> u8 {
        let mut mouse_buttons: u8 = 0;
        if glfw_window.get_mouse_button(glfw::MouseButtonLeft) == glfw::Action::Press { mouse_buttons |= 0x01 << 0; }
        if glfw_window.get_mouse_button(glfw::MouseButtonRight) == glfw::Action::Press { mouse_buttons |= 0x01 << 1; }
        if glfw_window.get_mouse_button(glfw::MouseButtonMiddle) == glfw::Action::Press { mouse_buttons |= 0x01 << 2; }
        return mouse_buttons;
    }
    pub fn reset(&mut self){
        self.mouse_wheel = 0.0;
        self.last_character = '\0';
    }
    pub fn handle_event(&mut self, imgui_context: &mut imgui::Context, event : &glfw::WindowEvent){
        if let glfw::WindowEvent::Key(key, _, action, _) = event {
            GlfwPlatform::handle_key_event(imgui_context, *key, *action);
        } else if let glfw::WindowEvent::Scroll(_, scroll_y) = event {
            self.mouse_wheel = *scroll_y as f32;
        } else if let glfw::WindowEvent::Char(character) = event {
           self.last_character = *character;
        }
    }
    pub fn handle_key_event(imgui_context : &mut imgui::Context, key : glfw::Key, action : glfw::Action){
        if action == glfw::Action::Press || action == glfw::Action::Release {
            let mut io = imgui_context.io_mut();
            if key == glfw::Key::LeftShift || key == glfw::Key::RightShift {
                io.key_shift = action == glfw::Action::Press;
            } else if key == glfw::Key::LeftControl || key == glfw::Key::RightControl {
                io.key_ctrl = action == glfw::Action::Press;
            } else if key == glfw::Key::LeftAlt || key == glfw::Key::RightAlt {
                io.key_alt = action == glfw::Action::Press;
            } else if key == glfw::Key::LeftSuper || key == glfw::Key::RightSuper {
                io.key_super = action == glfw::Action::Press;
            } else if key != glfw::Key::Unknown {
                io.keys_down[key as usize] = action == glfw::Action::Press;
            }
        }
    }
    pub fn get_mouse_wheel(&self) -> f32 {
        return self.mouse_wheel;
    }
    pub fn get_last_character(&self) -> char {
        return self.last_character;
    }
}