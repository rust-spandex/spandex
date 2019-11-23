use std::env::current_dir;
use std::fs::{create_dir_all, File};
use std::io::{Read, Write};
use std::process::exit;

use clap::{crate_description, crate_version, App, AppSettings, Arg, SubCommand};

use spandex::document::configuration::Config;
use spandex::{build, Error};

macro_rules! unwrap {
    ($e: expr, $error: expr) => {
        match $e {
            Some(e) => e,
            None => return Err($error),
        }
    };
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        exit(1);
    }
}

fn run() -> Result<(), Error> {
    let mut app = App::new("SpanDeX")
        .bin_name("spandex")
        .version(crate_version!())
        .about(crate_description!())
        .setting(AppSettings::ColoredHelp)
        .subcommand(
            SubCommand::with_name("init")
                .about("Creates a new default SpanDeX project")
                .arg(Arg::with_name("TITLE").required(false)),
        )
        .subcommand(SubCommand::with_name("build").about("Builds the SpanDeX project"));

    let matches = app.clone().get_matches();

    if let Some(init) = matches.subcommand_matches("init") {
        let mut current_dir = unwrap!(current_dir().ok(), Error::CannotReadCurrentDir);
        let current_dir_name = current_dir.clone();
        let current_dir_name = unwrap!(current_dir_name.file_name(), Error::CannotReadCurrentDir);
        let current_dir_name = unwrap!(current_dir_name.to_str(), Error::CannotReadCurrentDir);

        // Initialize the project
        let title = match init.value_of("TITLE") {
            // If a title was given, we will create a directory for the project
            Some(title) => {
                current_dir.push(title);
                title
            }

            // If no title was given, use current_dir_name
            None => current_dir_name,
        };

        // Try to create the directory
        create_dir_all(&current_dir).ok();

        // Create the default config and save it
        let config = Config::with_title(title);
        let toml = toml::to_string(&config).expect("Failed to generate toml");

        current_dir.push("spandex.toml");
        let mut file = File::create(&current_dir)?;
        file.write_all(toml.as_bytes())?;

        // Write an hello world file
        current_dir.pop();
        current_dir.push("main.dex");

        let mut file = File::create(&current_dir)?;
        file.write_all(b"# Hello world")?;
    } else if matches.subcommand_matches("build").is_some() {
        // Look up for spandex config file
        let mut current_dir = unwrap!(current_dir().ok(), Error::CannotReadCurrentDir);
        let config_path = loop {
            current_dir.push("spandex.toml");

            if current_dir.is_file() {
                break current_dir;
            } else {
                // Remove spandex.toml
                current_dir.pop();

                // Go to the parent directory
                if !current_dir.pop() {
                    return Err(Error::NoConfigFile);
                }
            }
        };

        // Read config file
        let mut file = File::open(&config_path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        let config: Config = toml::from_str(&content).expect("Failed to parse toml");
        build(&config)?;
    } else {
        // Nothing to do, print help
        app.print_help().ok();
        println!();
    }

    Ok(())
}
