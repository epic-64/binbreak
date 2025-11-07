use crate::binary_numbers::{BinaryNumbersGame, Bits};
use crate::main_screen_widget::MainScreenWidget;
use crate::utils::{AsciiArtWidget, AsciiCells};
use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::{Color, Modifier, Span, Style, Widget};
use ratatui::widgets::{List, ListItem, ListState};
use std::collections::HashMap;
use std::thread;
use std::time::Instant;

enum AppState {
    Start(StartMenuState),
    Playing(BinaryNumbersGame),
    Exit,
}

fn handle_start_input(state: &mut StartMenuState, key: KeyEvent) -> Option<AppState> {
    match key.code {
        KeyCode::Up => state.select_previous(),
        KeyCode::Down => state.select_next(),
        KeyCode::Enter => {
            let bits = state.selected_bits();
            return Some(AppState::Playing(BinaryNumbersGame::new(bits)));
        }
        KeyCode::Esc => return Some(AppState::Exit),
        _ => {}
    }
    None
}

fn render_start_screen(state: &mut StartMenuState, area: Rect, buf: &mut Buffer) {
    // Build ASCII art to obtain real dimensions
    let cells = ascii_art_cells();
    let ascii_width = cells.get_width();
    let ascii_height = cells.get_height();
    let ascii_widget = AsciiArtWidget::new(cells);

    let selected = state.selected_index();
    let upper_labels: Vec<String> = state.items.iter().map(|(l, _)| l.to_uppercase()).collect();
    let max_len = upper_labels
        .iter()
        .map(|s| s.len() as u16)
        .max()
        .unwrap_or(0);

    let list_width = 2 + max_len; // marker + space + label
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
    let ascii_area = Rect::new(
        ascii_x,
        ascii_y,
        ascii_width.min(area.width),
        ascii_height.min(area.height),
    );
    let list_area = Rect::new(
        list_x,
        list_y,
        list_width.min(area.width),
        list_height.min(area.height.saturating_sub(list_y - area.y)),
    );

    // Render ASCII art
    ascii_widget.render(ascii_area, buf);

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
            let line = format!("{} {}", marker, padded);
            let style = Style::default()
                .fg(palette[i % palette.len()])
                .add_modifier(Modifier::BOLD);
            ListItem::new(Span::styled(line, style))
        })
        .collect();

    let list = List::new(items);
    ratatui::widgets::StatefulWidget::render(list, list_area, buf, &mut state.list_state);
}

pub fn run_app(terminal: &mut ratatui::DefaultTerminal) -> color_eyre::Result<()> {
    let mut app_state = AppState::Start(StartMenuState::new());
    let mut last_frame_time = Instant::now();
    let target_frame_duration = std::time::Duration::from_millis(33); // ~30 FPS

    while !matches!(app_state, AppState::Exit) {
        let now = Instant::now();
        let dt = now - last_frame_time;
        last_frame_time = now;

        terminal.draw(|f| match &mut app_state {
            AppState::Start(menu) => render_start_screen(menu, f.area(), f.buffer_mut()),
            AppState::Playing(game) => f.render_widget(&mut *game, f.area()),
            AppState::Exit => {}
        })?;

        // Advance game if playing
        if let AppState::Playing(game) = &mut app_state {
            game.run(dt.as_secs_f64());
            if game.is_exit_intended() {
                app_state = AppState::Start(StartMenuState::new());
                continue;
            }
        }

        // handle input
        let poll_timeout = std::cmp::min(dt, target_frame_duration);
        if event::poll(poll_timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        // global exit via Ctrl+C
                        KeyCode::Char('c') | KeyCode::Char('C')
                            if key.modifiers == KeyModifiers::CONTROL =>
                        {
                            app_state = AppState::Exit;
                        }

                        // state-specific input handling
                        _ => {
                            app_state = match app_state {
                                AppState::Start(mut menu) => handle_start_input(&mut menu, key)
                                    .unwrap_or(AppState::Start(menu)),
                                AppState::Playing(mut game) => {
                                    game.handle_game_input(key);
                                    AppState::Playing(game)
                                }
                                AppState::Exit => AppState::Exit,
                            }
                        }
                    }
                }
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

fn ascii_art_cells() -> AsciiCells {
    let art = r#"
 ,,        ,,              ,,
*MM        db             *MM                                `7MM
 MM                        MM                                  MM
 MM,dMMb.`7MM  `7MMpMMMb.  MM,dMMb.`7Mb,od8 .gP"Ya   ,6"Yb.    MM  ,MP'
 MM    `Mb MM    MM    MM  MM    `Mb MM' "',M'   Yb 8)   MM    MM ;Y
 MM     M8 MM    MM    MM  MM     M8 MM    8M""""""  ,pm9MM    MM;Mm
 MM.   ,M9 MM    MM    MM  MM.   ,M9 MM    YM.    , 8M   MM    MM `Mb.
 P^YbmdP'.JMML..JMML  JMML.P^YbmdP'.JMML.   `Mbmmd' `Moo9^Yo..JMML. YA.
    "#;

    let colors = r#"
 ,,        ,,              ,,
*MM        db             *MM                                `7MM
 MM                        MM                                  MM
 MM,dMMb.`7MM  `7MMpMMMb.  MM,dMMb.`7Mb,od8 .gP"Ya   ,6"Yb.    MM  ,MP'
 MM    `Mb MM    MM    MM  MM    `Mb MM' "',M'   Yb 8)   MM    MM ;Y
 MM     M8 MM    MM    MM  MM     M8 MM    8M""""""  ,pm9MM    MM;Mm
 MM.   ,M9 MM    MM    MM  MM.   ,M9 MM    YM.    , 8M   MM    MM `Mb.
 P^YbmdP'.JMML..JMML  JMML.P^YbmdP'.JMML.   `Mbmmd' `Moo9^Yo..JMML. YA.
    "#;

    let color_map = HashMap::from([
        ('M', Color::White),
        ('b', Color::LightYellow),
        ('d', Color::LightCyan),
        ('Y', Color::LightGreen),
        ('8', Color::LightMagenta),
        ('*', Color::Magenta),
        ('`', Color::Cyan),
        ('6', Color::Green),
        ('9', Color::Red),
        ('(', Color::Blue),
        (')', Color::Blue),
        (' ', Color::Black),
    ]);

    let default_color = Color::LightBlue;
    AsciiCells::from(
        art.to_string(),
        colors.to_string(),
        &color_map,
        default_color,
    )
}

// Start menu state
struct StartMenuState {
    items: Vec<(String, Bits)>,
    list_state: ListState,
}

impl StartMenuState {
    fn new() -> Self {
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
            list_state: ListState::default().with_selected(Some(4)),
        } // default to normal (8 bits)
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
}
