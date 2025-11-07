use crate::main_screen_widget::{MainScreenWidget, WidgetRef};
use crate::utils::{center, When};
use crossterm::event::{KeyCode, KeyEvent};
use rand::prelude::SliceRandom;
use rand::Rng;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Flex, Layout, Rect};
use ratatui::prelude::Alignment::Center;
use ratatui::prelude::{Color, Line, Style, Stylize, Widget};
use ratatui::style::Modifier; // added for bold high score marker
use ratatui::text::Span;
use ratatui::widgets::BorderType::Double;
use ratatui::widgets::{Block, BorderType, Paragraph};
use std::collections::HashMap;
use std::fs::{File};
use std::io::{Read, Write};

struct StatsSnapshot {
    score: u32,
    streak: u32,
    max_streak: u32,
    rounds: u32,
    lives: u32,
    bits: Bits,
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
        let [middle] = Layout::horizontal([Constraint::Percentage(100)])
            .flex(Flex::Center)
            .areas(area);

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

        Block::bordered()
            .title_alignment(Center)
            .dark_gray()
            .render(stats_area, buf);

        if let Some(stats) = &self.stats_snapshot {
            let high_label = if stats.new_high_score {
                let style = Style::default().fg(Color::LightGreen).add_modifier(Modifier::BOLD);
                Span::styled(format!("Hi-Score: {}*  ", stats.score), style)
            } else {
                let style = Style::default().fg(Color::DarkGray);
                Span::styled(format!("Hi-Score: {}  ", stats.prev_high_score), style)
            };

            let line1 = Line::from(vec![
                Span::styled(format!("Mode: {}  ", stats.bits.label()), Style::default().fg(Color::Yellow)),
                high_label,
            ]);

            let line2 = Line::from(vec![
                Span::styled(format!("Score: {}  ", stats.score), Style::default().fg(Color::Green)),
                Span::styled(format!("Streak: {}  ", stats.streak), Style::default().fg(Color::Cyan)),
                Span::styled(format!("Max: {}  ", stats.max_streak), Style::default().fg(Color::Blue)),
                Span::styled(format!("Rounds: {}  ", stats.rounds), Style::default().fg(Color::Magenta)),
                Span::styled(format!("Lives: {}  ", stats.hearts), Style::default().fg(Color::Red)),
            ]);

            let widest = line1.width().max(line2.width()) as u16;
            Paragraph::new(vec![line1, line2])
                .alignment(Center)
                .render(center(stats_area, Constraint::Length(widest)), buf);

            // If game over, render game over block occupying the remaining area and return early
            if stats.game_state == GameState::GameOver {
                let combined_rect = Rect { x: current_number_area.x, y: current_number_area.y, width: current_number_area.width, height: current_number_area.height + suggestions_area.height + progress_bar_area.height + result_area.height };
                let block = Block::bordered()
                    .title("Game Over")
                    .title_alignment(Center)
                    .border_type(Double)
                    .title_style(Style::default().fg(Color::Red));
                block.render(combined_rect, buf);
                let mut lines = vec![
                    Line::from(Span::styled(format!("Final Score: {}", stats.score), Style::default().fg(Color::Green))),
                    Line::from(Span::styled(format!("Previous High: {}", stats.prev_high_score), Style::default().fg(Color::Yellow))),
                    Line::from(Span::styled(format!("Rounds Played: {}", stats.rounds), Style::default().fg(Color::Magenta))),
                    Line::from(Span::styled(format!("Max Streak: {}", stats.max_streak), Style::default().fg(Color::Cyan))),
                ];
                if stats.new_high_score {
                    lines.insert(1, Line::from(Span::styled("NEW HIGH SCORE!", Style::default().fg(Color::LightGreen).bold())));
                }
                if stats.lives == 0 {
                    lines.push(Line::from(Span::styled("You lost all your lives.", Style::default().fg(Color::Red))));
                }
                lines.push(Line::from(Span::styled("Press Enter to restart or Esc to exit", Style::default().fg(Color::Yellow))));
                Paragraph::new(lines)
                    .alignment(Center)
                    .render(center(combined_rect, Constraint::Length(48)), buf);
                return;
            }
        }

