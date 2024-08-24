# d2c-rs

Update Cloudflare DNS 'A' records with your public IP.

Created to be a drop in replacement for this tool: https://github.com/ddries/d2c.rs

---

d2c-rs (Dynamic DNS Cloudflare) is a very simple program to automatically update the IP address of A DNS records from Cloudflare using your current public IP. This tool is designed to be run regularly e.g. on a [cronjob](https://en.wikipedia.org/wiki/Cron).

## Project goals 
- Ease of use for the most simple use case of updating a DNS 'A' record for a single server
- "Fire and forget" configuration that doesn't take forever

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

#### Installing d2c-rs

Install d2c-rs using Cargo:

```sh
cargo install d2c-rs
```

Then, run d2c-rs from command-line for the first time to create the configuration directory:

```sh
d2c-rs
```

Create configuration file(s) with your zone id, API key and the desired DNS records. Only one pair of `zone-id` and `api-key` in each file. If you have any other zones or another Cloudflare account you want to configure it is possible by creating additional TOML files.

```sh
sudo nano /etc/d2c/d2c.toml
```

```toml
[api]
zone-id = "aaa"
api-key = "bbb"

[[dns]]
name = "test.example.com"
proxy = false

[[dns]]
name = "test2.example.com"
proxy = true
```

Finally, you can run d2c-rs to update the DNS records specified in your configuration files:

```sh
d2c-rs
```
or create a cronjob:

```sh
crontab -e # set cronjob to run d2c-rs periodically
```
