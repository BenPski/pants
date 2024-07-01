use iced::{Application, Font, Settings};
use pants_store::gui::state::manager::ManagerState;

fn main() -> iced::Result {
    ManagerState::run(Settings {
        default_font: Font::MONOSPACE,
        ..Default::default()
    })
}
