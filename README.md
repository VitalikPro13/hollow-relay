# Hollow Relay

WebSocket relay and signaling server for **Hollow** — a fully distributed, encrypted communication platform.

This repository serves as the public hub for the Hollow project. The relay is open-source; the Hollow app itself is proprietary.

## Contributing & Issues

This is the place to interact with the Hollow project! You can:

- **Report bugs** for both the relay and the main Hollow app
- **Request features** or suggest improvements
- **Contribute** relay fixes and enhancements via pull requests
- **Discuss** ideas for the platform in the Issues tab

I want to build Hollow together with the community. If you find a bug in the app, hit a connection issue, or have an idea for a cool feature — open an issue here.

## What the relay does

The relay is a lightweight, stateless message router. It does **not** store messages, decrypt content, or hold user data. All it does is:

- **WebSocket rooms** — peers join named rooms and exchange end-to-end encrypted messages through the relay. The relay forwards opaque blobs; it cannot read them.
- **Signaling HTTP** — bootstrap peer discovery (`/register`, `/unregister`, `/bootstrap/{room}`) with Ed25519-signed requests.
- **TURN credential generation** — time-limited HMAC-SHA1 credentials for NAT traversal via coturn (`/turn-credentials`).
- **License key gating** — optional closed-alpha access control via a `keys.json` file, with hot-reload and active connection revocation.

## Security properties

- All WebSocket authentication uses Ed25519 signature verification with 60-second timestamp skew protection.
- Native TLS termination via `tokio-rustls` with automatic Let's Encrypt certificate hot-reload.
- Bounded per-peer message channels (cap 32) — prevents slow-client RAM bloat and slow-loris attacks.
- Message size limits (10 MB), per-peer room caps (100 rooms), binary frame rate limiting (20 tokens/sec, burst 100).
- Room membership is enforced — peers cannot send to rooms they haven't joined.
- TCP socket buffer tuning (8 KB rx/tx) for high connection density.
- TURN credentials are time-limited (1 hour TTL) and derived from an environment variable (`TURN_SECRET`), never hardcoded.

## Running

```bash
cargo build --release

# With TLS (production — requires Let's Encrypt certs)
./target/release/hollow-relay \
  --public-ip 1.2.3.4 \
  --port 443 \
  --tls-cert /etc/letsencrypt/live/relay.example.com/fullchain.pem \
  --tls-key /etc/letsencrypt/live/relay.example.com/privkey.pem

# Without TLS (local testing)
./target/release/hollow-relay \
  --public-ip 127.0.0.1 \
  --port 8080 \
  --no-tls

# With license keys + TURN
TURN_SECRET=your_secret ./target/release/hollow-relay \
  --public-ip 1.2.3.4 \
  --domain relay.example.com \
  --keys-file keys.json
```

### CLI flags

| Flag | Default | Description |
|------|---------|-------------|
| `--port` | `443` | Listen port |
| `--public-ip` | *(required)* | Public IP of this server |
| `--domain` | `relay.anonlisten.com` | Domain for WSS advertisement |
| `--tls-cert` | `/etc/letsencrypt/live/relay.anonlisten.com/fullchain.pem` | TLS certificate chain PEM |
| `--tls-key` | `/etc/letsencrypt/live/relay.anonlisten.com/privkey.pem` | TLS private key PEM |
| `--no-tls` | `false` | Disable TLS (plain HTTP, for local testing) |
| `--keys-file` | `keys.json` | Path to license keys JSON (optional) |

### License keys format (`keys.json`)

```json
{
  "enabled": true,
  "keys": ["key1", "key2", "key3"]
}
```

The file is hot-reloaded every 30 seconds. Removing a key revokes the active connection using it.

## Deployment

The relay terminates TLS natively — no reverse proxy needed:

```
Client (WSS :443) → hollow-relay (TLS via tokio-rustls)
```

For systemd, grant the binary permission to bind port 443:

```ini
[Service]
AmbientCapabilities=CAP_NET_BIND_SERVICE
```

TLS certificates are hot-reloaded every 6 hours, so certbot renewals are picked up automatically without restart.

## License

MIT
