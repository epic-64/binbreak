use crossterm::event::KeyEvent;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

pub trait WidgetRef {
    fn render_ref(&self, area: Rect, buf: &mut Buffer);
}

pub trait MainScreenWidget: WidgetRef {
    fn run(&mut self, dt: f64) -> ();
    fn handle_input(&mut self, input: KeyEvent) -> ();
    fn is_exit_intended(&self) -> bool;

    fn get_name(&self) -> String {
        let type_name = std::any::type_name::<Self>();
        type_name.split("::").last().unwrap_or("Unknown").to_string()
    }

    fn get_overview(&self) -> String {
        format!("You are here: {}. The overview is not implemented.", self.get_name())
    }
}