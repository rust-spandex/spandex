use std::env::{self, current_dir};
use std::fs::{create_dir_all, File};
use std::io::{Read, Write};
use std::process::exit;

use colored::*;

use spandex::document::configuration::Config;
use spandex::Error;

macro_rules! unwrap {
    ($e: expr, $error: expr) => {
        match $e {
            Some(e) => e,
            None => return Err($error),
        }
    };
}

fn print_version() {
    println!("SpanDeX {}", env!("CARGO_PKG_VERSION"));
}

fn print_help() {
    println!(
        r#"{name} {version}
{description}

{USAGE}
    {command} [SUBCOMMAND]

{FLAGS}
    {help_short}, {help_long}       Prints help information
    {version_short}, {version_long}    Prints version information

{SUBCOMMANDS}
    {build}           Builds SpanDeX project
    {init}    Creates new default SpanDeX project"#,
        name = "SpanDeX".green(),
        version = env!("CARGO_PKG_VERSION"),
        description = env!("CARGO_PKG_DESCRIPTION"),
        USAGE = "USAGE:".yellow(),
        command = env!("CARGO_PKG_NAME"),
        FLAGS = "FLAGS:".yellow(),
        help_short = "-h".green(),
        help_long = "--help".green(),
        version_short = "-v".green(),
        version_long = "--version".green(),
        SUBCOMMANDS = "SUBCOMMANDS:".yellow(),
        build = "build".green(),
        init = "init [title]".green(),
    );
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        exit(1);
    }
}

fn init(name: Option<&String>) -> Result<(), Error> {
    let mut current_dir = unwrap!(current_dir().ok(), Error::CannotReadCurrentDir);
    let current_dir_name = current_dir.clone();
    let current_dir_name = unwrap!(current_dir_name.file_name(), Error::CannotReadCurrentDir);
    let current_dir_name = unwrap!(current_dir_name.to_str(), Error::CannotReadCurrentDir);

    // Initialize the project
    let title = match name.as_ref() {
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

    Ok(())
}

fn build() -> Result<(), Error> {
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
    spandex::build(&config)?;

    Ok(())
}

fn run() -> Result<(), Error> {
    let args = env::args().collect::<Vec<_>>();

    // The first argument is the name of the binary, the second one is the command
    if args.len() < 2 {
        eprintln!("{}: {}", "error".red().bold(), "toto");
        print_help();
        exit(1);
    }

    if args.contains(&String::from("-h")) || args.contains(&String::from("--help")) {
        print_help();
        exit(0);
    }

    if args.contains(&String::from("-v")) || args.contains(&String::from("--version")) {
        print_version();
        exit(0);
    }

    match args[1].as_ref() {
        "init" => init(args.get(2))?,

        "build" => build()?,

        command => {
            // Unknwon command
            eprintln!(
                "{}: {}{}{}",
                "error".bold().red(),
                "command \"",
                command,
                "\" does not exist."
            );
            print_help();
        }
    }

    Ok(())
}
