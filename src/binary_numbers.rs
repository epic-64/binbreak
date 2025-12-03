use crate::app::{NumberMode, get_mode_color};
use crate::keybinds;
use crate::main_screen_widget::{MainScreenWidget, WidgetRef};
use crate::utils::{When, center};
use crossterm::event::{KeyCode, KeyEvent};
use rand::Rng;
use rand::prelude::SliceRandom;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Flex, Layout, Rect};
use ratatui::prelude::Alignment::Center;
use ratatui::prelude::{Color, Line, Style, Stylize, Widget};
use ratatui::style::Modifier;
use ratatui::text::Span;
use ratatui::widgets::BorderType::Double;
use ratatui::widgets::{Block, BorderType, Paragraph};
use std::collections::HashMap;
use std::fmt::Write as _;
use std::fs::File;
use std::io::{Read, Write};

struct StatsSnapshot {
    score: u32,
    streak: u32,
    max_streak: u32,
    rounds: u32,
    lives: u32,
    bits: Bits,
    number_mode: NumberMode,
    hearts: String,
    game_state: GameState,
    prev_high_score: u32,
    new_high_score: bool,
}

impl WidgetRef for BinaryNumbersGame {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let [game_column] = Layout::horizontal([Constraint::Length(65)])
            .flex(Flex::Center)
            .horizontal_margin(1)
            .areas(area);

        self.puzzle.render_ref(game_column, buf);
    }
}

impl WidgetRef for BinaryNumbersPuzzle {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let [middle] =
            Layout::horizontal([Constraint::Percentage(100)]).flex(Flex::Center).areas(area);

        let [stats_area, current_number_area, suggestions_area, progress_bar_area, result_area] =
            Layout::vertical([
                Constraint::Length(4),
                Constraint::Length(5),
                Constraint::Length(3),
                Constraint::Length(4),
                Constraint::Length(5),
            ])
            .flex(Flex::Center)
            .horizontal_margin(0)
            .areas(middle);

        self.render_stats_area(stats_area, buf);

        if let Some(stats) = &self.stats_snapshot
            && stats.game_state == GameState::GameOver
        {
            render_game_over(
                stats,
                current_number_area,
                suggestions_area,
                progress_bar_area,
                result_area,
                buf,
            );
            return;
        }

        self.render_current_number(current_number_area, buf);
        self.render_suggestions(suggestions_area, buf);
        self.render_status_and_timer(progress_bar_area, buf);
        self.render_instructions(result_area, buf);
    }
}

impl BinaryNumbersPuzzle {
    fn render_stats_area(&self, area: Rect, buf: &mut Buffer) {
        Block::bordered().title_alignment(Center).dark_gray().render(area, buf);

        if let Some(stats) = &self.stats_snapshot {
            let high_label = if stats.new_high_score {
                let style = Style::default().fg(Color::LightGreen).add_modifier(Modifier::BOLD);
                Span::styled(format!("Hi-Score: {}*  ", stats.score), style)
            } else {
                let style = Style::default().fg(Color::DarkGray);
                Span::styled(format!("Hi-Score: {}  ", stats.prev_high_score), style)
            };

            let mode_color = get_mode_color(&stats.bits);
            let mode_label = format!("{} {}", stats.bits.label(), stats.number_mode.label());
            let line1 = Line::from(vec![
                Span::styled(format!("Mode: {}  ", mode_label), Style::default().fg(mode_color)),
                high_label,
            ]);

            let line2 = Line::from(vec![
                Span::styled(
                    format!("Score: {}  ", stats.score),
                    Style::default().fg(Color::Green),
                ),
                Span::styled(
                    format!("Streak: {}  ", stats.streak),
                    Style::default().fg(Color::Cyan),
                ),
                Span::styled(
                    format!("Max: {}  ", stats.max_streak),
                    Style::default().fg(Color::Blue),
                ),
                Span::styled(
                    format!("Rounds: {}  ", stats.rounds),
                    Style::default().fg(Color::Magenta),
                ),
                Span::styled(format!("Lives: {}  ", stats.hearts), Style::default().fg(Color::Red)),
            ]);

            #[allow(clippy::cast_possible_truncation)]
            let widest = line1.width().max(line2.width()) as u16;
            Paragraph::new(vec![line1, line2])
                .alignment(Center)
                .render(center(area, Constraint::Length(widest)), buf);
        }
    }

    fn render_current_number(&self, area: Rect, buf: &mut Buffer) {
        let [inner] =
            Layout::horizontal([Constraint::Percentage(100)]).flex(Flex::Center).areas(area);

        Block::bordered()
            .border_type(Double)
            .border_style(Style::default().dark_gray())
            .render(inner, buf);

        let binary_string = self.current_to_binary_string();
        let scale_suffix = match self.bits {
            Bits::FourShift4 => Some(" x16"),
            Bits::FourShift8 => Some(" x256"),
            Bits::FourShift12 => Some(" x4096"),
            _ => None,
        };
        let mut spans = vec![Span::raw(binary_string)];
        if let Some(sfx) = scale_suffix {
            spans.push(Span::styled(sfx, Style::default().fg(Color::DarkGray)));
        }
        #[allow(clippy::cast_possible_truncation)]
        let total_width = spans.iter().map(ratatui::prelude::Span::width).sum::<usize>() as u16;
        let lines: Vec<Line> = vec![Line::from(spans)];
        Paragraph::new(lines)
            .alignment(Center)
            .render(center(inner, Constraint::Length(total_width)), buf);
    }

