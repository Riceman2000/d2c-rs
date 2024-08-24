use std::{env, path::PathBuf, process};

use cloudflare::{
    endpoints::dns,
    framework::{self, async_api, auth},
};
use serde_derive::Deserialize;
use tokio::fs;
use tracing::{error, info};

const CONFIG_DIR: &str = "/etc/d2c";
const SAMPLE_CONFIG: &str = include_str!("./sample_config.toml");
const USAGE: &str = include_str!("./usage.txt");

#[derive(Debug, Deserialize)]
struct ConfigFile {
    #[serde(skip)]
    file_name: String,
    api: Api,
    dns: Vec<Dns>,
}

#[derive(Debug, Deserialize)]
struct Api {
    #[serde(alias = "zone-id")]
    zone_id: String,

    #[serde(alias = "api-key")]
    api_key: String,
}

#[derive(Debug, Deserialize)]
struct Dns {
    name: String,
    proxy: bool,
}

#[tokio::main]
async fn main() {
    // Send usage string
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("{USAGE}");
        process::exit(0);
    }

    // Setup logging
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    // Validate config dir
    let config_dir = PathBuf::from(CONFIG_DIR);
    if !config_dir.exists() {
        error!("Config directory at {CONFIG_DIR} does not exist, creating the directory");
        match fs::create_dir(config_dir)
            .await {
            Ok(()) =>  info!("Directory created, you must populate it with at least one config file following this format:\n{SAMPLE_CONFIG}"),
            Err(e) => error!("Failed to create config directory with: {e}"),
        }
        process::exit(1);
    }
    if !config_dir.is_dir() {
        error!("Config dir at {CONFIG_DIR} is not a directory");
        process::exit(1);
    }

    // Resolve public IP
    info!("Finding public IP");
    let Some(public_ip) = public_ip::addr_v4().await else {
        error!("Failed to resolve public IP");
        process::exit(1);
    };
    info!("Public IP found as {public_ip}");

    // Process config files
    info!("Parsing config directory {CONFIG_DIR}");
    let mut config_files = match fs::read_dir(CONFIG_DIR).await {
        Ok(f) => f,
        Err(e) => {
            error!("Failed to read config directory with: {e}");
            process::exit(1);
        }
    };
    let mut configs = Vec::new();
    while let Ok(Some(file)) = config_files.next_entry().await {
        info!("Found file {file:?}");
        let file_name = file
            .file_name()
            .into_string()
            .expect("Malformed file name {file:#?}");
        let content = fs::read_to_string(file.path())
            .await
            .expect("Failed to read file");

        let mut config: ConfigFile = match toml::from_str(&content) {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to read file {file_name} with {e}");
                continue;
            }
        };
        config.file_name = file_name;
        info!("File content: {config:#?}");
        configs.push(config);
    }
    info!("Configurations collected from {CONFIG_DIR}");

    // Update records
    for config in configs {
        info!("Processing file {}", config.file_name);
        let credentials = auth::Credentials::UserAuthToken {
            token: config.api.api_key,
        };
        let api_config = framework::HttpApiClientConfig::default();
        let environment = framework::Environment::Production;
        let client = match async_api::Client::new(credentials, api_config, environment) {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to form API client with {e}");
                continue;
            }
        };

        let dns_list_request = dns::ListDnsRecords {
            zone_identifier: &config.api.zone_id,
            params: dns::ListDnsRecordsParams::default(),
        };
        let dns_list = match client.request(&dns_list_request).await {
            Ok(d) => d,
            Err(e) => {
                error!("API request to list existing records failed with {e}");
                continue;
            }
        };
        info!("Existing records:\n{dns_list:#?}");

        for dns in config.dns {
            info!("Processing record {}", dns.name);
            info!("Record {} updated", dns.name);
        }
    }
}
