#![allow(deprecated)]

use std::collections::HashMap;

use std::fs::{File, self};
use std::io::Write;

use config::Config;

use clap::{Command, arg, Arg};

use clap_complete::{generate, Generator, Shell};
use reqwest::StatusCode;
use std::io;

fn cli() -> Command<'static> {
    Command::new("presence-light")
        .about("Watch and change the status of your presence-light application")
        // .subcommand_required(true)
        // .arg_required_else_help(true)
        // .allow_external_subcommands(true)
        // .allow_invalid_utf8_for_external_subcommands(true)
        .arg(
            Arg::new("generator")
                .long("generate")
                .possible_values(Shell::possible_values()),
        )
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
                        .subcommand(Command::new("backend").arg(
                            Arg::new("url")
                            .value_hint(clap::ValueHint::Url)
                        ))
                        .subcommand(Command::new("auth").arg(arg!(<TOKEN> "This auth token is used to send state changes.")))
                        .arg_required_else_help(true)
                )
        )
        .subcommand(
            Command::new("status")
                .about("Get the current status of your presence light")
        )
        .subcommand(
            Command::new("post").visible_alias("set")
                .about("Change the current presence light status")
                .arg(
                    Arg::new("state")
                    .possible_values(["BUSY", "OK_FOR_INTERRUPTIONS", "FREE", "OFF"])
                )
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

    if let Ok(generator) = matches.value_of_t::<Shell>("generator") {
        let mut cmd = cli();
        eprintln!("Generating completion file for {}...", generator);
        print_completions(generator, &mut cmd);
    }

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
        Some(("post", sub_matches)) => {
            
            let auth_token = settings.get("auth_token");
            let backend_url = settings.get("backend_url");

            let state = sub_matches.value_of("state").unwrap().to_string();
            
            if let Some(auth_token) = auth_token {
                if let Some(backend_url) = backend_url {


                    let client = reqwest::blocking::Client::new();

                    let res = client.post("http://httpbin.org/post")
                        .header(reqwest::header::AUTHORIZATION, format!("Bearer {}", auth_token))
                        .body(state.clone())
                        .send().unwrap();

                    let status = res.status();

                    if status == StatusCode::UNAUTHORIZED {
                        println!("Please provide a valid auth code!");
                        std::process::exit(1);
                    }

                    if !status.is_success() {
                        println!("Someting went wrong while trying to post to: \"{}\"", backend_url);
                        std::process::exit(1);
                    }

                } else {
                    println!("You have to provide a backend URL to use this command!");
                    std::process::exit(1);
                }
            } else {
                println!("You need to provide an auth token to use this command");
                std::process::exit(1);
            }

            println!("The presence light status was set to: \"{}\"", state);
            
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
                         
                            let url = sub_matches.value_of("url").unwrap();

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
        _ => {}
    }

}

fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut io::stdout());
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
