use crate::drivers::display::{DisplayManager, DisplayError, TextSize};
use crate::ui::framework::{Button, Label, Screen, Widget};
use crate::system::events::Event;
use std::sync::Arc;

pub struct HomeScreen {
    title: Label,
    status: Label,
    menu_button: Button,
    settings_button: Button,
    display: Arc<DisplayManager>,
    counter: u32,
}

impl HomeScreen {
    pub fn new(display: Arc<DisplayManager>) -> Self {
        let mut screen = Self {
            title: Label::new("visionHub OS Home", 5, 5, TextSize::Normal),
            status: Label::new("System Ready", 5, 20, TextSize::Small),
            menu_button: Button::new("Menu", 5, 35, 50, 20),
            settings_button: Button::new("Settings", 70, 35, 50, 20),
            display,
            counter: 0,
        };

        let counter = screen.counter;
        screen.menu_button.set_on_click(move || {
            log::info!("Menu button clicked");
        });

        screen.settings_button.set_on_click(move || {
            log::info!("Settings button clicked");
        });

        screen
    }

    pub fn update_status(&mut self, status: &str) {
        self.status.set_text(status);
    }

    pub fn increment_counter(&mut self) {
        self.counter += 1;
        self.update_status(&format!("Count: {}", self.counter));
    }


}

impl Screen for HomeScreen {
    fn draw(&self) -> Result<(), DisplayError> {
        self.display.clear()?;

        self.title.draw(&self.display)?;
        self.status.draw(&self.display)?;
        self.menu_button.draw(&self.display)?;
        self.settings_button.draw(&self.display)?;

        self.display.flush()
    }

    fn handle_event(&mut self, event: &Event) -> bool {
        match event {
            Event::ButtonPressed(pin) if *pin == 26 => {
                self.menu_button.handle_event(event);
                self.increment_counter();
                true
            },
            Event::ButtonReleased(pin) if *pin == 26 => {
                self.menu_button.handle_event(event);
                true
            },
            _ => false,
        }
    }
}
