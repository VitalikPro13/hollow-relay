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
- Message size limits (10 MB), per-peer room caps (100 rooms), binary frame rate limiting (20 tokens/sec, burst 100).
- Room membership is enforced — peers cannot send to rooms they haven't joined.
- TURN credentials are time-limited (1 hour TTL) and derived from an environment variable (`TURN_SECRET`), never hardcoded.

## Running

```bash
cargo build --release

# Minimal (no license keys, no TURN)
./target/release/hollow-relay --public-ip 1.2.3.4

# Full (with license keys + TURN)
TURN_SECRET=your_secret ./target/release/hollow-relay \
  --public-ip 1.2.3.4 \
  --domain relay.example.com \
  --keys-file keys.json
```

### CLI flags

| Flag | Default | Description |
|------|---------|-------------|
| `--http-port` | `8080` | HTTP + WebSocket listen port |
| `--public-ip` | *(required)* | Public IP of this server |
| `--domain` | `relay.anonlisten.com` | Domain for WSS advertisement |
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

Designed to sit behind Nginx with TLS termination:

```
Client (WSS :443) → Nginx (TLS) → hollow-relay (HTTP :8080)
```

## License

MIT