        // Existing puzzle rendering now uses updated area references
        let [inner] = Layout::horizontal([Constraint::Percentage(100)])
            .flex(Flex::Center)
            .areas(current_number_area);

        Block::bordered()
            .border_type(Double)
            .border_style(Style::default().dark_gray())
            .render(inner, buf);

        let binary_string = self.current_to_binary_string();
        let scale_suffix = match self.bits { Bits::FourShift4 => Some(" x16"), Bits::FourShift8 => Some(" x256"), Bits::FourShift12 => Some(" x4096"), _ => None };
        let mut spans = vec![Span::raw(binary_string.clone())];
        if let Some(sfx) = scale_suffix { spans.push(Span::styled(sfx, Style::default().fg(Color::DarkGray))); }
        let total_width = spans.iter().map(|s| s.width()).sum::<usize>() as u16;
        let lines: Vec<Line> = vec![Line::from(spans)];
        Paragraph::new(lines).alignment(Center).render(center(inner, Constraint::Length(total_width)), buf);

        let suggestions = self.suggestions();
        let suggestions_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Min(6); suggestions.len()])
            .split(suggestions_area);
        for (i, suggestion) in suggestions.iter().enumerate() {
            let item_is_selected = self.selected_suggestion == Some(*suggestion);
            let show_correct_number = self.guess_result.is_some();
            let is_correct_number = self.is_correct_guess(*suggestion);
            let area = suggestions_layout[i];

            let border_type = if item_is_selected { BorderType::Double } else { BorderType::Plain };

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
            Paragraph::new(format!("{}", suggestion_str))
                .white()
                .when(show_correct_number && is_correct_number, |p| p.light_green().underlined())
                .alignment(Center)
                .render(center(area, Constraint::Length(suggestion_str.len() as u16)), buf);
        }

        let [left, right] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .areas(progress_bar_area);

        Block::bordered().dark_gray().title("Status").title_alignment(Center).title_style(Style::default().white()).render(left, buf);

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
                Line::from(format!("{} {}", icon, line1_text).fg(color)),
                Line::from(gained_line.fg(color)),
            ];
            let widest = text.iter().map(|l| l.width()).max().unwrap_or(0) as u16;
            Paragraph::new(text)
                .alignment(Center)
                .style(Style::default().fg(color))
                .render(center(left, Constraint::Length(widest)), buf);
        }

        let ratio = self.time_left / self.time_total;
        let gauge_color = if ratio > 0.6 {
            Color::Green
        } else if ratio > 0.3 {
            Color::Yellow
        } else {
            Color::Red
        };

        // Replace previous split layout: keep everything inside a single bordered block and remove percent label
        let time_block = Block::bordered()
            .dark_gray()
            .title("Time Remaining")
            .title_style(Style::default().white())
            .title_alignment(Center);
        let inner_time = time_block.inner(right);
        time_block.render(right, buf);

        // Vertical layout inside the time block interior: gauge line + text line (2 lines total)
        let [gauge_line, time_line] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1)
        ]).areas(inner_time);

        render_ascii_gauge(gauge_line, buf, ratio, gauge_color);

        Paragraph::new(Line::from(Span::styled(
            format!("{:.2} seconds left", self.time_left),
            Style::default().fg(gauge_color),
        )))
        .alignment(Center)
        .render(time_line, buf);

        Block::bordered().dark_gray().render(result_area, buf);

        let instruction_spans: Vec<Span> = vec![
            hotkey_span("Left Right", "select  "),
            hotkey_span("Enter", "confirm  "),
            hotkey_span("S", "skip  "),
            hotkey_span("Esc", "exit"),
        ].iter().flatten().cloned().collect();

        Paragraph::new(vec![Line::from(instruction_spans)])
            .alignment(Center)
            .render(center(result_area, Constraint::Length(65)), buf);
    }
}

