use clap::{App, Arg};
use shellexpand;
use std::fs;
use std::process;

mod credentials;

fn main() {
    let app = App::new("Scale CLI")
        .version("0.1")
        .author("Nisse Knudsen <nisse.knudsen@gmail.com>")
        .about("Manage Scale API Keys via CLI")
        .arg(
            Arg::with_name("command")
                .help("Sets the command to execute")
                .index(1)
                .required(true),
        )
        .arg(
            Arg::with_name("account")
                .help("Sets the account name to apply operation on")
                .index(2)
                .required_ifs(&[("command", "add")]),
        )
        .arg(
            Arg::with_name("label")
                .help("Label of API Key")
                .index(3)
                .required_ifs(&[("command", "add")]),
        )
        .arg(
            Arg::with_name("value")
                .help("Value of API Key")
                .index(4)
                .required_ifs(&[("command", "add")]),
        );

    let matches = app.get_matches();

    match matches.value_of("command") {
        Some(command) => {
            let credentials: credentials::ScaleCredentials = load_or_create_credentials();
            match command {
                // list details from credentials file
                "list" => match matches.value_of("account") {
                    // If account is provided as next argument, print all stored keys for that account
                    Some(a_name) => match credentials.accounts.get(a_name) {
                        Some(a) => {
                            for (_kk, kv) in &a.keys {
                                println!("{}\t{}", kv.name, kv.value);
                            }
                        }
                        None => {}
                    },
                    // If no account has been provided, just list all accounts available for selection
                    None => {
                        for (_ak, av) in credentials.accounts {
                            println!("{}", av.name);
                        }
                    }
                },
                "add" => {
                    let account: String = match matches.value_of("account") {
                        Some(k) => String::from(k),
                        None => process::exit(1), // should not be reached because argument is required
                    };

                    let label: String = match matches.value_of("label") {
                        Some(k) => String::from(k),
                        None => process::exit(1), // should not be reached because argument is required
                    };

                    let value: String = match matches.value_of("value") {
                        Some(a) => String::from(a),
                        None => process::exit(1), // should not be reached because argument is required
                    };

                    credentials.add_entry(&account, &label, &value).save();
                }
                _ => {}
            }
        }
        None => {}
    }
}

fn get_directory() -> String {
    let scaleapi_dir: String = format!("{home_dir}/.scaleapi", home_dir = shellexpand::tilde("~"));
    match fs::create_dir_all(&scaleapi_dir) {
        Ok(_) => return scaleapi_dir,
        Err(error) => panic!(error),
    };
}

fn load_or_create_credentials() -> credentials::ScaleCredentials {
    let credentials_file: String =
        format!("{scaleapi_dir}/credentials", scaleapi_dir = get_directory());
    return credentials::ScaleCredentials::from_file(&credentials_file);
}
