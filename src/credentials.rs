use ini::Ini;
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs;
use std::os::unix::fs::PermissionsExt;

macro_rules! pub_struct {
    ($name:ident {$($field:ident: $t:ty,)*}) => {
        #[derive(Debug)]
        pub struct $name {
            $(pub $field: $t),*
        }
    }
}

pub_struct!(ScaleCredentials {
    filepath: String,
    accounts: HashMap<String, ScaleAccount>,
});

pub_struct!(ScaleAccount {
    name: String,
    keys: HashMap<String, ScaleApiKey>,
});

pub_struct!(ScaleApiKey {
    name: String,
    value: String,
});

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