fn hotkey_span<'a>(key: &'a str, description: &str) -> Vec<Span<'a>> {
    vec![
        Span::styled("<", Style::default().fg(Color::White)),
        Span::styled(key, Style::default().fg(Color::LightCyan)),
        Span::styled(format!("> {}", description), Style::default().fg(Color::White)),
    ]
}

pub struct BinaryNumbersGame {
    puzzle: BinaryNumbersPuzzle,
    bits: Bits,
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

#[derive(Copy, Clone, PartialEq)]
enum GameState { Active, Result, PendingGameOver, GameOver }

impl MainScreenWidget for BinaryNumbersGame {
    fn run(&mut self, dt: f64) {
        self.refresh_stats_snapshot();
        if self.game_state == GameState::GameOver { return; }
        self.puzzle.run(dt);
        if self.puzzle.guess_result.is_some() && !self.puzzle_resolved { self.finalize_round(); }
        self.refresh_stats_snapshot();
    }

    fn handle_input(&mut self, input: KeyEvent) -> () { self.handle_game_input(input); }
    fn is_exit_intended(&self) -> bool { self.exit_intended }
}

impl BinaryNumbersGame {
    pub fn new(bits: Bits) -> Self { Self::new_with_max_lives(bits, 3) }
    pub fn new_with_max_lives(bits: Bits, max_lives: u32) -> Self {
        let hs = HighScores::load();
        let starting_prev = hs.get(bits.high_score_key());
        Self {
            bits: bits.clone(),
            puzzle: Self::init_puzzle(bits.clone(), 0),
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
        }
    }

    pub fn init_puzzle(bits: Bits, streak: u32) -> BinaryNumbersPuzzle {
        BinaryNumbersPuzzle::new(bits, streak)
    }
}

impl BinaryNumbersGame {
    pub fn lives_hearts(&self) -> String {
        let full_count = self.lives.min(self.max_lives) as usize;
        let full = "♥".repeat(full_count);
        let empty_count = self.max_lives.saturating_sub(self.lives) as usize;
        let empty = "·".repeat(empty_count);
        format!("{}{}", full, empty)
    }

