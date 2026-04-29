# Hollow Relay

High-performance WebSocket relay and signaling server for **Hollow** — a fully distributed, encrypted communication platform.

Built with [uWebSockets](https://github.com/uNetworking/uWebSockets) (C++) for maximum connection density. A single $8/month VPS handles **~480,000 concurrent connections** at 14.5 KB per connection with native TLS.

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
- **Binary protocol** — `0x01` prefix for room broadcasts (e.g. encrypted messages), `0x02` prefix for targeted peer-to-peer delivery (e.g. file transfers, WebRTC signaling). The relay rewrites the target field to the sender field on forwarding.
- **Signaling HTTP** — bootstrap peer discovery (`/register`, `/unregister`, `/bootstrap/{room}`) with Ed25519-signed requests.
- **TURN credential generation** — time-limited HMAC-SHA1 credentials for NAT traversal via coturn (`/turn-credentials`).
- **License key gating** — optional closed-alpha access control via a `keys.json` file, with 30-second hot-reload and active connection revocation.
- **Server stats** — live memory, bandwidth, and online user count via `/server-stats` (reads `/proc` on Linux).

## Performance

Measured on an OVH VPS (4 vCPU / 8 GB RAM / 400 Mbps):

| Metric | Value |
|---|---|
| Per-connection memory | **14.5 KB** |
| Connections on 8 GB VPS | **~480,000** |
| Connections on 12 GB VPS | **~750,000** |
| Idle relay RSS | **10.5 MB** |
| Binary size | **636 KB** |
| Threads | **1** (single-threaded epoll) |
| Auth throughput | **800+/sec** |

Key: `SSL_MODE_RELEASE_BUFFERS` frees OpenSSL's 16 KB read/write buffers between messages, cutting per-connection cost in half for idle connections.

## Security properties

- All WebSocket authentication uses **Ed25519 signature verification** with 60-second timestamp skew protection.
- **Native TLS** via OpenSSL (TLS 1.3, AES-256-GCM) — no reverse proxy needed.
- **Backpressure handling** — 64 KB soft cap per socket (`getBufferedAmount()`), 256 KB hard ceiling. Slow consumers get messages dropped (clients auto-resync via CRDT/gossip).
- **Message size limits** (10 MB), per-peer room caps (100 rooms), binary frame rate limiting (20 tokens/sec, burst 100).
- **Room membership enforcement** — peers cannot send to rooms they haven't joined.
- TURN credentials are time-limited (1 hour TTL) and derived from an environment variable (`TURN_SECRET`), never hardcoded.

## Building

### Dependencies (Ubuntu/Debian)

```bash
sudo apt install cmake g++ libssl-dev libsodium-dev zlib1g-dev
```

### Build

```bash
git clone --recursive https://github.com/VitalikPro13/hollow-relay.git
cd hollow-relay
mkdir build && cd build
cmake .. -DCMAKE_BUILD_TYPE=Release
make -j$(nproc)
```

The output is a single binary: `build/hollow-relay` (~636 KB).

## Running

```bash
# With TLS (production)
./build/hollow-relay \
  --port 443 \
  --public-ip 1.2.3.4 \
  --cert-file /etc/letsencrypt/live/relay.example.com/fullchain.pem \
  --key-file /etc/letsencrypt/live/relay.example.com/privkey.pem

# With license keys + TURN
TURN_SECRET=your_secret ./build/hollow-relay \
  --port 443 \
  --public-ip 1.2.3.4 \
  --keys-file keys.json \
  --cert-file /path/to/fullchain.pem \
  --key-file /path/to/privkey.pem
```

### CLI flags

| Flag | Default | Description |
|------|---------|-------------|
| `--port` | `443` | Listen port |
| `--public-ip` | *(none)* | Public IP of this server |
| `--domain` | `relay.anonlisten.com` | Domain name |
| `--keys-file` | `keys.json` | License keys JSON path |
| `--cert-file` | `/etc/letsencrypt/live/relay.anonlisten.com/fullchain.pem` | TLS certificate chain |
| `--key-file` | `/etc/letsencrypt/live/relay.anonlisten.com/privkey.pem` | TLS private key |

### License keys format (`keys.json`)

```json
{
  "enabled": true,
  "keys": ["key1", "key2", "key3"]
}
```

The file is hot-reloaded every 30 seconds. Removing a key revokes the active connection using it.

## Deployment

The relay terminates TLS natively — no Nginx or reverse proxy needed:

```
Client (WSS :443) --> hollow-relay (TLS via OpenSSL)
```

Grant the binary permission to bind port 443 without root:

```bash
sudo setcap cap_net_bind_service=+ep ./build/hollow-relay
```

A sample systemd service file is provided in `deploy/hollow-relay.service`.

For certificate renewal, use certbot with a deploy hook:

```bash
# /etc/letsencrypt/renewal-hooks/deploy/reload-relay.sh
#!/bin/bash
systemctl restart hollow-relay
```

## Architecture

```
src/
  main.cpp           Entry point, CLI parsing, timer setup
  config.h           Config struct
  state.h            All shared state (single-threaded, no locks)
  crypto.h/.cpp      Ed25519 (libsodium), HMAC-SHA1 (OpenSSL), base64
  license.h/.cpp     License key load/validate/hot-reload/revocation
  http_handlers.h/.cpp  7 HTTP endpoints
  ws_handler.h/.cpp  WebSocket auth, room routing, binary protocol
  json.hpp           nlohmann/json (vendored single-header)
```

The relay is single-threaded by design. uWebSockets' epoll event loop handles all connections on one thread with correct backpressure and write draining. No mutexes, no atomics, no race conditions. For multi-core scaling, run multiple instances behind `SO_REUSEPORT`.

## Legacy Rust relay

The `legacy-rust/` directory contains the original Rust implementation (Axum/tokio/tungstenite). It served as the production relay through early development but was superseded by the C++ rewrite for a 12x connection density improvement (175 KB/conn to 14.5 KB/conn). The Rust code is preserved for reference but is no longer maintained.

## License

MIT