    fn render_suggestions(&self, area: Rect, buf: &mut Buffer) {
        let suggestions = self.suggestions();
        let suggestions_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Min(6); suggestions.len()])
            .split(area);

        for (i, suggestion) in suggestions.iter().enumerate() {
            let item_is_selected = self.selected_suggestion == Some(*suggestion);
            let show_correct_number = self.guess_result.is_some();
            let is_correct_number = self.is_correct_guess(*suggestion);
            let area = suggestions_layout[i];

            let border_type = if item_is_selected {
                BorderType::Double
            } else {
                BorderType::Plain
            };

            let border_color = if item_is_selected {
                match self.guess_result {
                    Some(GuessResult::Correct) => Color::Green,
                    Some(GuessResult::Incorrect) => Color::Red,
                    Some(GuessResult::Timeout) => Color::Yellow,
                    None => Color::LightCyan,
                }
            } else {
                Color::DarkGray
            };

            Block::bordered().border_type(border_type).fg(border_color).render(area, buf);

            let suggestion_str = format!("{suggestion}");

            #[allow(clippy::cast_possible_truncation)]
            Paragraph::new(suggestion_str.to_string())
                .white()
                .when(show_correct_number && is_correct_number, |p| p.light_green().underlined())
                .alignment(Center)
                .render(center(area, Constraint::Length(suggestion_str.len() as u16)), buf);
        }
    }

    fn render_status_and_timer(&self, area: Rect, buf: &mut Buffer) {
        let [left, right] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .areas(area);

        self.render_status(left, buf);
        self.render_timer(right, buf);
    }

    fn render_status(&self, area: Rect, buf: &mut Buffer) {
        Block::bordered()
            .dark_gray()
            .title("Status")
            .title_alignment(Center)
            .title_style(Style::default().white())
            .render(area, buf);

        if let Some(result) = &self.guess_result {
            let (icon, line1_text, color) = match result {
                GuessResult::Correct => (":)", "success", Color::Green),
                GuessResult::Incorrect => (":(", "incorrect", Color::Red),
                GuessResult::Timeout => (":(", "time's up", Color::Yellow),
            };

            let gained_line = match result {
                GuessResult::Correct => format!("gained {} points", self.last_points_awarded),
                GuessResult::Incorrect => "lost a life".to_string(),
                GuessResult::Timeout => "timeout".to_string(),
            };

            let text = vec![
                Line::from(format!("{icon} {line1_text}").fg(color)),
                Line::from(gained_line.fg(color)),
            ];
            #[allow(clippy::cast_possible_truncation)]
            let widest = text.iter().map(Line::width).max().unwrap_or(0) as u16;
            Paragraph::new(text)
                .alignment(Center)
                .style(Style::default().fg(color))
                .render(center(area, Constraint::Length(widest)), buf);
        }
    }

    fn render_timer(&self, area: Rect, buf: &mut Buffer) {
        let ratio = self.time_left / self.time_total;
        let gauge_color = if ratio > 0.6 {
            Color::Green
        } else if ratio > 0.3 {
            Color::Yellow
        } else {
            Color::Red
        };

        let time_block = Block::bordered()
            .dark_gray()
            .title("Time Remaining")
            .title_style(Style::default().white())
            .title_alignment(Center);
        let inner_time = time_block.inner(area);
        time_block.render(area, buf);

        let [gauge_line, time_line] =
            Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).areas(inner_time);

        render_ascii_gauge(gauge_line, buf, ratio, gauge_color);

        Paragraph::new(Line::from(Span::styled(
            format!("{:.2} seconds left", self.time_left),
            Style::default().fg(gauge_color),
        )))
        .alignment(Center)
        .render(time_line, buf);
    }

    fn render_instructions(&self, area: Rect, buf: &mut Buffer) {
        Block::bordered().dark_gray().render(area, buf);

        let instruction_spans: Vec<Span> = [
            hotkey_span("Left Right", "select  "),
            hotkey_span("Enter", "confirm  "),
            hotkey_span("S", "skip  "),
            hotkey_span("Esc", "exit"),
        ]
        .iter()
        .flatten()
        .cloned()
        .collect();

        Paragraph::new(vec![Line::from(instruction_spans)])
            .alignment(Center)
            .render(center(area, Constraint::Length(65)), buf);
    }
}

fn hotkey_span<'a>(key: &'a str, description: &str) -> Vec<Span<'a>> {
    vec![
        Span::styled("<", Style::default().fg(Color::White)),
        Span::styled(key, Style::default().fg(Color::LightCyan)),
        Span::styled(format!("> {description}"), Style::default().fg(Color::White)),
    ]
}