    fn finalize_round(&mut self) {
        if let Some(result) = self.puzzle.guess_result {
            self.rounds += 1;
            match result {
                GuessResult::Correct => {
                    self.streak += 1;
                    if self.streak > self.max_streak { self.max_streak = self.streak; }
                    let streak_bonus = (self.streak - 1) * 2;
                    let points = 10 + streak_bonus;
                    self.score += points;
                    self.puzzle.last_points_awarded = points;
                    if self.streak % 5 == 0 && self.lives < self.max_lives { self.lives += 1; }
                }
                GuessResult::Incorrect | GuessResult::Timeout => {
                    self.streak = 0;
                    self.puzzle.last_points_awarded = 0;
                    if self.lives > 0 { self.lives -= 1; }
                }
            }
            // high score update
            let bits_key = self.bits.high_score_key();
            let prev = self.high_scores.get(bits_key);
            if self.score > prev {
                if !self.new_high_score_reached { self.prev_high_score_for_display = prev; }
                self.high_scores.update(bits_key, self.score);
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
        if input.code == KeyCode::Esc { self.exit_intended = true; return; }
        if self.game_state == GameState::GameOver { self.handle_game_over_input(input); return; }
        match self.puzzle.guess_result {
            None => self.handle_no_result_yet(input),
            Some(_) => self.handle_result_available(input),
        }
    }

    fn handle_game_over_input(&mut self, input: KeyEvent) {
        match input.code {
            KeyCode::Enter => { self.reset_game_state(); }
            KeyCode::Esc => { self.exit_intended = true; }
            _ => {}
        }
    }

    fn reset_game_state(&mut self) {
        self.score = 0;
        self.streak = 0;
        self.rounds = 0;
        self.lives = self.max_lives.min(3);
        self.game_state = GameState::Active;
        self.max_streak = 0;
        self.prev_high_score_for_display = self.high_scores.get(self.bits.high_score_key());
        self.new_high_score_reached = false;
        self.puzzle = Self::init_puzzle(self.bits.clone(), 0);
        self.puzzle_resolved = false;
        self.refresh_stats_snapshot();
    }

    fn handle_no_result_yet(&mut self, input: KeyEvent) {
        match input.code {
            KeyCode::Right => {
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
            }
            KeyCode::Left => {
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
            }
            KeyCode::Enter => {
                if let Some(selected) = self.puzzle.selected_suggestion {
                    if self.puzzle.is_correct_guess(selected) {
                        self.puzzle.guess_result = Some(GuessResult::Correct);
                    } else {
                        self.puzzle.guess_result = Some(GuessResult::Incorrect);
                    }
                    self.finalize_round();
                }
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                // Skip puzzle counts as timeout
                self.puzzle.guess_result = Some(GuessResult::Timeout);
                self.finalize_round();
            }
            _ => {}
        }
    }

    fn handle_result_available(&mut self, input: KeyEvent) {
        match input.code {
            KeyCode::Enter => {
                match self.game_state {
                    GameState::PendingGameOver => {
                        // reveal summary
                        self.game_state = GameState::GameOver;
                    }
                    GameState::Result => {
                        // start next puzzle
                        self.puzzle = Self::init_puzzle(self.bits.clone(), self.streak);
                        self.puzzle_resolved = false;
                        self.game_state = GameState::Active;
                    }
                    GameState::GameOver => { /* handled elsewhere */ }
                    GameState::Active => { /* shouldn't be here */ }
                }
            }
            KeyCode::Esc => self.exit_intended = true,
            _ => {}
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
            hearts: self.lives_hearts(),
            game_state: self.game_state,
            prev_high_score: self.prev_high_score_for_display,
            new_high_score: self.new_high_score_reached,
        });
    }
}

#[derive(PartialEq, Copy, Clone)]
enum GuessResult {
    Correct,
    Incorrect,
    Timeout,
}

#[derive(Clone)]
pub enum Bits { Four, FourShift4, FourShift8, FourShift12, Eight, Twelve, Sixteen, }

impl Bits {
    pub fn to_int(&self) -> u32 { match self { Bits::Four | Bits::FourShift4 | Bits::FourShift8 | Bits::FourShift12 => 4, Bits::Eight => 8, Bits::Twelve => 12, Bits::Sixteen => 16 } }
    pub fn scale_factor(&self) -> u32 { match self { Bits::Four => 1, Bits::FourShift4 => 16, Bits::FourShift8 => 256, Bits::FourShift12 => 4096, Bits::Eight => 1, Bits::Twelve => 1, Bits::Sixteen => 1 } }
    pub fn high_score_key(&self) -> u32 { match self { Bits::Four => 4, Bits::FourShift4 => 44, Bits::FourShift8 => 48, Bits::FourShift12 => 412, Bits::Eight => 8, Bits::Twelve => 12, Bits::Sixteen => 16 } }
    pub fn upper_bound(&self) -> u32 { (u32::pow(2, self.to_int()) - 1) * self.scale_factor() }
    pub fn suggestion_count(&self) -> usize { match self { Bits::Four | Bits::FourShift4 | Bits::FourShift8 | Bits::FourShift12 => 3, Bits::Eight => 4, Bits::Twelve => 5, Bits::Sixteen => 6 } }
    pub fn label(&self) -> &'static str { match self { Bits::Four => "4 bits", Bits::FourShift4 => "4 bits*16", Bits::FourShift8 => "4 bits*256", Bits::FourShift12 => "4 bits*4096", Bits::Eight => "8 bits", Bits::Twelve => "12 bits", Bits::Sixteen => "16 bits" } }
}

pub struct BinaryNumbersPuzzle {
    bits: Bits,
    current_number: u32, // scaled value used for suggestions matching
    raw_current_number: u32, // raw bit value (unscaled) for display
    suggestions: Vec<u32>,
    selected_suggestion: Option<u32>,
    time_total: f64,
    time_left: f64,
    guess_result: Option<GuessResult>,
    last_points_awarded: u32,
    stats_snapshot: Option<StatsSnapshot>,
}

impl BinaryNumbersPuzzle {
    pub fn new(bits: Bits, streak: u32) -> Self {
        let mut rng = rand::rng();

        let mut suggestions = Vec::new();
        let scale = bits.scale_factor();
        while suggestions.len() < bits.suggestion_count() {
            let raw = rng.random_range(0..=u32::pow(2, bits.to_int()) - 1);
            let num = raw * scale;
            if !suggestions.contains(&num) { suggestions.push(num); }
        }

        let current_number = suggestions[0]; // scaled value
        let raw_current_number = current_number / scale; // back-calculate raw bits
        suggestions.shuffle(&mut rng);

        // Base time by bits + difficulty scaling (shorter as streak increases)
        let base_time = match bits {
            Bits::Four | Bits::FourShift4 | Bits::FourShift8 | Bits::FourShift12 => 8.0,
            Bits::Eight => 12.0,
            Bits::Twelve => 16.0,
            Bits::Sixteen => 20.0,
        };
        let penalty = (streak as f64) * 0.5; // 0.5s less per streak
        let time_total = (base_time - penalty).max(5.0);
        let time_left = time_total;
        let selected_suggestion = Some(suggestions[0]);
        let guess_result = None;
        let last_points_awarded = 0;

        Self {
            bits,
            current_number,
            raw_current_number,
            suggestions,
            time_total,
            time_left,
            selected_suggestion,
            guess_result,
            last_points_awarded,
            stats_snapshot: None,
        }
    }

