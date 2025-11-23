use ratatui::layout::Flex;
use ratatui::prelude::*;
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct AsciiCell {
    pub ch: char,
    pub x: u16,
    pub y: u16,
    pub color: Color,
}

#[allow(clippy::cast_possible_truncation)]
pub fn parse_ascii_art(
    art: &str,
    color_map_str: &str,
    color_map: &HashMap<char, Color>,
    default_color: Color,
) -> Vec<AsciiCell> {
    let art_lines: Vec<Vec<char>> = art.lines().map(|line| line.chars().collect()).collect();
    let color_lines: Vec<Vec<char>> =
        color_map_str.lines().map(|line| line.chars().collect()).collect();

    assert_eq!(art_lines.len(), color_lines.len(), "Art and color string must have same height");

    let mut pixels = Vec::new();

    for (y, (art_row, color_row)) in art_lines.iter().zip(color_lines.iter()).enumerate() {
        assert_eq!(art_row.len(), color_row.len(), "Mismatched line lengths");

        for (x, (&ch, &color_ch)) in art_row.iter().zip(color_row.iter()).enumerate() {
            let color = color_map.get(&color_ch).copied().unwrap_or(default_color);
            pixels.push(AsciiCell { ch, x: x as u16, y: y as u16, color });
        }
    }

    pixels
}

pub struct AsciiCells {
    pub cells: Vec<AsciiCell>,
}

impl AsciiCells {
    pub fn from(
        art: &str,
        color_map_str: &str,
        color_map: &HashMap<char, Color>,
        default_color: Color,
    ) -> Self {
        Self { cells: parse_ascii_art(art, color_map_str, color_map, default_color) }
    }

    pub fn get_width(&self) -> u16 {
        self.cells.iter().map(|cell| cell.x).max().unwrap_or(0) + 1
    }

    pub fn get_height(&self) -> u16 {
        self.cells.iter().map(|cell| cell.y).max().unwrap_or(0) + 1
    }
}

pub struct AsciiArtWidget {
    collection: AsciiCells,
}

impl AsciiArtWidget {
    pub const fn new(collection: AsciiCells) -> Self {
        Self { collection }
    }
}

impl Widget for AsciiArtWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for pixel in self.collection.cells {
            let position = Position::new(pixel.x + area.x, pixel.y + area.y);

            if area.contains(position) {
                #[allow(clippy::expect_used)]
                buf.cell_mut(position)
                    .expect("Failed to get cell at position")
                    .set_char(pixel.ch)
                    .set_fg(pixel.color);
            }
        }
    }
}

pub struct AsciiAnimationWidget {
    frames: Vec<AsciiCells>,
    frame_duration: Duration,
    looping: bool,
    start_time: Instant,
    pause_at_end: Duration,
}

impl AsciiAnimationWidget {
    /// Create a new animation widget with the given frames and frame duration
    ///
    /// # Example
    /// ```
    /// use std::time::Duration;
    ///
    /// // Create frames
    /// let frame1 = AsciiCells::from(art1, color_map1, &color_map, Color::White);
    /// let frame2 = AsciiCells::from(art2, color_map2, &color_map, Color::White);
    ///
    /// // Create looping animation with 200ms per frame
    /// let animation = AsciiAnimationWidget::new(
    ///     vec![frame1, frame2],
    ///     Duration::from_millis(200),
    ///     true
    /// );
    ///
    /// // Or use convenience methods:
    /// let looping = AsciiAnimationWidget::looping(vec![frame1, frame2])
    ///     .with_frame_duration(Duration::from_millis(200));
    ///
    /// let once = AsciiAnimationWidget::once(vec![frame1, frame2]);
    /// ```
    pub fn new(frames: Vec<AsciiCells>, frame_duration: Duration, looping: bool) -> Self {
        Self {
            frames,
            frame_duration,
            looping,
            start_time: Instant::now(),
            pause_at_end: Duration::ZERO,
        }
    }

    /// Create a new looping animation with default frame duration of 100ms
    pub fn looping(frames: Vec<AsciiCells>) -> Self {
        Self::new(frames, Duration::from_millis(100), true)
    }

    /// Create a new one-shot animation with default frame duration of 100ms
    pub fn once(frames: Vec<AsciiCells>) -> Self {
        Self::new(frames, Duration::from_millis(100), false)
    }

    /// Set the frame duration
    pub fn with_frame_duration(mut self, duration: Duration) -> Self {
        self.frame_duration = duration;
        self
    }

    /// Set a pause duration at the end of the animation (before looping)
    pub fn with_pause_at_end(mut self, pause: Duration) -> Self {
        self.pause_at_end = pause;
        self
    }

    /// Reset the animation to the beginning
    pub fn reset(&mut self) {
        self.start_time = Instant::now();
    }

