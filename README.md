# ALICE-Cache-Cloud

Sub-millisecond key-value cache with namespace isolation, Markov-oracle predictive prefetching, TTL support, and real-time hit/miss analytics.

## Architecture

```
Client
  │
  ▼
┌─────────────────────────────────────────┐
│          ALICE-Cache-Cloud API          │
│         (Rust / Axum, port 8081)        │
└────────────┬────────────────┬───────────┘
             │                │
    ┌─────────────┐   ┌──────────────────┐
    │  Key-Value  │   │  Markov-Oracle    │
    │  Store      │   │  Prefetch Engine  │
    └──────┬──────┘   └────────┬─────────┘
           │                   │
  ┌────────▼───────────────────▼────────┐
  │          Namespace Manager          │
  │  (Isolation, TTL, Eviction Policy)  │
  └────────────────────────────────────┘
           │
  ┌────────▼────────────────────────────┐
  │   Analytics Collector               │
  │   (Hit/Miss rates, Memory usage)    │
  └─────────────────────────────────────┘
```

## Features

| Feature | Details |
|---------|---------|
| Namespaced Storage | Independent TTL policies and eviction per namespace |
| Markov-Oracle Prefetch | Learns access patterns and pre-warms cache entries |
| TTL Support | Fixed or sliding-window expiry from 1s to 30 days |
| Hit/Miss Tracking | Per-namespace and per-key real-time counters |
| Eviction Policies | LRU, LFU, and TTL-based eviction strategies |

## API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/cache/set` | Set a key with value, namespace, and optional TTL |
| POST | `/api/v1/cache/get` | Retrieve a key from a given namespace |
| DELETE | `/api/v1/cache/delete` | Delete a key from a namespace |
| POST | `/api/v1/cache/prefetch` | Trigger Markov-oracle prefetch for a list of keys |
| GET | `/api/v1/cache/stats` | Hit/miss rates, memory usage, namespace breakdown |

## Quick Start

```bash
# Clone and start the backend
git clone https://github.com/your-org/ALICE-Cache-Cloud.git
cd ALICE-Cache-Cloud
cargo run --release

# In a second terminal, start the frontend
cd frontend
npm install
npm run dev
# Open http://localhost:3000
```

### Example: Set a Key

```bash
curl -X POST http://localhost:8081/api/v1/cache/set \
  -H "Content-Type: application/json" \
  -d '{"namespace":"default","key":"user:1","value":"{\"name\":\"Alice\"}","ttl_secs":3600}'
```

### Example: Get a Key

```bash
curl -X POST http://localhost:8081/api/v1/cache/get \
  -H "Content-Type: application/json" \
  -d '{"namespace":"default","key":"user:1"}'
```

### Example: Trigger Prefetch

```bash
curl -X POST http://localhost:8081/api/v1/cache/prefetch \
  -H "Content-Type: application/json" \
  -d '{"namespace":"default","keys":["user:1","user:2","user:3"]}'
```

## License

AGPL-3.0-or-later — see [LICENSE](LICENSE) for details.
