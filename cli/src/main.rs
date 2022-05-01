#![allow(deprecated)]

use std::{sync::RwLock, collections::HashMap};

use std::fs::{File, self};
use std::io::Write;

use config::Config;

use clap::{Command, arg};

fn cli() -> Command<'static> {
    Command::new("presence-light")
        .about("Watch and change the status of your presence-light application")
        .subcommand_required(true)
        // .arg_required_else_help(true)
        // .allow_external_subcommands(true)
        // .allow_invalid_utf8_for_external_subcommands(true)
        .subcommand(
            Command::new("config")
                .about("Read and alter the presence-light-cli configuration")
                .subcommand(
                    Command::new("get")
                        .subcommand_required(true)
                        .about("Read the configuration")
                        .subcommand(Command::new("backend"))
                        .arg_required_else_help(true),
                )
                .subcommand(
                    Command::new("set")
                        .subcommand_required(true)
                        .about("Set the configuration")
                        .subcommand(Command::new("backend").arg(arg!(<URL> "This URL is used to communicate with the backend.")))
                        .subcommand(Command::new("auth").arg(arg!(<TOKEN> "This auth token is used to send state changes.")))
                        .arg_required_else_help(true)
                )
        )
        .subcommand(
            Command::new("status")
                .about("Get the current status of your presence light")
        )
}

fn main() {
    
    let mut config_dir = dirs::config_dir().expect("Could not find global config dir.");

    config_dir.push("presence-light");

    fs::create_dir_all(config_dir.to_str().unwrap()).expect(format!("Could not create config dir: {}", config_dir.to_str().unwrap()).as_str());

    config_dir.push("config.toml");

    if !config_dir.exists() {
        File::create(&config_dir).expect("Could not create config file."); 
    }
    let settings = Config::builder()
        .add_source(config::File::with_name(config_dir.to_str().unwrap()))
        .build()
        .unwrap();

    let settings = settings
        .try_deserialize::<HashMap<String, String>>()
        .unwrap();

    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("status", _sub_matches)) => {
            let backend_url = settings.get("backend_url");

            if backend_url.is_none() {
                println!("You have to provide a backend URL to use this command!");
            } else {
               
                let mut backend_url = backend_url.unwrap().clone();

                if backend_url.ends_with("/") {
                    backend_url.pop();
                }

                backend_url.push_str("/current");

                let body = reqwest::blocking::get(backend_url.as_str()).expect("Could not fetch from the provided URL!").text().unwrap();

                println!("The current State is: {}", body);

            }

                
        },
        Some(("config", sub_matches)) => {
            match sub_matches.subcommand() {
                Some(("get", sub_matches)) => {
                    match sub_matches.subcommand() {
                        Some(("backend", _)) => {
                            let setting = settings.get("backend_url");
                            
                            match setting {
                                Some(value) => println!("The current backend URL is: \"{}\"", value),
                                None => println!("There isn't a backend URL yet.")
                            }

                        },
                        Some(("auth", _)) => {
                            let setting = settings.get("auth_token");
                            
                            match setting {
                                Some(value) => println!("The current auth token is: \"{}\"", value),
                                None => println!("There isn't a auth token set yet.")
                            }

                        },

                        _ => unreachable!()
                    }

                },
                Some(("set", sub_matches)) => {
                    match sub_matches.subcommand() {
                        Some(("backend", sub_matches)) => {
                         
                            let url = sub_matches.value_of("URL").unwrap();

                            write_to_config(settings, "backend_url", url);

                            println!("The backend URL was set to: \"{}\"", url);
                            
                        },
                        Some(("auth", sub_matches)) => {
                         
                            let token = sub_matches.value_of("TOKEN").unwrap();

                            write_to_config(settings, "auth_token", token);

                            println!("The auth token was set to: \"{}\"", token);
                            
                        },

                        _ => unreachable!()
                    }

                },
                _ => unreachable!()
            }
        },
        _ => unreachable!()
    }

}

fn write_to_config(config: HashMap<String, String>, change_key: &str, change_value: &str) {

    let mut config_dir = dirs::config_dir().expect("Could not find global config dir.");

    config_dir.push("presence-light");
    config_dir.push("config.toml");

    let mut file = File::create(config_dir).unwrap();

    for (key, value) in config.iter() {
        if key.as_str() == change_key {
            writeln!(&mut file, "{} = \"{}\"", key, change_value).unwrap();
        } else {
            writeln!(&mut file, "{} = \"{}\"", key, value).unwrap();
        }
    }

    if config.get(change_key).is_none() {
        writeln!(&mut file, "{} = \"{}\"", change_key, change_value).unwrap();
    }

}
