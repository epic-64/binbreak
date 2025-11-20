mod app;
mod binary_numbers;
mod keybinds;
mod main_screen_widget;
mod utils;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let mut terminal = ratatui::init();
    let result = app::run_app(&mut terminal);
    ratatui::restore();
    result
}
