use cargo_next::{bump_toml_version, get_version, set_version, Increment};
use clap::{Parser, Subcommand};
use semver::Version;
use std::{env::current_dir, io, process::exit};

#[derive(Debug, Parser)]
#[clap(author, bin_name("cargo-next"), version)]
struct Cli {
    /// This is because when we're called from cargo, our first arg is the command we were calld as.
    next: String,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Get,
    Major,
    Minor,
    Patch,
    Set { version: Option<String> },
}

fn read_stdin() -> Result<Option<String>, std::io::Error> {
    let mut piped = String::new();
    io::stdin().read_line(&mut piped)?;
    let piped_trim = piped.trim();
    match piped_trim.is_empty() {
        true => Ok(None),
        false => Ok(Some(piped_trim.to_string())),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Check if the current directory is actually a cargo project.
    let cargo_project_dir_path = current_dir()?;
    let cargo_toml_file_path = cargo_project_dir_path.join("Cargo.toml");
    if !cargo_toml_file_path.exists() {
        eprintln!("Not inside a cargo project folder!");
        exit(1);
    }

    let res = match cli.command {
        Commands::Get => {
            let res = get_version(&cargo_toml_file_path);
            if let Ok(version) = &res {
                println!("{version}");
            }
            res
        }
        Commands::Set { mut version } => {
            if let None = version {
                version = read_stdin()?;
            }
            match version {
                Some(v) => set_version(&cargo_toml_file_path, v),
                None => Ok(Version::parse("0.0.0")?),
            }
        }
        Commands::Major => bump_toml_version(&cargo_toml_file_path, Increment::Major),
        Commands::Minor => bump_toml_version(&cargo_toml_file_path, Increment::Minor),
        Commands::Patch => bump_toml_version(&cargo_toml_file_path, Increment::Patch),
    };

    if let Err(e) = res {
        eprintln!("{e}");
        exit(1);
    }

    // // If no flag has been specified and no version, read from stdin.
    // if !cli.major && !cli.minor && !cli.patch && !cli.get && cli.version.is_none() {
    //     let mut piped = String::new();
    //     io::stdin().read_line(&mut piped)?;
    //     let piped_trim = piped.trim();
    //     if !piped_trim.is_empty() {
    //         cli.version = Some(piped_trim.to_string());
    //     }
    // }

    // if cli.get {
    //     println!("{}", get_version(&cargo_toml_file_path)?);
    // } else if cli.major {
    //     bump_toml_version(&cargo_toml_file_path, Increment::Major)?;
    // } else if cli.minor {
    //     bump_toml_version(&cargo_toml_file_path, Increment::Minor)?;
    // } else if cli.patch {
    //     bump_toml_version(&cargo_toml_file_path, Increment::Patch)?;
    // } else {
    //     // Safety: Either `version` contains a String supplied from the user or the CLI
    //     // waits until it can read from stdin, in which case a version gets set as well.
    //     set_version(&cargo_toml_file_path, cli.version.unwrap())?;
    // }

    Ok(())
}
