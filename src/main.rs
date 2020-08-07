use std::fs;
use std::string::String;
use std::os::unix::fs::PermissionsExt;
use structopt::StructOpt;
use ini::Ini;
use shellexpand;

#[derive(Debug, StructOpt)]
#[structopt(name = "example", about = "An example of StructOpt usage.")]
struct Cli {

    command_type: String,

    //#[structopt(required_if("command_type", "list"))]
    account_name: Option<String>,

    //#[structopt(required_if("account_name", Some)]
    key_name: Option<String>,

    /*
    /// Activate debug mode
    // short and long flags (-d, --debug) will be deduced from the field's name
    #[structopt(short, long)]
    debug: bool,

    /// Set speed
    // we don't want to name it "speed", need to look smart
    #[structopt(short = "v", long = "velocity", default_value = "42")]
    speed: f64,

    /// Input file
    #[structopt(parse(from_os_str))]
    input: PathBuf,

    /// Output file, stdout if not present
    #[structopt(parse(from_os_str))]
    output: Option<PathBuf>,

    /// Where to write the output: to `stdout` or `file`
    #[structopt(short)]
    out_type: String,

    /// File name: only required when `out-type` is set to `file`
    #[structopt(name = "FILE", required_if("out-type", "file"))]
    file_name: Option<String>,
    */
}

fn main() {
    println!("Hello, init!");

    let scaleapi_dir = create_directory();
    let credentials_file = format!("{scaleapi_dir}/credentials", scaleapi_dir=scaleapi_dir);
    let mut credentials = load_or_create_credentials(&credentials_file);

    let args = Cli::from_args();
    println!("{:?}",args);

    if args.command_type == "list" {
        if args.account_name == None {
            for k in credentials.sections() {
                println!("{}", k.unwrap_or("No sections available."));
            }
        } else {
            let section_name = Some(args.account_name.unwrap());
            let section = credentials.section(section_name).unwrap();
            
            if args.key_name == None {
                for (key, value) in section.iter() {
                    println!("{}\t{}", key, value);
                }
            }
            else {
                let property_name = args.key_name.unwrap();
                println!("{}", section.get(property_name).unwrap_or(""))
            }
            
        }
    }
    else if args.command_type == "add" {
        
    }
    
    

    credentials.with_section(Some("default")).set("API_KEY", "32423423123");
    save_credentials(&credentials, &credentials_file);
    
}

fn save_credentials(credentials: &Ini, credentials_file: &String) {
    credentials.write_to_file(credentials_file).unwrap();
    fs::set_permissions(credentials_file, fs::Permissions::from_mode(0o600)).unwrap();
}

fn create_directory() -> String {
    let home_dir = shellexpand::tilde("~");
    let scaleapi_dir = format!("{home_dir}/.scaleapi", home_dir=home_dir);
    fs::create_dir_all(&scaleapi_dir).unwrap();

    return scaleapi_dir;
}

fn load_or_create_credentials(credentials_file: &String) -> Ini {
    let credentials = Ini::load_from_file(credentials_file);
    let credentials = match credentials {
        Ok(credentials) => credentials,
        Err(_error) => Ini::new(),
    };
    return credentials
}