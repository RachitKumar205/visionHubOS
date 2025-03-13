use crate::drivers::display::{DisplayManager, DisplayError, TextSize};
use crate::ui::framework::{Label, ProgressBar, Screen, Widget};
use crate::system::events::Event;
use std::sync::Arc;
use std::time::Duration;

pub struct LoadingScreen {
    title: Label,
    message: Label,
    progress_bar: ProgressBar,
    display: Arc<DisplayManager>,
    progress: u8,
    step_duration: Duration,
    last_update: std::time::Instant,
}

impl LoadingScreen {
    pub fn new(display: Arc<DisplayManager>, title: &str, message: &str) -> Self {
        Self {
            title: Label::new(title, 10, 10, TextSize::Normal),
            message: Label::new(message, 10, 30, TextSize::Small),
            progress_bar: ProgressBar::new(10, 45, 108, 0),
            display,
            progress: 0,
            step_duration: Duration::from_millis(100),
            last_update: std::time::Instant::now(),
        }
    }

    pub fn set_message(&mut self, message: &str) {
        self.message.set_text(message);
    }

    pub fn set_progress(&mut self, progress: u8) {
        self.progress = progress.min(100);
        self.progress_bar.set_progress(self.progress);
    }

    pub fn update(&mut self) -> Result<(), DisplayError> {
        let now = std::time::Instant::now();

        if now.duration_since(self.last_update) >= self.step_duration {
            if self.progress < 100 {
                self.progress += 1;
                self.progress_bar.set_progress(self.progress);
                self.draw()?;
            }

            self.last_update = now;
        }

        Ok(())
    }

    pub fn is_complete(&self) -> bool {
        self.progress >= 100
    }
}

impl Screen for LoadingScreen {
    fn draw(&self) -> Result<(), DisplayError> {
        self.display.clear()?;

        self.title.draw(&self.display)?;
        self.message.draw(&self.display)?;
        self.progress_bar.draw(&self.display)?;

        self.display.flush()
    }

    fn handle_event(&mut self, event: &Event) -> bool {
        match event {
            Event::SystemTick => {
                let _ = self.update();
                true
            },
            _ => false,
        }
    }
}
