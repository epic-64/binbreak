use ratatui::layout::Flex;
use ratatui::prelude::*;
use std::collections::HashMap;

pub struct AsciiCell {
    pub ch: char,
    pub x: u16,
    pub y: u16,
    pub color: Color,
}

pub fn parse_ascii_art(
    art: String,
    color_map_str: String,
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
            let color = color_map.get(&color_ch).cloned().unwrap_or(default_color);
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
        art: String,
        color_map_str: String,
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
    pub fn new(collection: AsciiCells) -> Self {
        Self { collection }
    }
}

impl Widget for AsciiArtWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for pixel in self.collection.cells {
            let position = Position::new(pixel.x + area.x, pixel.y + area.y);

            if area.contains(position) {
                buf.cell_mut(position)
                    .expect("Failed to get cell at position")
                    .set_char(pixel.ch)
                    .set_fg(pixel.color);
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
