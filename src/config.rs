use clap::Parser;

#[derive(Parser)]
#[command(name = "hollow-relay", about = "Hollow signaling + WebSocket relay server")]
pub struct Config {
    /// Port to listen on (443 for TLS, 8080 for plain HTTP)
    #[arg(long, default_value = "443")]
    pub port: u16,

    /// Public IP address of this server
    #[arg(long)]
    pub public_ip: String,

    /// Domain name for WSS (used in external address advertisement)
    #[arg(long, default_value = "relay.anonlisten.com")]
    pub domain: String,

    /// Path to license keys JSON file (optional, keys disabled if missing)
    #[arg(long, default_value = "keys.json")]
    pub keys_file: String,

    /// TLS certificate chain PEM file (fullchain.pem from Let's Encrypt)
    #[arg(long, default_value = "/etc/letsencrypt/live/relay.anonlisten.com/fullchain.pem")]
    pub tls_cert: String,

    /// TLS private key PEM file (privkey.pem from Let's Encrypt)
    #[arg(long, default_value = "/etc/letsencrypt/live/relay.anonlisten.com/privkey.pem")]
    pub tls_key: String,

    /// Disable TLS and run plain HTTP (for local testing)
    #[arg(long, default_value = "false")]
    pub no_tls: bool,
}
