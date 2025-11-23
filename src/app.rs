use crate::binary_numbers::{BinaryNumbersGame, Bits};
use crate::keybinds;
use crate::main_screen_widget::MainScreenWidget;
use crate::utils::{ProceduralAnimationWidget};
use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use indoc::indoc;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::{Color, Modifier, Span, Style, Widget};
use ratatui::widgets::{List, ListItem, ListState};
use std::cmp;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant};

static LAST_SELECTED_INDEX: AtomicUsize = AtomicUsize::new(4);

fn get_last_selected_index() -> usize {
    LAST_SELECTED_INDEX.load(Ordering::Relaxed)
}

fn set_last_selected_index(index: usize) {
    LAST_SELECTED_INDEX.store(index, Ordering::Relaxed);
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum FpsMode {
    RealTime,    // 30 FPS with polling
    Performance, // Block until input for minimal CPU
}

enum AppState {
    Start(StartMenuState),
    Playing(BinaryNumbersGame),
    Exit,
}

fn handle_start_input(state: &mut StartMenuState, key: KeyEvent) -> Option<AppState> {
    match key {
        x if keybinds::is_up(x) => state.select_previous(),
        x if keybinds::is_down(x) => state.select_next(),
        x if keybinds::is_select(x) => {
            let bits = state.selected_bits();
            // Store the current selection before entering the game
            set_last_selected_index(state.selected_index());
            return Some(AppState::Playing(BinaryNumbersGame::new(bits)));
        },
        x if keybinds::is_exit(x) => return Some(AppState::Exit),
        KeyEvent { code: KeyCode::Char('a' | 'A'), .. } => state.toggle_animation(),
        _ => {},
    }
    None
}


fn render_start_screen(state: &mut StartMenuState, area: Rect, buf: &mut Buffer) {
    // Get animation dimensions
    let ascii_width = state.animation.get_width();
    let ascii_height = state.animation.get_height();

    let selected = state.selected_index();
    let upper_labels: Vec<String> = state.items.iter().map(|(l, _)| l.to_uppercase()).collect();
    #[allow(clippy::cast_possible_truncation)]
    let max_len = upper_labels.iter().map(|s| s.len() as u16).max().unwrap_or(0);

    let list_width = 2 + max_len; // marker + space + label
    #[allow(clippy::cast_possible_truncation)]
    let list_height = upper_labels.len() as u16;

    // Vertical spacing between ASCII art and list
    let spacing: u16 = 3;
    let total_height = ascii_height + spacing + list_height;

    // Center vertically & horizontally
    let start_y = area.y + area.height.saturating_sub(total_height) / 2;
    let ascii_x = area.x + area.width.saturating_sub(ascii_width) / 2;
    let list_x = area.x + area.width.saturating_sub(list_width) / 2;
    let ascii_y = start_y;
    let list_y = ascii_y + ascii_height + spacing;

    // Define rects (clamp to area)
    let ascii_area =
        Rect::new(ascii_x, ascii_y, ascii_width.min(area.width), ascii_height.min(area.height));
    let list_area = Rect::new(
        list_x,
        list_y,
        list_width.min(area.width),
        list_height.min(area.height.saturating_sub(list_y - area.y)),
    );

    // Render ASCII animation (handles paused state internally)
    state.animation.render_to_buffer(ascii_area, buf);

    // Palette for menu flair
    let palette = [
        Color::LightGreen,
        Color::LightCyan,
        Color::LightBlue,
        Color::LightMagenta,
        Color::LightYellow,
        Color::LightRed,
    ];

    let items: Vec<ListItem> = upper_labels
        .into_iter()
        .enumerate()
        .map(|(i, label)| {
            let marker = if i == selected { '»' } else { ' ' };
            let padded = format!("{:<width$}", label, width = max_len as usize);
            let line = format!("{marker} {padded}");
            let style =
                Style::default().fg(palette[i % palette.len()]).add_modifier(Modifier::BOLD);
            ListItem::new(Span::styled(line, style))
        })
        .collect();

    let list = List::new(items);
    ratatui::widgets::StatefulWidget::render(list, list_area, buf, &mut state.list_state);
}

fn handle_crossterm_events(app_state: &mut AppState) -> color_eyre::Result<()> {
    if let Event::Key(key) = event::read()?
        && key.kind == KeyEventKind::Press
    {
        match key.code {
            // global exit via Ctrl+C
            KeyCode::Char('c' | 'C') if key.modifiers == KeyModifiers::CONTROL => {
                *app_state = AppState::Exit;
            },

            // state-specific input handling
            _ => {
                *app_state = match std::mem::replace(app_state, AppState::Exit) {
                    AppState::Start(mut menu) => {
                        handle_start_input(&mut menu, key).unwrap_or(AppState::Start(menu))
                    },
                    AppState::Playing(mut game) => {
                        game.handle_input(key);
                        AppState::Playing(game)
                    },
                    AppState::Exit => AppState::Exit,
                }
            },
        }
    }
    Ok(())
}

/// Determine the appropriate FPS mode based on the current game state
fn get_fps_mode(game: &BinaryNumbersGame) -> FpsMode {
    if game.is_active() {
        FpsMode::RealTime // Timer running, needs continuous updates
    } else {
        FpsMode::Performance // All other cases, block for minimal CPU
    }
}

pub fn run_app(terminal: &mut ratatui::DefaultTerminal) -> color_eyre::Result<()> {
    let mut app_state = AppState::Start(StartMenuState::new());
    let mut last_frame_time = Instant::now();
    let target_frame_duration = std::time::Duration::from_millis(33); // ~30 FPS

    while !matches!(app_state, AppState::Exit) {
        let now = Instant::now();
        let dt = now - last_frame_time;
        last_frame_time = now;

        // Advance game BEFORE drawing so stats are updated
        if let AppState::Playing(game) = &mut app_state {
            game.run(dt.as_secs_f64());
            if game.is_exit_intended() {
                app_state = AppState::Start(StartMenuState::new());
                continue;
            }
        }

        terminal.draw(|f| match &mut app_state {
            AppState::Start(menu) => render_start_screen(menu, f.area(), f.buffer_mut()),
            AppState::Playing(game) => f.render_widget(&mut *game, f.area()),
            AppState::Exit => {},
        })?;

        // handle input
        if let AppState::Playing(game) = &app_state {
            if get_fps_mode(game) == FpsMode::RealTime {
                let poll_timeout = cmp::min(dt, target_frame_duration);
                if event::poll(poll_timeout)? {
                    handle_crossterm_events(&mut app_state)?;
                }
            } else {
                // performance mode: block thread until an input event occurs
                handle_crossterm_events(&mut app_state)?;
            }
        } else if let AppState::Start(menu) = &app_state {
            // For start menu, use real-time mode only if animation is running
            if !menu.animation.is_paused() {
                let poll_timeout = cmp::min(dt, target_frame_duration);
                if event::poll(poll_timeout)? {
                    handle_crossterm_events(&mut app_state)?;
                }
            } else {
                // Animation paused, use performance mode to save CPU
                handle_crossterm_events(&mut app_state)?;
            }
        }

        // cap frame rate
        let frame_duration = last_frame_time.elapsed();
        if frame_duration < target_frame_duration {
            thread::sleep(target_frame_duration - frame_duration);
        }
    }
    Ok(())
}

fn ascii_animation() -> ProceduralAnimationWidget {
    let art = indoc! {r#"
         ,,        ,,              ,,
        *MM        db             *MM      [a: toggle animation]     `7MM
         MM                        MM                                  MM
         MM,dMMb.`7MM  `7MMpMMMb.  MM,dMMb.`7Mb,od8 .gP"Ya   ,6"Yb.    MM  ,MP'
         MM    `Mb MM    MM    MM  MM    `Mb MM' "',M'   Yb 8)   MM    MM ;Y
         MM     M8 MM    MM    MM  MM     M8 MM    8M""""""  ,pm9MM    MM;Mm
         MM.   ,M9 MM    MM    MM  MM.   ,M9 MM    YM.    , 8M   MM    MM `Mb.
         P^YbmdP'.JMML..JMML  JMML.P^YbmdP'.JMML.   `Mbmmd' `Moo9^Yo..JMML. YA.
    "#}.to_string();

    // Get dimensions for calculations
    let art_lines: Vec<&str> = art.lines().collect();
    let height = art_lines.len();
    let width = art_lines.iter().map(|line| line.len()).max().unwrap_or(0);

    let strip_width = 8.0;
    let start_offset = -strip_width;
    let end_offset = (width + height) as f32 + strip_width;
    let total_range = end_offset - start_offset;

    // Color function that calculates colors on-the-fly based on animation progress
    let color_fn = move |x: usize, y: usize, progress: f32| -> Color {
        let offset = start_offset + progress * total_range;
        let diag_pos = (x + y) as f32;
        let dist_from_strip = (diag_pos - offset).abs();

        if dist_from_strip < strip_width {
            Color::LightGreen
        } else {
            Color::DarkGray
        }
    };

    // Character function that replaces characters in the green strip with static '0' or '1'
    let char_fn = move |x: usize, y: usize, progress: f32, original_char: char| -> char {
        let offset = start_offset + progress * total_range;
        let diag_pos = (x + y) as f32;
        let dist_from_strip = (diag_pos - offset).abs();

        if dist_from_strip < strip_width {
            // Use a hash function based only on position (no frame/progress)
            // This creates a static pattern that doesn't change
            let mut hash = x.wrapping_mul(2654435761);
            hash ^= y.wrapping_mul(2246822519);
            hash = hash.wrapping_mul(668265263);
            hash ^= hash >> 15;

            if (hash & 1) == 0 { '0' } else { '1' }
        } else {
            original_char
        }
    };

    ProceduralAnimationWidget::new(
        art,
        50, // 50 frames worth of timing
        Duration::from_millis(50),
        color_fn,
    )
    .with_char_fn(char_fn)
    .with_pause_at_end(Duration::from_secs(2))
}

// Start menu state
struct StartMenuState {
    items: Vec<(String, Bits)>,
    list_state: ListState,
    animation: ProceduralAnimationWidget,
}

impl StartMenuState {
    fn new() -> Self {
        Self::with_selected(get_last_selected_index())
    }

    fn with_selected(selected_index: usize) -> Self {
        let items = vec![
            ("easy       (4 bits)".to_string(), Bits::Four),
            ("easy+16    (4 bits*16)".to_string(), Bits::FourShift4),
            ("easy+256   (4 bits*256)".to_string(), Bits::FourShift8),
            ("easy+4096  (4 bits*4096)".to_string(), Bits::FourShift12),
            ("normal     (8 bits)".to_string(), Bits::Eight),
            ("master     (12 bits)".to_string(), Bits::Twelve),
            ("insane     (16 bits)".to_string(), Bits::Sixteen),
        ];

        Self {
            items,
            list_state: ListState::default().with_selected(Some(selected_index)),
            animation: ascii_animation(),
        }
    }

    fn selected_index(&self) -> usize {
        self.list_state.selected().unwrap_or(0)
    }
    fn selected_bits(&self) -> Bits {
        self.items[self.selected_index()].1.clone()
    }
    fn select_next(&mut self) {
        self.list_state.select_next();
    }
    fn select_previous(&mut self) {
        self.list_state.select_previous();
    }
    fn toggle_animation(&mut self) {
        self.animation.toggle_pause();
    }
}
