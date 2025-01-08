use anyhow::anyhow;
use candadolib::{
    add, export, find, import, init, key, ls, passphrase, password, read, rm, token,
    tui::{self, App, TableApp},
    unlock, update, Encrypter, ABOUT, VERSION,
};
use clap::{Parser, Subcommand};
use std::{fs, io::Write, path::PathBuf};

#[derive(Parser)]
#[command(version = VERSION, about = ABOUT, long_about = None)]
pub struct CandadoCLI {
    #[command(subcommand)]
    apps: Apps,
}

#[derive(Subcommand)]
enum Apps {
    Gen(Generators),
    Vault(Manager),
}

#[derive(Parser)]
#[command(about = "Generate Secrets")]
struct Generators {
    #[command(subcommand)]
    pub generator: Generator,
}

#[derive(Parser)]
#[command(about = "Manage Passwords")]
struct Manager {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
enum Generator {
    #[command(about = "Generate Password")]
    Password {
        #[arg(short = 'l', long, default_value_t = 16)]
        length: u32,
    },
    #[command(about = "Generate Token")]
    Token {
        #[arg(short = 'l', long, default_value_t = 32)]
        length: u32,
    },
    #[command(about = "Generate Key")]
    Key {
        #[arg(short = 'l', long, default_value_t = 16)]
        length: u32,
    },
    #[command(about = "Generate Passphrase")]
    Passphrase {
        #[arg(short = 'l', long, default_value_t = 4)]
        length: u32,
        #[arg(short = 'c', long, help = "use custom wordlist")]
        wordlist: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum Command {
    #[command(about = "init vault")]
    Init,

    #[command(about = "List entries")]
    Ls,

    #[command(about = "Remove entry")]
    Rm { id: String },

    #[command(about = "Read entry")]
    Inspect { id: String },

    #[command(about = "Add an entry")]
    Add {
        service: String,
        email: String,

        #[arg(short = 'p', long)]
        password: Option<String>,
        #[arg(short = 'n', long)]
        username: Option<String>,
        #[arg(short = 'u', long)]
        url: Option<String>,
    },

    #[command(about = "Update entry")]
    Update {
        id: String,

        #[arg(short = 's', long)]
        service: Option<String>,
        #[arg(short = 'e', long)]
        email: Option<String>,
        #[arg(short = 'p', long)]
        password: Option<String>,
        #[arg(short = 'n', long)]
        username: Option<String>,
        #[arg(short = 'u', long)]
        url: Option<String>,
    },

    #[command(about = "Find entries")]
    Find { query: String },

    #[command(about = "Import entries from .json file")]
    Import { file: PathBuf },

    #[command(about = "Export entries to .json file")]
    Export { file: PathBuf },
}

impl CandadoCLI {
    pub fn run() -> Result<(), anyhow::Error> {
        let cli = CandadoCLI::parse();
        match cli.apps {
            Apps::Gen(gen) => match gen.generator {
                Generator::Password { length } => {
                    let pass = password(length);
                    println!("{pass}");
                    Ok(())
                }
                Generator::Token { length } => {
                    let token = token(length);
                    println!("{token}");
                    Ok(())
                }
                Generator::Key { length } => {
                    let key = key(length);
                    println!("{key}");
                    Ok(())
                }
                Generator::Passphrase { length, wordlist } => {
                    let phrase = passphrase(length, &wordlist);
                    println!("{phrase}");
                    Ok(())
                }
            },
            Apps::Vault(manager) => match manager.command {
                Command::Init => {
                    println!("Initializing new Vault!");

                    if Encrypter::load_keyfile_path().is_ok() {
                        println!("WARNING there is already a existing Vault!");
                        println!("continue to permantly DELETE the exsting vault.");
                        print!("Are you sure to continue? [y/n]: ");
                        std::io::stdout().flush().unwrap();

                        let mut buffer = String::new();
                        std::io::stdin().read_line(&mut buffer)?;

                        match buffer.trim() {
                            "y" | "Y" | "yes" | "YES" => {
                                println!("Deleting vault...");
                                let path = format!("{}/.candado", std::env::var("HOME")?);
                                fs::remove_dir_all(path)?;
                            }
                            _ => {
                                return Err(anyhow!("Aborted."));
                            }
                        }
                    }

                    println!("WARNING");
                    println!("Make sure to use a strong password! Do not only rely on encryption.");
                    println!("Make sure to remember your passwort or save it in a safe location, you CAN'T recover access if you loose your password!");

                    match init() {
                        Ok(()) => println!("Vault Created!"),
                        Err(e) => println!("{e}"),
                    }
                    Ok(())
                }
                Command::Ls => {
                    let encrypter = unlock()?;
                    let entries = ls(encrypter)?;
                    tui::init(App::Table(TableApp::new(entries)?))
                }
                Command::Find { query } => {
                    let encrypter = unlock()?;
                    let entries = find(encrypter, &query)?;
                    tui::init(App::Table(TableApp::new(entries)?))
                }
                Command::Inspect { id } => {
                    let encrypter = unlock()?;
                    let entry = read(encrypter, &id)?;
                    tui::init(App::Table(TableApp::new(vec![entry])?))
                }
                Command::Add {
                    service,
                    email,
                    password,
                    username,
                    url,
                } => {
                    let encrypter = unlock()?;
                    match add(encrypter, service, email, password, username, url) {
                        Ok(()) => println!("Entry added: OK"),
                        Err(e) => println!("{e}"),
                    }
                    Ok(())
                }
                Command::Update {
                    id,
                    service,
                    email,
                    password,
                    username,
                    url,
                } => {
                    let encrypter = unlock()?;
                    match update(encrypter, &id, service, email, password, username, url) {
                        Ok(()) => println!("Entry updated: OK"),
                        Err(e) => println!("{e}"),
                    }
                    Ok(())
                }
                Command::Rm { id } => {
                    let encrypter = unlock()?;
                    match rm(encrypter, &id) {
                        Ok(()) => println!("Entry deleted: OK"),
                        Err(e) => println!("{e}"),
                    }
                    Ok(())
                }
                Command::Import { file } => {
                    let encrypter = unlock()?;
                    match import(encrypter, file) {
                        Ok(()) => println!("Import: OK"),
                        Err(e) => println!("{e}"),
                    }
                    Ok(())
                }
                Command::Export { file } => {
                    let encrypter = unlock()?;
                    match export(encrypter, file) {
                        Ok(()) => println!("Export: OK"),
                        Err(e) => println!("{e}"),
                    }
                    Ok(())
                }
            },
        }
    }
}

fn main() -> Result<(), anyhow::Error> {
    CandadoCLI::run()
}
