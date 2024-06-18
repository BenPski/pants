use iced::{Application, Settings};
use pants_store::gui::state::manager::ManagerState;

fn main() -> iced::Result {
    ManagerState::run(Settings::default())
}