fn render_game_over(
    stats: &StatsSnapshot,
    current_number_area: Rect,
    suggestions_area: Rect,
    progress_bar_area: Rect,
    result_area: Rect,
    buf: &mut Buffer,
) {
    let combined_rect = Rect {
        x: current_number_area.x,
        y: current_number_area.y,
        width: current_number_area.width,
        height: current_number_area.height
            + suggestions_area.height
            + progress_bar_area.height
            + result_area.height,
    };
    Block::bordered().border_style(Style::default().fg(Color::DarkGray)).render(combined_rect, buf);

    let mut lines = vec![
        Line::from(Span::styled(
            format!("Final Score: {}", stats.score),
            Style::default().fg(Color::Green),
        )),
        Line::from(Span::styled(
            format!("Previous High: {}", stats.prev_high_score),
            Style::default().fg(Color::Yellow),
        )),
        Line::from(Span::styled(
            format!("Rounds Played: {}", stats.rounds),
            Style::default().fg(Color::Magenta),
        )),
        Line::from(Span::styled(
            format!("Max Streak: {}", stats.max_streak),
            Style::default().fg(Color::Cyan),
        )),
    ];
    if stats.new_high_score {
        lines.insert(
            1,
            Line::from(Span::styled(
                "NEW HIGH SCORE!",
                Style::default().fg(Color::LightGreen).bold(),
            )),
        );
    }
    if stats.lives == 0 {
        lines.push(Line::from(Span::styled(
            "You lost all your lives.",
            Style::default().fg(Color::Red),
        )));
    }
    lines.push(Line::from(Span::styled(
        "Press Enter to restart or Esc to exit",
        Style::default().fg(Color::Yellow),
    )));
    Paragraph::new(lines)
        .alignment(Center)
        .render(center(combined_rect, Constraint::Length(48)), buf);
}

pub struct BinaryNumbersGame {
    puzzle: BinaryNumbersPuzzle,
    bits: Bits,
    number_mode: NumberMode,
    exit_intended: bool,
    score: u32,
    streak: u32,
    rounds: u32,
    puzzle_resolved: bool,
    lives: u32,
    max_lives: u32,
    game_state: GameState,
    max_streak: u32,
    high_scores: HighScores,
    prev_high_score_for_display: u32,
    new_high_score_reached: bool,
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum GameState {
    Active,
    Result,
    PendingGameOver,
    GameOver,
}

impl MainScreenWidget for BinaryNumbersGame {
    fn run(&mut self, dt: f64) {
        self.refresh_stats_snapshot();
        if self.game_state == GameState::GameOver {
            return;
        }
        self.puzzle.run(dt);
        if self.puzzle.guess_result.is_some() && !self.puzzle_resolved {
            self.finalize_round();
        }
        self.refresh_stats_snapshot();
    }

    fn handle_input(&mut self, input: KeyEvent) {
        self.handle_game_input(input);
    }
    fn is_exit_intended(&self) -> bool {
        self.exit_intended
    }
}

impl BinaryNumbersGame {
    pub fn new(bits: Bits, number_mode: NumberMode) -> Self {
        Self::new_with_max_lives(bits, number_mode, 3)
    }
    pub fn new_with_max_lives(bits: Bits, number_mode: NumberMode, max_lives: u32) -> Self {
        let hs = HighScores::load();
        let high_score_key = Self::compute_high_score_key(&bits, number_mode);
        let starting_prev = hs.get(&high_score_key);
        let mut game = Self {
            bits: bits.clone(),
            number_mode,
            puzzle: Self::init_puzzle(bits, number_mode, 0),
            exit_intended: false,
            score: 0,
            streak: 0,
            rounds: 0,
            puzzle_resolved: false,
            lives: max_lives.min(3),
            max_lives,
            game_state: GameState::Active,
            max_streak: 0,
            high_scores: hs,
            prev_high_score_for_display: starting_prev,
            new_high_score_reached: false,
        };
        // Initialize stats snapshot immediately so stats display on first render
        game.refresh_stats_snapshot();
        game
    }

    pub fn init_puzzle(bits: Bits, number_mode: NumberMode, streak: u32) -> BinaryNumbersPuzzle {
        BinaryNumbersPuzzle::new(bits, number_mode, streak)
    }

    fn compute_high_score_key(bits: &Bits, number_mode: NumberMode) -> String {
        let bits_key = bits.high_score_key();
        let mode_suffix = match number_mode {
            NumberMode::Unsigned => "u",
            NumberMode::Signed => "s",
        };
        format!("{}{}", bits_key, mode_suffix)
    }

    pub fn is_active(&self) -> bool {
        self.game_state == GameState::Active
    }
}

impl BinaryNumbersGame {
    pub fn lives_hearts(&self) -> String {
        let full_count = self.lives.min(self.max_lives) as usize;
        let full = "♥".repeat(full_count);
        let empty_count = self.max_lives.saturating_sub(self.lives) as usize;
        let empty = "·".repeat(empty_count);
        format!("{full}{empty}")
    }

