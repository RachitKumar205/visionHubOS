use crate::drivers::display::{DisplayManager, DisplayError, TextSize};
use crate::system::events::{Event, EventHandler};
use embedded_graphics::prelude::*;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use std::any::Any;

pub trait Widget {
    fn draw(&self, display: &DisplayManager) -> Result<(), DisplayError>;
    fn handle_event(&mut self, event: &Event) -> bool;
    fn get_bounds(&self) -> Rectangle;
}

#[derive(Clone)]
pub struct Rectangle {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

pub struct Label {
    text: String,
    position: Point,
    size: TextSize,
    bounds: Rectangle,
}

impl Label {
    pub fn new(text: &str, x: i32, y: i32, size: TextSize) -> Self {
        let char_width = match size {
            TextSize::Small => 5,
            TextSize::Normal => 6,
            TextSize::Large => 8,
        };

        let width = text.len() as u32 * char_width;
        let height = match size {
            TextSize::Small => 8,
            TextSize::Normal => 10,
            TextSize::Large => 16
        };

        Self {
            text: text.to_string(),
            position: Point::new(x, y),
            size,
            bounds: Rectangle {x, y, width, height },
        }
    }

    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();

        let char_width = match self.size {
            TextSize::Small => 5,
            TextSize::Normal => 6,
            TextSize::Large => 8,
        };

        self.bounds.width = text.len() as u32 * char_width;
    }
}

impl Widget for Label {
    fn draw(&self, display: &DisplayManager) -> Result<(), DisplayError> {
        display.draw_text(&self.text, self.position.x, self.position.y, self.size.clone())
    }

    fn handle_event(&mut self, _event: &Event) -> bool {
        false
        //ain't no event that requires to be handled by a label dawg
    }

    fn get_bounds(&self) -> Rectangle {
        self.bounds.clone()
    }
}

pub struct Button {
    label: Label,
    bounds: Rectangle,
    pressed: bool,
    on_click: Option<Box<dyn Fn() + Send>>,
}

impl Button {
    pub fn new(text: &str, x: i32, y: i32, width: u32, height: u32) -> Self {
        let label_x = x + (width as i32 - text.len() as i32 * 6) / 2;
        let label_y = y + (height as i32 - 10) / 2;

        Self {
            label: Label::new(text, label_x, label_y, TextSize::Normal),
            bounds: Rectangle { x, y, width, height },
            pressed: false,
            on_click: None,
        }
    }

    pub fn set_on_click<F>(&mut self, callback: F)
    where
        F: Fn() + Send + 'static,
    {
        self.on_click = Some(Box::new(callback));
    }
}

impl Widget for Button {
    fn draw(&self, display: &DisplayManager) -> Result<(), DisplayError> {
        display.draw_rectangle(
            self.bounds.x,
            self.bounds.y,
            self.bounds.width,
            self.bounds.height,
            self.pressed,
        )?;

        self.label.draw(display)
    }

    fn handle_event(&mut self, event: &Event) -> bool {
        match event {
            Event::ButtonPressed(pin) if *pin == 26 => {
                self.pressed = true;
                true
            },
            Event::ButtonReleased(pin) if *pin == 26 => {
                self.pressed = false;
                if let Some(callback) = &self.on_click {
                    callback();
                }
                true
            },
            _ => false,
        }
    }

    fn get_bounds(&self) -> Rectangle {
        self.bounds.clone()
    }
}

pub struct ProgressBar {
    bounds: Rectangle,
    progress: u8,
}

impl ProgressBar {
    pub fn new(x: i32, y: i32, width: u32, progress: u8) -> Self {
        Self {
            bounds: Rectangle { x, y, width, height: 8 },
            progress: progress.min(100),
        }
    }

    pub fn set_progress(&mut self, progress: u8) {
        self.progress = progress.min(100);
    }
}

impl Widget for ProgressBar {
    fn draw(&self, display: &DisplayManager) -> Result<(), DisplayError> {
        display.draw_progress_bar(
            self.bounds.x,
            self.bounds.y,
            self.bounds.width,
            self.progress
        )
    }

    fn handle_event(&mut self, _event: &Event) -> bool {
        false
    }

    fn get_bounds(&self) -> Rectangle {
        self.bounds.clone()
    }
}

pub trait Screen: Any {
    fn draw(&self) -> Result<(), DisplayError>;
    fn handle_event(&mut self, event: &Event) -> bool;
}

pub struct DefaultScreen {
    widgets: Vec<Box<dyn Widget + Send>>,
    display: Arc<DisplayManager>,
}

impl DefaultScreen {
    pub fn new(display: Arc<DisplayManager>) -> Self {
        Self {
            widgets: Vec::new(),
            display,
        }
    }

    pub fn add_widget<W>(&mut self, widget: W)
    where
        W: Widget + Send + 'static,
    {
        self.widgets.push(Box::new(widget));
    }

}

impl Screen for DefaultScreen {
    fn draw(&self) -> Result<(), DisplayError> {
        self.display.clear()?;

        for widget in &self.widgets {
            widget.draw(&self.display)?;
        }

        self.display.flush()
    }

    fn handle_event(&mut self, event: &Event) -> bool {
        let mut handled = false;

        for widget in &mut self.widgets {
            if widget.handle_event(event) {
                handled = true;
                break;
            }
        }

        if handled {
            let _ = self.draw();
        }

        handled
    }
}

pub struct ScreenManager {
    screens: Vec<Box<dyn Screen + Send>>,
    current_screen: usize,
    display: Arc<DisplayManager>,
    event_queue: Arc<Mutex<VecDeque<Event>>>,
}

impl ScreenManager {
    pub fn new(display: Arc<DisplayManager>, event_queue: Arc<Mutex<VecDeque<Event>>>) -> Self {
        Self {
            screens: Vec::new(),
            current_screen: 0,
            display,
            event_queue,
        }
    }

    pub fn add_screen<S>(&mut self, screen: S)
    where 
        S: Screen + Send + 'static,
    {
        self.screens.push(Box::new(screen));
    }

    pub fn switch_to_screen(&mut self, index: usize) -> Result<(), DisplayError> {
        if index < self.screens.len() {
            self.current_screen = index;
            self.screens[self.current_screen].draw()?;
        }
        Ok(())
    }

    pub fn process_events(&mut self) -> Result<(), DisplayError> {
        let mut queue = self.event_queue.lock().unwrap();

        while let Some(event) = queue.pop_front() {
            self.screens[self.current_screen].handle_event(&event);
        }

        Ok(())
    }

    pub fn get_screen_as_mut<T: 'static>(&mut self) -> Option<&mut T> {
        let screen_box = &mut self.screens[self.current_screen];

        let is_correct_type = {
            let screen: &dyn Screen = &**screen_box;

            screen.type_id() == std::any::TypeId::of::<T>()
        };

        if is_correct_type {
            unsafe {
                let ptr = &mut **screen_box as *mut dyn Screen;
                Some(&mut *(ptr as *mut T))
            }
        } else {
            None
        }

    }
}
