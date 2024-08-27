use std::{env, net::Ipv4Addr, path::PathBuf, process};

use cloudflare::{
    endpoints::dns,
    framework::{self, async_api, auth},
};
use serde_derive::Deserialize;
use tokio::fs;
use tracing::{debug, error, info, warn, Level};

const CONFIG_DIR: &str = "/etc/d2c";
const SAMPLE_CONFIG: &str = include_str!("./sample_config.toml");
const USAGE: &str = include_str!("./usage.txt");

#[tokio::main]
async fn main() {
    // Send usage string
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("{USAGE}");
        process::exit(0);
    }

    // Setup logging
    let subscriber = if args.iter().any(|a| *a == "-vv") {
        tracing_subscriber::FmtSubscriber::builder()
            .with_max_level(Level::TRACE)
            .finish()
    } else if args.iter().any(|a| *a == "-v") {
        tracing_subscriber::FmtSubscriber::builder()
            .with_max_level(Level::DEBUG)
            .finish()
    } else {
        tracing_subscriber::FmtSubscriber::new()
    };
    tracing::subscriber::set_global_default(subscriber).unwrap();

    validate_config_dir().await;

    let configs = parse_config_files().await;

    // Resolve public IP
    info!("Finding public IP");
    let Some(public_ip) = public_ip::addr_v4().await else {
        error!("Failed to resolve public IP");
        process::exit(1);
    };
    info!("Public IP found as {public_ip}");

    update_records(configs, public_ip).await;
}

async fn validate_config_dir() {
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
}

#[derive(Debug, Deserialize)]
struct ConfigFile {
    #[serde(skip)]
    file_name: String,
    api: Api,
    dns: Vec<Dns>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Api {
    zone_id: String,
    api_key: String,
}

#[derive(Debug, Deserialize)]
struct Dns {
    name: String,
    proxy: bool,
}

async fn parse_config_files() -> Vec<ConfigFile> {
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
        let file_name = file
            .file_name()
            .into_string()
            .expect("Malformed file name {file:#?}");
        info!("Found file {file_name}");
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
        debug!("File content: {config:#?}");
        configs.push(config);
    }
    info!("Configurations collected from {CONFIG_DIR}");
    configs
}

async fn update_records(configs: Vec<ConfigFile>, public_ip: Ipv4Addr) {
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
        debug!("Existing records:\n{dns_list:#?}");

        for dns in config.dns {
            info!("Processing record \"{}\"", dns.name);
            let Some(existing_record) = dns_list.result.iter().find(|r| r.name == dns.name) else {
                warn!("Matching DNS record for \"{}\" could not be found to update, please add it in Cloudflare manually.", dns.name);
                continue;
            };
            let last_update_time = existing_record.modified_on;
            let update_request = dns::UpdateDnsRecord {
                zone_identifier: &config.api.zone_id,
                identifier: &existing_record.id,
                params: dns::UpdateDnsRecordParams {
                    ttl: None,
                    proxied: Some(dns.proxy),
                    name: &dns.name,
                    content: dns::DnsContent::A { content: public_ip },
                },
            };
            let update_response = match client.request(&update_request).await {
                Ok(r) => r,
                Err(e) => {
                    error!("Failed to update record \"{}\" with {e}", dns.name);
                    continue;
                }
            };
            let update_name = update_response.result.name;
            let update_time = update_response.result.modified_on;
            let since_last_update = update_time - last_update_time;
            let since_last_update = since_last_update.num_hours();
            info!("Record \"{update_name}\" updated, hours since last update: {since_last_update}");
        }
    }
}