    fn finalize_round(&mut self) {
        if let Some(result) = self.puzzle.guess_result {
            self.rounds += 1;
            match result {
                GuessResult::Correct => {
                    self.streak += 1;
                    if self.streak > self.max_streak {
                        self.max_streak = self.streak;
                    }
                    let streak_bonus = (self.streak - 1) * 2;
                    let points = 10 + streak_bonus;
                    self.score += points;
                    self.puzzle.last_points_awarded = points;
                    if self.streak.is_multiple_of(5) && self.lives < self.max_lives {
                        self.lives += 1;
                    }
                },
                GuessResult::Incorrect | GuessResult::Timeout => {
                    self.streak = 0;
                    self.puzzle.last_points_awarded = 0;
                    if self.lives > 0 {
                        self.lives -= 1;
                    }
                },
            }
            // high score update
            let bits_key = Self::compute_high_score_key(&self.bits, self.number_mode);
            let prev = self.high_scores.get(&bits_key);
            if self.score > prev {
                if !self.new_high_score_reached {
                    self.prev_high_score_for_display = prev;
                }
                self.high_scores.update(&bits_key, self.score);
                self.new_high_score_reached = true;
                let _ = self.high_scores.save();
            }
            // set state after round resolution
            if self.lives == 0 {
                self.game_state = GameState::PendingGameOver; // defer summary until Enter
            } else {
                self.game_state = GameState::Result;
            }
            self.puzzle_resolved = true;
        }
    }

    pub fn handle_game_input(&mut self, input: KeyEvent) {
        if keybinds::is_exit(input) {
            self.exit_intended = true;
            return;
        }

        if self.game_state == GameState::GameOver {
            self.handle_game_over_input(input);
            return;
        }
        match self.puzzle.guess_result {
            None => self.handle_no_result_yet(input),
            Some(_) => self.handle_result_available(input),
        }
    }

    fn handle_game_over_input(&mut self, key: KeyEvent) {
        match key {
            x if keybinds::is_select(x) => {
                self.reset_game_state();
            },
            x if keybinds::is_exit(x) => {
                self.exit_intended = true;
            },
            _ => {},
        }
    }

    fn reset_game_state(&mut self) {
        self.score = 0;
        self.streak = 0;
        self.rounds = 0;
        self.lives = self.max_lives.min(3);
        self.game_state = GameState::Active;
        self.max_streak = 0;
        let high_score_key = Self::compute_high_score_key(&self.bits, self.number_mode);
        self.prev_high_score_for_display = self.high_scores.get(&high_score_key);
        self.new_high_score_reached = false;
        self.puzzle = Self::init_puzzle(self.bits.clone(), self.number_mode, 0);
        self.puzzle_resolved = false;
        self.refresh_stats_snapshot();
    }

    fn handle_no_result_yet(&mut self, input: KeyEvent) {
        match input {
            x if keybinds::is_right(x) => {
                // select the next suggestion
                if let Some(selected) = self.puzzle.selected_suggestion {
                    let current_index = self.puzzle.suggestions.iter().position(|&x| x == selected);
                    if let Some(index) = current_index {
                        let next_index = (index + 1) % self.puzzle.suggestions.len();
                        self.puzzle.selected_suggestion = Some(self.puzzle.suggestions[next_index]);
                    }
                } else {
                    // if no suggestion is selected, select the first one
                    self.puzzle.selected_suggestion = Some(self.puzzle.suggestions[0]);
                }
            },
            x if keybinds::is_left(x) => {
                // select the previous suggestion
                if let Some(selected) = self.puzzle.selected_suggestion {
                    let current_index = self.puzzle.suggestions.iter().position(|&x| x == selected);
                    if let Some(index) = current_index {
                        let prev_index = if index == 0 {
                            self.puzzle.suggestions.len() - 1
                        } else {
                            index - 1
                        };
                        self.puzzle.selected_suggestion = Some(self.puzzle.suggestions[prev_index]);
                    }
                }
            },
            x if keybinds::is_select(x) => {
                if let Some(selected) = self.puzzle.selected_suggestion {
                    if self.puzzle.is_correct_guess(selected) {
                        self.puzzle.guess_result = Some(GuessResult::Correct);
                    } else {
                        self.puzzle.guess_result = Some(GuessResult::Incorrect);
                    }
                    self.finalize_round();
                }
            },
            KeyEvent { code: KeyCode::Char('s' | 'S'), .. } => {
                // Skip puzzle counts as timeout
                self.puzzle.guess_result = Some(GuessResult::Timeout);
                self.finalize_round();
            },
            _ => {},
        }
    }

    fn handle_result_available(&mut self, key: KeyEvent) {
        match key {
            x if keybinds::is_select(x) => {
                match self.game_state {
                    GameState::PendingGameOver => {
                        // reveal summary
                        self.game_state = GameState::GameOver;
                    },
                    GameState::Result => {
                        // start next puzzle
                        self.puzzle =
                            Self::init_puzzle(self.bits.clone(), self.number_mode, self.streak);
                        self.puzzle_resolved = false;
                        self.game_state = GameState::Active;
                    },
                    GameState::GameOver => { /* handled elsewhere */ },
                    GameState::Active => { /* shouldn't be here */ },
                }
            },
            x if keybinds::is_exit(x) => self.exit_intended = true,
            _ => {},
        }
    }

