use ratatui::layout::Flex;
use ratatui::prelude::*;
use std::time::{Duration, Instant};

/// Type alias for the color function used in procedural animations
type ColorFn = Box<dyn Fn(usize, usize, f32, usize, Color) -> Color>;

/// Type alias for the character transformation function
type CharFn = Box<dyn Fn(usize, usize, f32, usize, char) -> char>;

/// A procedural animation widget that calculates colors on-the-fly
/// This is much more memory efficient than storing multiple frames
pub struct ProceduralAnimationWidget {
    art: String,
    width: u16,
    height: u16,
    num_frames: usize,
    frame_duration: Duration,
    pause_at_end: Duration,
    start_time: Instant,
    paused: bool,
    paused_progress: f32,
    paused_cycle: usize,
    highlight_color: Color,  // The color for the animated strip
    color_fn: ColorFn,       // (x, y, progress, cycle, highlight_color) -> Color
    char_fn: Option<CharFn>, // (x, y, progress, cycle, original_char) -> char
}

impl ProceduralAnimationWidget {
    pub fn new(
        art: String,
        num_frames: usize,
        frame_duration: Duration,
        color_fn: impl Fn(usize, usize, f32, usize, Color) -> Color + 'static,
    ) -> Self {
        let art_lines: Vec<&str> = art.lines().collect();
        let height = art_lines.len() as u16;
        let width = art_lines.iter().map(|line| line.len()).max().unwrap_or(0) as u16;

        Self {
            art,
            width,
            height,
            num_frames,
            frame_duration,
            pause_at_end: Duration::ZERO,
            start_time: Instant::now(),
            paused: false,
            paused_progress: 0.0,
            paused_cycle: 0,
            highlight_color: Color::LightGreen, // Default color
            color_fn: Box::new(color_fn),
            char_fn: None,
        }
    }

    pub fn with_char_fn(
        mut self,
        char_fn: impl Fn(usize, usize, f32, usize, char) -> char + 'static,
    ) -> Self {
        self.char_fn = Some(Box::new(char_fn));
        self
    }

    pub fn with_pause_at_end(mut self, pause: Duration) -> Self {
        self.pause_at_end = pause;
        self
    }

    pub fn pause(&mut self) {
        if !self.paused {
            let (progress, cycle) = self.get_animation_progress_and_cycle();
            self.paused_progress = progress;
            self.paused_cycle = cycle;
            self.paused = true;
        }
    }

    pub fn unpause(&mut self) {
        if self.paused {
            // Adjust start_time so that the animation continues from paused_progress
            let animation_duration = self.frame_duration * self.num_frames as u32;
            let total_cycle_duration = animation_duration + self.pause_at_end;
            let elapsed_at_pause = Duration::from_millis(
                (self.paused_cycle as f32 * total_cycle_duration.as_millis() as f32
                    + self.paused_progress * animation_duration.as_millis() as f32)
                    as u64,
            );
            self.start_time = Instant::now() - elapsed_at_pause;
            self.paused = false;
        }
    }

    pub fn toggle_pause(&mut self) {
        if self.paused {
            self.unpause();
        } else {
            self.pause();
        }
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn get_width(&self) -> u16 {
        self.width
    }

    pub fn get_height(&self) -> u16 {
        self.height
    }

    /// Set the highlight color for the animation
    pub fn set_highlight_color(&mut self, color: Color) {
        self.highlight_color = color;
    }

    fn get_animation_progress_and_cycle(&self) -> (f32, usize) {
        if self.paused {
            return (self.paused_progress, self.paused_cycle);
        }

        let elapsed = self.start_time.elapsed();
        let animation_duration = self.frame_duration * self.num_frames as u32;
        let total_cycle_duration = animation_duration + self.pause_at_end;

        let cycle = (elapsed.as_millis() / total_cycle_duration.as_millis()) as usize;
        let cycle_time = elapsed.as_millis() % total_cycle_duration.as_millis();

        // If we're in the pause period, return 1.0 (end of animation)
        if cycle_time >= animation_duration.as_millis() {
            return (1.0, cycle);
        }

        // Otherwise calculate progress through animation
        let progress = cycle_time as f32 / animation_duration.as_millis() as f32;
        (progress, cycle)
    }

    pub fn render_to_buffer(&self, area: Rect, buf: &mut Buffer) {
        let (progress, cycle) = self.get_animation_progress_and_cycle();
        self.render_to_buffer_at_progress(area, buf, progress, cycle);
    }

    pub fn render_to_buffer_at_progress(
        &self,
        area: Rect,
        buf: &mut Buffer,
        progress: f32,
        cycle: usize,
    ) {
        for (y, line) in self.art.lines().enumerate() {
            for (x, ch) in line.chars().enumerate() {
                if ch == ' ' {
                    continue; // Skip spaces
                }

                let color = (self.color_fn)(x, y, progress, cycle, self.highlight_color);

                // Apply character transformation if char_fn is provided
                let display_char = if let Some(ref char_fn) = self.char_fn {
                    char_fn(x, y, progress, cycle, ch)
                } else {
                    ch
                };

                let position = Position::new(x as u16 + area.x, y as u16 + area.y);

                if area.contains(position) {
                    #[allow(clippy::expect_used)]
                    buf.cell_mut(position)
                        .expect("Failed to get cell at position")
                        .set_char(display_char)
                        .set_fg(color);
                }
            }
        }
    }
}

pub fn center(area: Rect, horizontal: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal]).flex(Flex::Center).areas(area);

    vertically_center(area)
}

pub fn vertically_center(area: Rect) -> Rect {
    let constraints = [Constraint::Fill(1), Constraint::Min(1), Constraint::Fill(1)];
    let [_, center, _] = Layout::vertical(constraints).areas(area);
    center
}

pub trait When {
    fn when(self, condition: bool, action: impl FnOnce(Self) -> Self) -> Self
    where
        Self: Sized;
}

impl<T> When for T {
    fn when(self, condition: bool, action: impl FnOnce(T) -> T) -> Self {
        if condition { action(self) } else { self }
    }
}
