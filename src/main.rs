use clap::{App, Arg};
use ini::Ini;
use shellexpand;
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process;
use std::string::String;

#[derive(Debug)]
struct ScaleCredentials {
    filepath: String,
    accounts: HashMap<String, ScaleAccount>,
}

#[derive(Debug)]
struct ScaleAccount {
    name: String,
    keys: HashMap<String, ScaleApiKey>,
}

#[derive(Debug)]
struct ScaleApiKey {
    name: String,
    value: String,
}

impl ScaleCredentials {
    pub fn new(fp: &String) -> ScaleCredentials {
        let credentials = ScaleCredentials {
            filepath: fp.clone(),
            accounts: HashMap::new(),
        };

        return credentials;
    }

    pub fn from_file(fp: &String) -> ScaleCredentials {
        match Ini::load_from_file(&fp) {
            Ok(credentials) => return ScaleCredentials::from_ini(credentials, fp),
            Err(_error) => {
                println!("Not able to parse file. Creating empty credentials config.");
                return ScaleCredentials::new(fp);
            }
        };
    }

    pub fn add_entry(mut self, account: &String, label: &String, value: &String) -> Self {
        let mut account_entry: ScaleAccount = match self.accounts.remove(account) {
            Some(k) => k,
            None => ScaleAccount {
                name: String::from(account),
                keys: HashMap::new(),
            },
        };

        let key_entry: ScaleApiKey = match account_entry.keys.remove(label) {
            Some(_) | None => ScaleApiKey {
                name: String::from(label),
                value: String::from(value),
            },
        };

        account_entry.keys.insert(label.clone(), key_entry);
        self.accounts.insert(account.clone(), account_entry);

        return self;
    }

    pub fn save(&self) {
        let credentials_file: &String = &self.filepath;
        let ini: Ini = self.to_ini();
        ini.write_to_file(credentials_file).unwrap();
        fs::set_permissions(credentials_file, fs::Permissions::from_mode(0o600)).unwrap();
    }

    fn from_ini(ini: Ini, fp: &String) -> ScaleCredentials {
        let mut credentials: ScaleCredentials = ScaleCredentials::new(fp);

        for j in ini.sections() {
            match j {
                Some(s_val) => {
                    let mut account = ScaleAccount {
                        name: String::from(s_val),
                        keys: HashMap::new(),
                    };

                    match ini.section(Some(&account.name)) {
                        Some(properties) => {
                            for (k, v) in properties.iter() {
                                let api_key = ScaleApiKey {
                                    name: String::from(k),
                                    value: String::from(v),
                                };
                                account.keys.insert(api_key.name.clone(), api_key);
                            }
                        }
                        None => println!("Not able to parse content for section {}", &account.name),
                    }

                    credentials.accounts.insert(account.name.clone(), account);
                }
                None => {}
            }
        }

        return credentials;
    }

    fn to_ini(&self) -> Ini {
        let mut ini: Ini = Ini::new();
        for (_ak, av) in &self.accounts {
            for (_kk, kv) in &av.keys {
                ini.set_to(Some(&av.name), kv.name.clone(), kv.value.clone());
            }
        }

        return ini;
    }
}

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
            let credentials: ScaleCredentials = load_or_create_credentials();
            match command {
                // list details from credentials file
                "list" => match matches.value_of("account") {
                    // If account is provided as next argument, print all stored keys for that account
                    Some(account) => match credentials.accounts.get(account) {
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

fn load_or_create_credentials() -> ScaleCredentials {
    let credentials_file: String =
        format!("{scaleapi_dir}/credentials", scaleapi_dir = get_directory());
    return ScaleCredentials::from_file(&credentials_file);
}
