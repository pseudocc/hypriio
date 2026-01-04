mod control;

use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <lock|unlock>", args[0]);
        std::process::exit(1);
    }

    let lock = match args[1].as_str() {
        "lock" => true,
        "unlock" => false,
        _ => {
            eprintln!("Invalid command: {}", args[1]);
            eprintln!("Usage: {} <lock|unlock>", args[0]);
            std::process::exit(1);
        }
    };

    let mut config = control::Config::load();
    config.set_lock(lock)?;

    println!("Orientation {}", if lock { "locked" } else { "unlocked" });
    Ok(())
}

