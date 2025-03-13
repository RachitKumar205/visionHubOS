use crate::drivers::display::{DisplayManager, DisplayError, TextSize};
use crate::ui::framework::{Button, Label, Screen, Widget};
use crate::system::events::Event;
use std::sync::Arc;

pub struct MenuItem {
    button: Button,
    action: Arc<dyn Fn() + Send + Sync>,
}

impl MenuItem {
    pub fn new<F>(text: &str, x: i32, y: i32, width: u32, action: F) -> Self 
    where
        F: Fn() + Send + Sync + 'static,
    {
        let action = Arc::new(action);
        let mut button = Button::new(text, x, y, width, 15);
        let action_clone = Arc::clone(&action);
        button.set_on_click(move || action_clone());

        Self {
            button,
            action,
        }
    }
}

pub struct MenuScreen {
    title: Label,
    items: Vec<MenuItem>,
    back_button: Button,
    display: Arc<DisplayManager>,
    selected_index: usize,
}

impl MenuScreen {
    pub fn new(display: Arc<DisplayManager>, title: &str) -> Self {
        let mut back_button = Button::new("Back", 5, 50, 40, 15);

        back_button.set_on_click(|| {
            log::info!("Back button clicked");
        });

        Self {
            title: Label::new(title, 5, 5, TextSize::Normal),
            items: Vec::new(),
            back_button,
            display,
            selected_index: 0,
        }
    }
    
    pub fn add_item<F>(&mut self, text: &str, action: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        let y_position = 20 + (self.items.len() as i32 * 18);
        let item = MenuItem::new(text, 10, y_position, 108, action);
        self.items.push(item);
    }

    pub fn select_next(&mut self) {
        if !self.items.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.items.len();
            let _ = self.draw();
        }
    }

    pub fn select_prev(&mut self) {
        if !self.items.is_empty() {
            self.selected_index = if self.selected_index == 0 {
                self.items.len() - 1
            } else {
                self.selected_index - 1
            };
            let _ = self.draw();
        }
    }

    pub fn activate_selected(&mut self) {
        if !self.items.is_empty() {
            let action = &self.items[self.selected_index].action;
            action();
        }
    }
}

impl Screen for MenuScreen {
    fn draw(&self) -> Result<(), DisplayError> {
        self.display.clear()?;

        self.title.draw(&self.display)?;

        for (index, item) in self.items.iter().enumerate() {
            
            //if item is currently selected, highlight it
            if index == self.selected_index {
                self.display.draw_rectangle(5, item.button.get_bounds().y - 2, 118, item.button.get_bounds().height + 4, false)?;
            }

            item.button.draw(&self.display)?;
        }

        self.back_button.draw(&self.display)?;
        
        self.display.flush()
    }

    fn handle_event(&mut self, event: &Event) -> bool {
        match event {
            Event::ButtonPressed(pin) if *pin == 32 => {
                self.activate_selected();
                true
            },
            Event::ButtonPressed(pin) if *pin == 33 => {
                self.select_next();
                true
            },
            _ => false,
        }
    }
}
