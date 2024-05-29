use clap::Parser;

use pants::{cli::CliArgs, vault_encrypted::VaultInterface};

fn main() {
    let args = CliArgs::parse();
    if let Some(command) = args.command {
        match VaultInterface::interaction(command) {
            Ok(output) => {
                let res = output.finish();
                if let Err(e) = res {
                    println!("Encountered error: {}", e);
                }
            }
            Err(e) => {
                println!("Encountered error: {}", e);
            }
        }
    }
}
// fn main() {
//     let validator = |input: &str| {
//         if input.chars().count() == 0 {
//             let val = Err((SchemaError::BadValues))?;
//             Ok(Validation::Valid)
//         } else {
//             Ok(Validation::Valid)
//         }
//     };
//     let p = Password::new("Enter password:")
//         .with_display_toggle_enabled()
//         .with_display_mode(inquire::PasswordDisplayMode::Masked)
//         .with_validator(validator)
//         .prompt();
//
//     println!("{:?}", p)
// }
