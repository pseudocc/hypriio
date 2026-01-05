mod control;

use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <lock|unlock>", args[0]);
        std::process::exit(1);
    }

    let command = match args[1].as_str() {
        "lock" => control::socket::Command::Lock,
        "unlock" => control::socket::Command::Unlock,
        _ => {
            eprintln!("Invalid command: {}", args[1]);
            eprintln!("Usage: {} <lock|unlock>", args[0]);
            std::process::exit(1);
        }
    };

    let mut socket = control::socket::Client::connect()?;
    socket.send(command)?;

    println!("Orientation {}ed", args[1]);
    Ok(())
}