    /// Get the current frame index based on elapsed time
    fn current_frame_index(&self) -> usize {
        if self.frames.is_empty() {
            return 0;
        }

        let elapsed = self.start_time.elapsed();
        let frame_count = self.frames.len();
        let animation_duration = self.frame_duration * frame_count as u32;
        let total_cycle_duration = animation_duration + self.pause_at_end;

        if self.looping {
            let cycle_time = elapsed.as_millis() % total_cycle_duration.as_millis();

            // If we're in the pause period, show the last frame
            if cycle_time >= animation_duration.as_millis() {
                return frame_count - 1;
            }

            // Otherwise calculate the frame index
            let frames_elapsed = cycle_time / self.frame_duration.as_millis();
            #[allow(clippy::cast_possible_truncation)]
            let frame_index = frames_elapsed as usize;
            frame_index.min(frame_count - 1)
        } else {
            let frames_elapsed = elapsed.as_millis() / self.frame_duration.as_millis();
            #[allow(clippy::cast_possible_truncation)]
            let frame_index = frames_elapsed as usize;
            frame_index.min(frame_count - 1)
        }
    }

    /// Check if the animation has finished (only relevant for non-looping animations)
    pub fn is_finished(&self) -> bool {
        if self.looping || self.frames.is_empty() {
            return false;
        }

        let elapsed = self.start_time.elapsed();
        let total_duration = self.frame_duration * self.frames.len() as u32;
        elapsed >= total_duration
    }

    /// Get the width of the animation (assumes all frames have the same width)
    pub fn get_width(&self) -> u16 {
        self.frames.first().map_or(0, AsciiCells::get_width)
    }

    /// Get the height of the animation (assumes all frames have the same height)
    pub fn get_height(&self) -> u16 {
        self.frames.first().map_or(0, AsciiCells::get_height)
    }

    /// Render the animation to a buffer without consuming self
    pub fn render_to_buffer(&self, area: Rect, buf: &mut Buffer) {
        if self.frames.is_empty() {
            return;
        }

        let frame_index = self.current_frame_index();
        let current_frame = &self.frames[frame_index];

        for pixel in &current_frame.cells {
            let position = Position::new(pixel.x + area.x, pixel.y + area.y);

            if area.contains(position) {
                #[allow(clippy::expect_used)]
                buf.cell_mut(position)
                    .expect("Failed to get cell at position")
                    .set_char(pixel.ch)
                    .set_fg(pixel.color);
            }
        }
    }
}

impl Widget for AsciiAnimationWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if self.frames.is_empty() {
            return;
        }

        let frame_index = self.current_frame_index();
        let current_frame = &self.frames[frame_index];

        for pixel in &current_frame.cells {
            let position = Position::new(pixel.x + area.x, pixel.y + area.y);

            if area.contains(position) {
                #[allow(clippy::expect_used)]
                buf.cell_mut(position)
                    .expect("Failed to get cell at position")
                    .set_char(pixel.ch)
                    .set_fg(pixel.color);
            }
        }
    }
}

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
    color_fn: Box<dyn Fn(usize, usize, f32) -> Color>, // (x, y, progress) -> Color
    char_fn: Option<Box<dyn Fn(usize, usize, f32, char) -> char>>, // (x, y, progress, original_char) -> char
}

impl ProceduralAnimationWidget {
    pub fn new(
        art: String,
        num_frames: usize,
        frame_duration: Duration,
        color_fn: impl Fn(usize, usize, f32) -> Color + 'static,
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
            color_fn: Box::new(color_fn),
            char_fn: None,
        }
    }

    pub fn with_char_fn(
        mut self,
        char_fn: impl Fn(usize, usize, f32, char) -> char + 'static,
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
            self.paused_progress = self.get_animation_progress();
            self.paused = true;
        }
    }

    pub fn unpause(&mut self) {
        if self.paused {
            // Adjust start_time so that the animation continues from paused_progress
            let animation_duration = self.frame_duration * self.num_frames as u32;
            let elapsed_at_pause = Duration::from_millis(
                (self.paused_progress * animation_duration.as_millis() as f32) as u64,
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

    pub fn get_current_progress(&self) -> f32 {
        self.get_animation_progress()
    }

    fn get_animation_progress(&self) -> f32 {
        if self.paused {
            return self.paused_progress;
        }

        let elapsed = self.start_time.elapsed();
        let animation_duration = self.frame_duration * self.num_frames as u32;
        let total_cycle_duration = animation_duration + self.pause_at_end;

        let cycle_time = elapsed.as_millis() % total_cycle_duration.as_millis();

        // If we're in the pause period, return 1.0 (end of animation)
        if cycle_time >= animation_duration.as_millis() {
            return 1.0;
        }

        // Otherwise calculate progress through animation
        cycle_time as f32 / animation_duration.as_millis() as f32
    }

    pub fn render_to_buffer(&self, area: Rect, buf: &mut Buffer) {
        let progress = self.get_animation_progress();
        self.render_to_buffer_at_progress(area, buf, progress);
    }

    pub fn render_to_buffer_at_progress(&self, area: Rect, buf: &mut Buffer, progress: f32) {
        for (y, line) in self.art.lines().enumerate() {
            for (x, ch) in line.chars().enumerate() {
                if ch == ' ' {
                    continue; // Skip spaces
                }

                let color = (self.color_fn)(x, y, progress);

                // Apply character transformation if char_fn is provided
                let display_char = if let Some(ref char_fn) = self.char_fn {
                    char_fn(x, y, progress, ch)
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
