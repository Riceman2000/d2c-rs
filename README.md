# d2c-rs

Update Cloudflare DNS 'A' records with your public IP.

Created to be a drop in replacement for this tool: https://github.com/ddries/d2c.rs

> [!WARNING]  
> This code has not been published to Crates.io due to the dependency [`cloudflare-rs`](https://github.com/cloudflare/cloudflare-rs) requiring the latest git revision. 
> Once this dependency has been updated on Crates.io and the necessary patches published, d2c-rs will be published to Crates.io.
> Until then, the only installation method is to clone this repository and build via cargo.

---

d2c-rs (Dynamic DNS Cloudflare) is a very simple program to automatically update the IP address of A DNS records from Cloudflare using your current public IP. This tool is designed to be run regularly e.g. on a [cronjob](https://en.wikipedia.org/wiki/Cron).

## Project goals 
- Ease of use for the most simple use case of updating a DNS 'A' record for a single server
- "Fire and forget" configuration that is as simple as possible

## Project non-goals
- Adapting the entire Cloudflare DNS API
- Adding a ton of configuration options 
- Converting this to a daemon or service

### Configure

d2c-rs is configured using TOML files located in `/etc/d2c/`. The first time you run d2c-rs from the command-line, it will create the config directory for you. You will then need to manually create one or more TOML configuration files.

The script processes all files in `/etc/d2c/` directory that end with `.toml`, e.g. `/etc/d2c/d2c.toml`, `/etc/d2c/zone1.toml` or `/etc/d2c/zone2.toml`.

Syntax:

```toml
[api]
zone-id = "aaa" # your DNS zone ID
api-key = "bbb" # your API key with DNS records permissions

[[dns]]
name = "dns1.example.com" # DNS name
proxy = true              # Proxied by Cloudflare?

[[dns]]
name = "dns2.example.com"
proxy = false
```

When d2c-rs is run, it will process each `*.toml` TOML file in the `/etc/d2c/` directory, updating the records configured in each with the current public IP of the machine. The A records must be created from the Cloudflare dashboard first; then d2c-rs will be able to update them with the server's public IP. 

To run d2c-rs regularly, create a cronjob:

```sh
crontab -e # set cronjob to run d2c-rs periodically
```

### Usage
```sh
d2c-rs --help
```
Prints the following information:

```
d2c (Dynamic Dns Cloudflare): Update the Cloudflare DNS A records for your dynamic IP.

Usage: d2c-rs

`d2c` UPDATES existing records. Please, create them in Cloudflare Dashboard before running this script.

The configuration is done in `/etc/d2c/*.toml` files in TOML format.
Configuration file structure:

[api]
zone-id = "<zone id>"
api-key = "<api key>"

[[dns]]
name = "test.example.com"
proxy = false

[[dns]]
name = "test2.example.com"
proxy = true
```

For more verbosity, use the `-v` or `-vv` flags:

```sh 
d2c-rs -v
# Or
d2c-rs -vv
```

### Installation

#### From Crates.io

> [!WARNING]  
> This method currently does not work.

Install d2c-rs using Cargo:

```sh
cargo install d2c-rs
```

#### Manually from source

Clone this repository and ensure you have [rustup](https://rustup.rs/) installed.

```sh 
git clone https://github.com/Riceman2000/d2c-rs.git
cd d2c-rs
cargo install --path .
```
