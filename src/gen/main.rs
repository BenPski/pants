use std::process::exit;

use pants_gen::cli::CliArgs;

fn main() {
    if let Some(password) = CliArgs::run() {
        println!("{}", password);
    } else {
        println!("Contraints couldn't be met, try again after adjusting the spec");
        exit(1)
    }
}
