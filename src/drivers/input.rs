use esp_idf_hal::{
    gpio::{AnyIOPin, Input, Pin, PinDriver},
    prelude::*,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::system::events::{Event, EventQueue};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    Pressed,
    Released,
}

#[derive(Debug)]
pub enum InputError {
    GpioError,
    NotInitialized,
}

pub struct InputManager {
    button_states: Arc<Mutex<HashMap<u32, ButtonState>>>,
    event_queue: Arc<EventQueue>,
}

impl InputManager {
    pub fn new(event_queue: Arc<EventQueue>) -> Self {
        Self {
            button_states: Arc::new(Mutex::new(HashMap::new())),
            event_queue,
        }
    }

    pub fn register_button<P: Pin>(&self, pin: &PinDriver<'_, P, Input>, pin_number: u32) -> Result<(), InputError> {
        let mut states = self.button_states.lock().unwrap();
        states.insert(pin_number, if pin.is_high() {ButtonState::Released} else {ButtonState::Pressed});
        Ok(())
    }

    pub fn update_button_state(&self, pin_number: u32, state: bool) -> Result<(), InputError> {
        let mut states = self.button_states.lock().unwrap();

        if let Some(current_state) = states.get(&pin_number) {
            let new_state = if state {ButtonState::Released} else {ButtonState::Pressed};

            if new_state != *current_state {
                match new_state {
                    ButtonState::Pressed => {
                        self.event_queue.push(Event::ButtonPressed(pin_number));
                    },
                    ButtonState::Released => {
                        self.event_queue.push(Event::ButtonReleased(pin_number));
                    },
                }

                states.insert(pin_number, new_state);
            }
        }

        Ok(())
    }

    pub fn get_button_state(&self, pin_number: u32) -> Result<ButtonState, InputError> {
        let states = self.button_states.lock().unwrap();
        states.get(&pin_number).copied().ok_or(InputError::NotInitialized)
    }
}

pub struct ButtonPoller<'a> {
    input_manager: Arc<InputManager>,
    buttons: Vec<(PinDriver<'a, AnyIOPin, Input>, u32)>,
}

impl<'a> ButtonPoller<'a> {
    pub fn new(input_manager: Arc<InputManager>) -> Self {
        Self {
            input_manager,
            buttons: Vec::new(),
        }
    }

    pub fn add_button(&mut self, pin: PinDriver<'a, AnyIOPin, Input>, pin_number: u32) {
        let _ = self.input_manager.register_button(&pin, pin_number);
        self.buttons.push((pin, pin_number));
    }

    pub fn poll(&mut self) {
        for (pin, pin_number) in &self.buttons {
            let _ = self.input_manager.update_button_state(*pin_number, pin.is_high());
        }
    }
}

