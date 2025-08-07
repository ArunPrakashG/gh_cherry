use crossterm::event::{Event, KeyEvent, MouseEvent};

#[derive(Debug, Clone)]
#[allow(dead_code)] // This enum is for future TUI event handling
pub enum AppEvent {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Tick,
    Resize(u16, u16),
}

impl From<Event> for AppEvent {
    fn from(event: Event) -> Self {
        match event {
            Event::Key(key) => AppEvent::Key(key),
            Event::Mouse(mouse) => AppEvent::Mouse(mouse),
            Event::Resize(width, height) => AppEvent::Resize(width, height),
            _ => AppEvent::Tick,
        }
    }
}
