use pants_store::cli::CliApp;

fn main() {
    CliApp::run()
}

// fn main() {
//     let args = CliArgs::parse();
//     if let Some(command) = args.command {
//         match command {
//             pants_store::cli::CLICommands::Gen(args) => {
//                 if let Some(p) = args.execute() {
//                     println!("{p}");
//                 } else {
//                     println!("Could not satisfy password spec constraints");
//                 }
//             }
//             _ => match VaultInterface::interaction(command) {
//                 Ok(output) => {
//                     let res = output.finish();
//                     if let Err(e) = res {
//                         println!("Encountered error: {}", e);
//                     }
//                 }
//                 Err(e) => {
//                     println!("Encountered error: {}", e);
//                 }
//             },
//         }
//     }
// }
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