    fn refresh_stats_snapshot(&mut self) {
        self.puzzle.stats_snapshot = Some(StatsSnapshot {
            score: self.score,
            streak: self.streak,
            max_streak: self.max_streak,
            rounds: self.rounds,
            lives: self.lives,
            bits: self.bits.clone(),
            number_mode: self.number_mode,
            hearts: self.lives_hearts(),
            game_state: self.game_state,
            prev_high_score: self.prev_high_score_for_display,
            new_high_score: self.new_high_score_reached,
        });
    }
}

#[derive(PartialEq, Copy, Clone, Debug)]
enum GuessResult {
    Correct,
    Incorrect,
    Timeout,
}

#[derive(Clone)]
pub enum Bits {
    Four,
    FourShift4,
    FourShift8,
    FourShift12,
    Eight,
    Twelve,
    Sixteen,
}

impl Bits {
    pub const fn to_int(&self) -> u32 {
        match self {
            Self::Four | Self::FourShift4 | Self::FourShift8 | Self::FourShift12 => 4,
            Self::Eight => 8,
            Self::Twelve => 12,
            Self::Sixteen => 16,
        }
    }
    pub const fn scale_factor(&self) -> u32 {
        match self {
            Self::Four => 1,
            Self::FourShift4 => 16,
            Self::FourShift8 => 256,
            Self::FourShift12 => 4096,
            Self::Eight => 1,
            Self::Twelve => 1,
            Self::Sixteen => 1,
        }
    }
    pub const fn high_score_key(&self) -> u32 {
        match self {
            Self::Four => 4,
            Self::FourShift4 => 44,
            Self::FourShift8 => 48,
            Self::FourShift12 => 412,
            Self::Eight => 8,
            Self::Twelve => 12,
            Self::Sixteen => 16,
        }
    }
    pub const fn upper_bound(&self) -> u32 {
        (u32::pow(2, self.to_int()) - 1) * self.scale_factor()
    }
    pub const fn suggestion_count(&self) -> usize {
        match self {
            Self::Four | Self::FourShift4 | Self::FourShift8 | Self::FourShift12 => 3,
            Self::Eight => 4,
            Self::Twelve => 5,
            Self::Sixteen => 6,
        }
    }
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Four => "4 bit",
            Self::FourShift4 => "4 bit*16",
            Self::FourShift8 => "4 bit*256",
            Self::FourShift12 => "4 bit*4096",
            Self::Eight => "8 bit",
            Self::Twelve => "12 bit",
            Self::Sixteen => "16 bit",
        }
    }
}

pub struct BinaryNumbersPuzzle {
    bits: Bits,
    /// Raw bit pattern (unscaled) for display as binary string.
    /// This is u32 (not i32) because it stores the BIT PATTERN, not the numeric value.
    /// In signed mode, negative numbers use two's complement representation:
    /// - For -1 in 4-bit: raw_current_number = 15 (0b1111), displayed as "1111"
    /// - For -8 in 4-bit: raw_current_number = 8 (0b1000), displayed as "1000"
    ///
    /// The same bit pattern has different meanings in signed vs unsigned mode.
    raw_current_number: u32,
    suggestions: Vec<i32>,
    correct_answer: i32,
    selected_suggestion: Option<i32>,
    time_total: f64,
    time_left: f64,
    guess_result: Option<GuessResult>,
    last_points_awarded: u32,
    stats_snapshot: Option<StatsSnapshot>,
    skip_first_dt: bool, // Skip first dt to prevent timer jump when starting new puzzle
}

impl BinaryNumbersPuzzle {
    pub fn new(bits: Bits, number_mode: NumberMode, streak: u32) -> Self {
        let mut rng = rand::rng();

        let mut suggestions = Vec::new();
        let scale = bits.scale_factor();
        let num_bits = bits.to_int();

        match number_mode {
            NumberMode::Unsigned => {
                while suggestions.len() < bits.suggestion_count() {
                    let raw = rng.random_range(0..u32::pow(2, num_bits));
                    let num = (raw * scale) as i32;
                    if !suggestions.contains(&num) {
                        suggestions.push(num);
                    }
                }
            },
            NumberMode::Signed => {
                // For signed mode, use two's complement representation
                // Range is from -(2^(n-1)) to 2^(n-1)-1
                while suggestions.len() < bits.suggestion_count() {
                    let raw = rng.random_range(0..u32::pow(2, num_bits));
                    // Convert raw bits to signed value using two's complement
                    let signed_val = if raw >= (1 << (num_bits - 1)) {
                        // Negative number: raw - 2^n
                        (raw as i32) - (1 << num_bits)
                    } else {
                        // Positive number
                        raw as i32
                    };
                    let num = signed_val * (scale as i32);
                    if !suggestions.contains(&num) {
                        suggestions.push(num);
                    }
                }
            },
        }

        // Pick a random suggestion as the current number
        let correct_index = rng.random_range(0..suggestions.len());
        let current_number_signed = suggestions[correct_index];

        // Shuffle suggestions so the correct answer is in a random position
        suggestions.shuffle(&mut rng);

        // Calculate raw_current_number based on mode
        let raw_current_number = match number_mode {
            NumberMode::Unsigned => {
                let current_number = current_number_signed.unsigned_abs();
                current_number / scale
            },
            NumberMode::Signed => {
                // For signed mode, we need to preserve the two's complement representation
                // Example: -1 in 4-bit two's complement is 0b1111
                // We cast i32 to u32 (preserving bit pattern), then mask to n-bits
                // Result: -1 becomes 15u32 (0b1111), which displays as "1111"
                let unscaled_signed = current_number_signed / (scale as i32);

                // Convert to unsigned bits using two's complement masking
                // Casting i32 to u32 reinterprets the bits (not a numeric conversion)
                // For n-bit number, mask is (2^n - 1)
                let mask = (1u32 << num_bits) - 1;
                (unscaled_signed as u32) & mask
            },
        };

        // Calculate time based on difficulty
        let time_total = 10.0 - (streak.min(8) as f64 * 0.5);
        let time_left = time_total;

        let selected_suggestion = Some(suggestions[0]);
        let guess_result = None;
        let last_points_awarded = 0;

        Self {
            bits,
            raw_current_number,
            suggestions,
            correct_answer: current_number_signed,
            time_total,
            time_left,
            selected_suggestion,
            guess_result,
            last_points_awarded,
            stats_snapshot: None,
            skip_first_dt: true,
        }
    }

