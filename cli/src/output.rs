use std::{thread, time::Duration};

use arboard::Clipboard;
use clap::ValueEnum;
use pants_store::{reads::Reads, store::Store};
use secrecy::ExposeSecret;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OutputStyle {
    Clipboard,
    None,
    Raw,
}

impl OutputStyle {
    pub fn handle_reads(&self, reads: Reads<Store>) -> anyhow::Result<()> {
        if reads.data.is_empty() {
            println!("No data read from vault");
            return Ok(());
        }
        match self {
            OutputStyle::Clipboard => {
                let mut clipboard = Clipboard::new()?;
                let orig = clipboard.get_text().unwrap_or("".to_string());
                for (key, value) in reads.data.clone().into_iter() {
                    for (ident, item) in value.data {
                        clipboard.set_text(item.expose_secret().to_string())?;

                        if inquire::Text::new(&format!(
                            "Copied `{key}-{ident}` to clipboard, hit enter to continue"
                        ))
                        .prompt()
                        .is_err()
                        {
                            break;
                        }
                    }
                }
                clipboard.set_text(orig)?;
                println!("Resetting clipboard");
                thread::sleep(Duration::from_secs(1));
            }
            OutputStyle::Raw => {
                println!("{:?}", reads);
            }
            OutputStyle::None => {}
        }
        Ok(())
    }
}
