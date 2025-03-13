use std::time::{Duration, Instant};

#[derive(Clone, Copy)]
pub enum AnimationState {
    Ready,
    Running,
    Completed,
}

pub trait Animation {
    fn update(&mut self, delta_time: Duration) -> bool;
    fn reset(&mut self);
    fn get_state(&self) -> AnimationState;
}

pub struct FadeAnimation {
    start_value: f32,
    end_value: f32,
    current_value: f32,
    duration: Duration,
    elapsed: Duration,
    state: AnimationState,
}

impl FadeAnimation {
    pub fn new(start_value: f32, end_value: f32, duration: Duration) -> Self {
        Self {
            start_value,
            end_value,
            current_value: start_value,
            duration,
            elapsed: Duration::from_secs(0),
            state: AnimationState::Ready,
        }
    } 

    pub fn get_value(&self) -> f32 {
        self.current_value
    }
}

impl Animation for FadeAnimation {
    fn update(&mut self, delta_time: Duration) -> bool {
        match self.state {
            AnimationState::Ready => {
                self.state = AnimationState::Running;
                self.elapsed = Duration::from_secs(0);
                self.current_value = self.start_value;
                false
            },
            AnimationState::Running => {
                self.elapsed += delta_time;

                if self.elapsed >= self.duration {
                    self.current_value = self.end_value;
                    self.state = AnimationState::Completed;
                    true
                } else {
                    let progress = self.elapsed.as_secs_f32() / self.duration.as_secs_f32();
                    self.current_value = self.start_value + (self.end_value - self.start_value) * progress;
                    false
                }
            },
            AnimationState::Completed => true,
        }
    }

    fn reset(&mut self) {
        self.state = AnimationState::Ready;
        self.elapsed = Duration::from_secs(0);
        self.current_value = self.start_value;
    }

    fn get_state(&self) -> AnimationState {
        self.state.clone()
    }
}

pub struct SlideAnimation {
    start_pos: (i32, i32),
    end_pos: (i32, i32),
    current_pos: (i32, i32),
    duration: Duration,
    elapsed: Duration,
    state: AnimationState,
}

impl SlideAnimation {
    pub fn new(start_pos: (i32, i32), end_pos: (i32, i32), duration: Duration) -> Self {
        Self {
            start_pos,
            end_pos,
            current_pos: start_pos,
            duration,
            elapsed: Duration::from_secs(0),
            state: AnimationState::Ready,
        }
    }

    pub fn get_position(&self) -> (i32, i32) {
        self.current_pos
    }
}

impl Animation for SlideAnimation {
    fn update(&mut self, delta_time: Duration) -> bool {
        match self.state {
            AnimationState::Ready => {
                self.state = AnimationState::Running;
                self.elapsed = Duration::from_secs(0);
                self.current_pos = self.start_pos;
                false
            },
            AnimationState::Running => {
                self.elapsed += delta_time;

                if self.elapsed >= self.duration {
                    self.current_pos = self.end_pos;
                    self.state = AnimationState::Completed;
                    true
                } else {
                    let progress = self.elapsed.as_secs_f32() / self.duration.as_secs_f32();
                    let x = self.start_pos.0 + ((self.end_pos.0 - self.start_pos.0) as f32 * progress) as i32;
                    let y = self.start_pos.1 + ((self.end_pos.1 - self.start_pos.1) as f32 * progress) as i32;
                    self.current_pos = (x, y);
                    false
                }
            },
            AnimationState::Completed => true,
        }
    }

    fn reset(&mut self) {
        self.state = AnimationState::Ready;
        self.elapsed = Duration::from_secs(0);
        self.current_pos = self.start_pos;
    }

    fn get_state(&self) -> AnimationState {
        self.state.clone()
    }
}

pub struct AnimationManager {
    animations: Vec<Box<dyn Animation + Send>>,
    last_update: Instant,
}

impl AnimationManager {
    pub fn new() -> Self {
        Self {
            animations: Vec::new(),
            last_update: Instant::now(),
        }
    }

    pub fn add_animation<A>(&mut self, animation: A)
    where
        A: Animation + Send + 'static,
    {
        self.animations.push(Box::new(animation));
    }

    pub fn update(&mut self) -> bool {
        let now = Instant::now();
        let delta = now.duration_since(self.last_update);
        self.last_update = now;

        let mut all_completed = true;

        for animation in &mut self.animations {
            let completed = animation.update(delta);
            if !completed {
                all_completed = false;
            }
        }

        all_completed
    }

    pub fn reset_all(&mut self) {
        for animation in &mut self.animations {
            animation.reset();
        }
    }
}
