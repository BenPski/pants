use iced::{Application, Settings};
use pants_store::gui::state::vault::VaultState;

fn main() -> iced::Result {
    VaultState::run(Settings::default())
}