    pub fn suggestions(&self) -> &[u32] { &self.suggestions }
    pub fn is_correct_guess(&self, guess: u32) -> bool { guess == self.current_number }

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
        if self.guess_result.is_some() {
            // If a guess has been made, we don't need to run the game logic anymore.
            return;
        }

        self.time_left = (self.time_left - dt).max(0.0);

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
    let clamped = if ratio < 0.0 { 0.0 } else if ratio > 1.0 { 1.0 } else { ratio };
    let fill_width = ((area.width as f64) * clamped).round().min(area.width as f64) as u16;
    if area.height == 0 { return; }
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

// NEW: HighScores management
struct HighScores { scores: HashMap<u32, u32>, }

impl HighScores {
    const FILE: &'static str = "binbreak_highscores.txt";

    fn empty() -> Self { Self { scores: HashMap::new() } }

    fn load() -> Self {
        let mut hs = Self::empty();
        if let Ok(mut file) = File::open(Self::FILE) {
            let mut contents = String::new();
            if file.read_to_string(&mut contents).is_ok() {
                for line in contents.lines() {
                    if let Some((k,v)) = line.split_once('=') {
                        if let (Ok(bits), Ok(score)) = (k.trim().parse::<u32>(), v.trim().parse::<u32>()) {
                            hs.scores.insert(bits, score);
                        }
                    }
                }
            }
        }
        hs
    }

    fn save(&self) -> std::io::Result<()> {
        let mut data = String::new();
        for key in [4u32,44u32,48u32,412u32,8u32,12u32,16u32] {
            let val = self.get(key);
            data.push_str(&format!("{}={}\n", key, val));
        }
        let mut file = File::create(Self::FILE)?;
        file.write_all(data.as_bytes())
    }

    fn get(&self, bits: u32) -> u32 {
        *self.scores.get(&bits).unwrap_or(&0)
    }

    fn update(&mut self, bits: u32, score: u32) {
        self.scores.insert(bits, score);
    }
}
