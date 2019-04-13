#[macro_use]
extern crate log;

use std::error::Error;
use std::env::current_dir;
use std::path::PathBuf;
use std::process::exit;
use std::fs::{File, create_dir_all};
use std::io::{Read, Write};

use clap::{App, Arg, SubCommand, crate_version, crate_authors, crate_description};

use spandex::config::Config;
use spandex::Error as SError;

macro_rules! unwrap {
    ($e: expr, $error: expr) => {
        match $e {
            Some(e) => e,
            None => return Err(Box::new($error)),
        }
    }
}

fn main() {
    beautylog::init(log::LevelFilter::Trace).ok();

    if let Err(e) = run() {
        error!("{}", e);
        exit(1);
    }
}

fn run() -> Result<(), Box<Error>> {

    let matches = App::new("SpanDeX")
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .about(crate_description!())
        .subcommand(SubCommand::with_name("init")
            .about("Creates a new default SpanDeX project")
            .arg(Arg::with_name("TITLE")
                 .required(false)))
        .subcommand(SubCommand::with_name("build")
            .about("Builds the SpanDeX project"))
        .get_matches();

    if let Some(init) = matches.subcommand_matches("init") {

        let mut current_dir = PathBuf::from(unwrap!(current_dir().ok(), SError::CannotReadCurrentDir));
        let current_dir_name = current_dir.clone();
        let current_dir_name = unwrap!(current_dir_name.file_name(), SError::CannotReadCurrentDir);
        let current_dir_name = unwrap!(current_dir_name.to_str(), SError::CannotReadCurrentDir);

        // Initialize the project
        let title = match init.value_of("TITLE") {
            // If a title was given, we will create a directory for the project
            Some(title) => {
                current_dir.push(title);
                title
            },

            // If no title was given, use current_dir_name
            None => current_dir_name,
        };

        // Try to create the directory
        create_dir_all(&current_dir).ok();

        // Create the default config and save it
        let config = Config::with_title(title);
        let toml =toml::to_string(&config)?;

        current_dir.push("spandex.toml");
        let mut file = File::create(&current_dir)?;
        file.write(toml.as_bytes())?;

        // Write an hello world file
        current_dir.pop();
        current_dir.push("main.txt");

        let mut file = File::create(&current_dir)?;
        file.write("Hello world".as_bytes())?;

    } else if let Some(_) = matches.subcommand_matches("build") {

        // Look up for spandex config file
        let mut current_dir = PathBuf::from(unwrap!(current_dir().ok(), SError::CannotReadCurrentDir));
        let config_path = loop {
            current_dir.push("spandex.toml");

            if current_dir.is_file() {
                break current_dir;
            } else {

                // Remove spandex.toml
                current_dir.pop();

                // Go to the parent directory
                if ! current_dir.pop() {
                    return Err(Box::new(SError::NoConfigFile));
                }
            }
        };

        // Read config file
        let mut file = File::open(&config_path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        let config: Config = toml::from_str(&content)?;
        config.build()?;

    }



    Ok(())
}
