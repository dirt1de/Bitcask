use clap::{Parser, Subcommand};
use kvs::{KvStore, Result};
use std::env::current_dir;

#[derive(Parser)]
#[clap(version, about, long_about = None)] // This line helps
                                           // to extract the meta information from Cargo.toml

// The Cli struct holds all the options, positional, and subcommands
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Set { key: String, value: String },
    Get { key: String },
    Rm { key: String },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Set { key, value } => {
            let mut kv_store = KvStore::open(current_dir()?)?;
            let set_result = kv_store.set(key, value);

            if let Err(e) = set_result {
                println!("We have an error: {}", e);
                std::process::exit(1)
            };
        }
        Commands::Get { key } => {
            let mut kv_store = KvStore::open(current_dir()?)?;
            let get_result = kv_store.get(key);

            match get_result {
                Ok(Some(value)) => {
                    println!("{}", value);
                }
                Ok(None) => {
                    println!("Key not found");
                }
                Err(e) => {
                    println!("We have an error: {}", e);
                    std::process::exit(1)
                }
            }
        }
        Commands::Rm { key } => {
            let mut kv_store = KvStore::open(current_dir()?)?;
            let remove_result = kv_store.remove(key);

            if let Err(_e) = remove_result {
                println!("Key not found");
                std::process::exit(1);
            }
        }
    };

    Ok(())
}