    pub fn suggestions(&self) -> &[i32] {
        &self.suggestions
    }

    pub fn is_correct_guess(&self, guess: i32) -> bool {
        guess == self.correct_answer
    }

    pub fn current_to_binary_string(&self) -> String {
        let width = self.bits.to_int() as usize;
        let raw = format!("{:0width$b}", self.raw_current_number, width = width);
        raw.chars()
            .collect::<Vec<_>>()
            .chunks(4)
            .map(|chunk| chunk.iter().collect::<String>())
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub fn run(&mut self, dt: f64) {
        if self.skip_first_dt {
            self.skip_first_dt = false;
            return;
        }
        if self.guess_result.is_some() {
            return;
        }
        self.time_left -= dt;
        if self.time_left <= 0.0 {
            self.guess_result = Some(GuessResult::Timeout);
        }
    }
}

impl Widget for &mut BinaryNumbersGame {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.render_ref(area, buf);
    }
}

// Simple ASCII gauge renderer to avoid variable glyph heights from Unicode block elements
fn render_ascii_gauge(area: Rect, buf: &mut Buffer, ratio: f64, color: Color) {
    #[allow(clippy::cast_sign_loss)]
    #[allow(clippy::cast_possible_truncation)]
    let fill_width =
        (f64::from(area.width) * ratio.clamp(0.0, 1.0)).round().min(f64::from(area.width)) as u16;

    if area.height == 0 {
        return;
    }

    for x in 0..area.width {
        let filled = x < fill_width;
        let symbol = if filled { "=" } else { " " };
        let style = if filled {
            Style::default().fg(color)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        if let Some(cell) = buf.cell_mut((area.x + x, area.y)) {
            cell.set_symbol(symbol);
            cell.set_style(style);
        }
    }
}

struct HighScores {
    scores: HashMap<String, u32>,
}

impl HighScores {
    const FILE: &'static str = "binbreak_highscores.txt";

    fn empty() -> Self {
        Self { scores: HashMap::new() }
    }

    fn load() -> Self {
        let mut hs = Self::empty();
        if let Ok(mut file) = File::open(Self::FILE) {
            let mut contents = String::new();
            if file.read_to_string(&mut contents).is_ok() {
                for line in contents.lines() {
                    if let Some((k, v)) = line.split_once('=')
                        && let Ok(score) = v.trim().parse::<u32>()
                    {
                        hs.scores.insert(k.trim().to_string(), score);
                    }
                }
            }
        }
        hs
    }

    fn save(&self) -> std::io::Result<()> {
        let mut data = String::new();
        for key in [
            "4u", "4s", "44u", "44s", "48u", "48s", "412u", "412s", "8u", "8s", "12u", "12s",
            "16u", "16s",
        ] {
            let val = self.get(key);
            let _ = writeln!(data, "{key}={val}");
        }
        let mut file = File::create(Self::FILE)?;
        file.write_all(data.as_bytes())
    }

    fn get(&self, bits: &str) -> u32 {
        *self.scores.get(bits).unwrap_or(&0)
    }

    fn update(&mut self, bits: &str, score: u32) {
        self.scores.insert(bits.to_string(), score);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyEventKind, KeyEventState, KeyModifiers};
    use std::fs;
    use std::sync::Mutex;

    static HS_LOCK: Mutex<()> = Mutex::new(());

    fn with_high_score_file<F: FnOnce()>(f: F) {
        #[allow(clippy::expect_used)]
        let _guard = HS_LOCK.lock().expect("Failed to lock high score mutex");
        let original = fs::read_to_string(HighScores::FILE).ok();
        f();
        // restore
        match original {
            Some(data) => {
                let _ = fs::write(HighScores::FILE, data);
            },
            None => {
                let _ = fs::remove_file(HighScores::FILE);
            },
        }
    }

    #[test]
    fn bits_properties() {
        assert_eq!(Bits::Four.to_int(), 4);
        assert_eq!(Bits::Four.upper_bound(), 15);
        assert_eq!(Bits::Four.suggestion_count(), 3);

        assert_eq!(Bits::FourShift4.scale_factor(), 16);
        assert_eq!(Bits::FourShift4.upper_bound(), 240);
        assert_eq!(Bits::FourShift4.suggestion_count(), 3);

        assert_eq!(Bits::FourShift8.scale_factor(), 256);
        assert_eq!(Bits::FourShift12.high_score_key(), 412);
        assert_eq!(Bits::Eight.upper_bound(), 255);

        assert_eq!(Bits::Sixteen.suggestion_count(), 6);
    }

    #[test]
    fn puzzle_generation_unique_and_scaled() {
        let p = BinaryNumbersPuzzle::new(Bits::FourShift4.clone(), NumberMode::Unsigned, 0);
        let scale = Bits::FourShift4.scale_factor();
        assert_eq!(p.suggestions().len(), Bits::FourShift4.suggestion_count());
        // uniqueness
        let mut sorted = p.suggestions().to_vec();
        sorted.sort_unstable();
        for pair in sorted.windows(2) {
            assert_ne!(pair[0], pair[1]);
        }
        // scaling property
        for &s in p.suggestions() {
            assert_eq!(s.unsigned_abs() % scale, 0);
        }
        // correct answer must be one of suggestions and raw_current_number * scale == correct_answer (unsigned)
        assert!(p.suggestions().contains(&p.correct_answer));
        assert_eq!(p.raw_current_number * scale, p.correct_answer.unsigned_abs());
    }

    #[test]
    fn binary_string_formatting_groups_every_four_bits() {
        let mut p = BinaryNumbersPuzzle::new(Bits::Eight, NumberMode::Unsigned, 0);
        p.raw_current_number = 0xAB; // 171 = 10101011
        assert_eq!(p.current_to_binary_string(), "1010 1011");
        let mut p4 = BinaryNumbersPuzzle::new(Bits::Four, NumberMode::Unsigned, 0);
        p4.raw_current_number = 0b0101;
        assert_eq!(p4.current_to_binary_string(), "0101");
    }

    #[test]
    fn signed_mode_negative_numbers_show_sign_bit() {
        // Test 4-bit signed mode with a negative number
        let mut p = BinaryNumbersPuzzle::new(Bits::Four, NumberMode::Signed, 0);
        // In 4-bit two's complement, -8 is represented as 1000
        p.raw_current_number = 0b1000; // -8 in 4-bit two's complement
        assert_eq!(p.current_to_binary_string(), "1000", "4-bit: -8 should be 1000");

        // In 4-bit two's complement, -1 is represented as 1111
        p.raw_current_number = 0b1111; // -1 in 4-bit two's complement
        assert_eq!(p.current_to_binary_string(), "1111", "4-bit: -1 should be 1111");

        // Test 8-bit signed mode with a negative number
        let mut p8 = BinaryNumbersPuzzle::new(Bits::Eight, NumberMode::Signed, 0);
        // In 8-bit two's complement, -128 is represented as 10000000
        p8.raw_current_number = 0b10000000; // -128 in 8-bit two's complement
        assert_eq!(p8.current_to_binary_string(), "1000 0000", "8-bit: -128 should be 1000 0000");

        // In 8-bit two's complement, -1 is represented as 11111111
        p8.raw_current_number = 0b11111111; // -1 in 8-bit two's complement
        assert_eq!(p8.current_to_binary_string(), "1111 1111", "8-bit: -1 should be 1111 1111");
    }

    #[test]
    fn signed_mode_puzzle_generates_correct_raw_bits_for_negative() {
        // Generate many puzzles and check that when we have a negative number,
        // the raw_current_number has the sign bit set correctly
        for _ in 0..20 {
            let p = BinaryNumbersPuzzle::new(Bits::Four, NumberMode::Signed, 0);
            let current_signed = p.correct_answer;

            if current_signed < 0 {
                // For negative numbers in 4-bit two's complement, the MSB (bit 3) should be 1
                // which means raw_current_number should be >= 8 (0b1000)
                assert!(
                    p.raw_current_number >= 8,
                    "Negative number {} should have raw bits >= 8 (sign bit set), but got {}. Binary: {}",
                    current_signed,
                    p.raw_current_number,
                    p.current_to_binary_string()
                );
            } else {
                // For positive numbers (including 0), MSB should be 0
                // which means raw_current_number should be < 8
                assert!(
                    p.raw_current_number < 8,
                    "Positive number {} should have raw bits < 8 (sign bit clear), but got {}. Binary: {}",
                    current_signed,
                    p.raw_current_number,
                    p.current_to_binary_string()
                );
            }
        }
    }

    #[test]
    fn puzzle_timeout_sets_guess_result() {
        let mut p = BinaryNumbersPuzzle::new(Bits::Four, NumberMode::Unsigned, 0);
        p.time_left = 0.5;
        // First run() skips dt due to skip_first_dt flag
        // The reason for this is to prevent timer jump when starting a new puzzle
        p.run(1.0);
        assert_eq!(p.guess_result, None, "First run should skip dt");
        // Second run() actually applies the dt and triggers timeout
        p.run(1.0); // exceed remaining time
        assert_eq!(p.guess_result, Some(GuessResult::Timeout));
    }

    #[test]
    fn suggestions_are_randomized() {
        // Verify that suggestions are properly randomized and the first suggestion
        // is not always the correct answer
        let mut first_is_correct_count = 0;
        let trials = 100;

        for _ in 0..trials {
            let p = BinaryNumbersPuzzle::new(Bits::Four, NumberMode::Unsigned, 0);
            // Check if the first suggestion happens to be the correct answer
            if p.suggestions[0] == p.correct_answer {
                first_is_correct_count += 1;
            }
        }

        // With 3 suggestions for 4-bit mode, we expect roughly 33% to be correct by chance
        // Allow a range of 20-50% (which is generous for 100 trials to account for randomness)
        // The key point is that it's NOT 100% (which would indicate no randomization)
        assert!(
            first_is_correct_count >= 20 && first_is_correct_count <= 50,
            "First suggestion was correct {} times out of {}, expected around 33% (20-50 range). \
             If this is close to 100%, suggestions are not randomized!",
            first_is_correct_count,
            trials
        );

        // Also verify that the correct answer is actually one of the suggestions
        for _ in 0..10 {
            let p = BinaryNumbersPuzzle::new(Bits::Eight, NumberMode::Signed, 0);
            assert!(
                p.suggestions.contains(&p.correct_answer),
                "correct_answer {} must be in suggestions {:?}",
                p.correct_answer,
                p.suggestions
            );
        }
    }

    #[test]
    fn finalize_round_correct_increments_score_streak_and_sets_result_state() {
        with_high_score_file(|| {
            let mut g = BinaryNumbersGame::new(Bits::Four, NumberMode::Unsigned);
            // ensure deterministic: mark puzzle correct
            let answer = g.puzzle.correct_answer;
            g.puzzle.guess_result = Some(GuessResult::Correct);
            g.finalize_round();
            assert_eq!(g.streak, 1);
            assert_eq!(g.score, 10); // base points
            assert_eq!(g.puzzle.last_points_awarded, 10);
            assert_eq!(g.game_state, GameState::Result);
            assert!(g.puzzle_resolved);
            assert!(g.puzzle.is_correct_guess(answer));
        });
    }

    #[test]
    fn life_awarded_every_five_streak() {
        with_high_score_file(|| {
            let mut g = BinaryNumbersGame::new_with_max_lives(Bits::Four, NumberMode::Unsigned, 3);
            g.lives = 2; // below max
            g.streak = 4; // about to become 5
            g.puzzle.guess_result = Some(GuessResult::Correct);
            g.finalize_round();
            assert_eq!(g.streak, 5);
            assert_eq!(g.lives, 3); // gained life
        });
    }

    #[test]
    fn incorrect_guess_resets_streak_and_loses_life() {
        with_high_score_file(|| {
            let mut g = BinaryNumbersGame::new(Bits::Four, NumberMode::Unsigned);
            g.streak = 3;
            let lives_before = g.lives;
            g.puzzle.guess_result = Some(GuessResult::Incorrect);
            g.finalize_round();
            assert_eq!(g.streak, 0);
            assert_eq!(g.lives, lives_before - 1);
        });
    }

    #[test]
    fn pending_game_over_when_life_reaches_zero() {
        with_high_score_file(|| {
            let mut g = BinaryNumbersGame::new(Bits::Four, NumberMode::Unsigned);
            g.lives = 1;
            g.puzzle.guess_result = Some(GuessResult::Incorrect);
            g.finalize_round();
            assert_eq!(g.lives, 0);
            assert_eq!(g.game_state, GameState::PendingGameOver);
        });
    }

    #[test]
    fn high_score_updates_and_flag_set() {
        with_high_score_file(|| {
            let mut g = BinaryNumbersGame::new(Bits::Four, NumberMode::Unsigned);
            // Force previous high score low
            let key = BinaryNumbersGame::compute_high_score_key(&g.bits, g.number_mode);
            g.high_scores.update(&key, 5);
            g.prev_high_score_for_display = 5;
            g.puzzle.guess_result = Some(GuessResult::Correct);
            g.finalize_round();
            assert!(g.new_high_score_reached);
            assert!(g.high_scores.get(&key) >= 10);
            assert_eq!(g.prev_high_score_for_display, 5); // previous stored
        });
    }

    #[test]
    fn hearts_representation_matches_lives() {
        let mut g = BinaryNumbersGame::new_with_max_lives(Bits::Four, NumberMode::Unsigned, 3);
        g.lives = 2;
        assert_eq!(g.lives_hearts(), "♥♥·");
    }

    #[test]
    fn handle_input_navigation_changes_selected_suggestion() {
        let mut g = BinaryNumbersGame::new(Bits::Four, NumberMode::Unsigned);
        let initial = g.puzzle.selected_suggestion;
        // Simulate Right key
        let right_event = KeyEvent {
            code: KeyCode::Right,
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };
        g.handle_game_input(right_event);
        assert_ne!(g.puzzle.selected_suggestion, initial);
        // Simulate Left key should cycle back
        let left_event = KeyEvent {
            code: KeyCode::Left,
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };
        g.handle_game_input(left_event);
        assert!(g.puzzle.selected_suggestion.is_some());
    }
}
