use enigo::{Enigo, KeyboardControllable, MouseControllable};

use crate::protocol::{Action, MouseButton};

pub struct Controller {
    enigo: Enigo,
}

fn to_enigo_button(mouse_button: MouseButton) -> enigo::MouseButton {
    match mouse_button {
        MouseButton::Left => enigo::MouseButton::Left,
        MouseButton::Middle => enigo::MouseButton::Middle,
        MouseButton::Right => enigo::MouseButton::Right,
    }
}

impl Controller {
    pub fn new() -> Self {
        Self { enigo: Enigo::new() }
    }

    pub fn perform(&mut self, action: Action) {
        match action {
            Action::KeySequence { text } => self.enigo.key_sequence(&text),
            Action::MouseMoveTo { point } => self.enigo.mouse_move_to(point.x, point.y),
            Action::MouseMoveBy { delta } => self.enigo.mouse_move_relative(delta.x, delta.y),
            Action::MouseDown { button } => self.enigo.mouse_down(to_enigo_button(button)),
            Action::MouseUp { button } => self.enigo.mouse_up(to_enigo_button(button)),
            Action::MouseClick { button } => self.enigo.mouse_click(to_enigo_button(button)),
        }
    }
}
