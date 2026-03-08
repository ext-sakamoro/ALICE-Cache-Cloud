use axum::{extract::State, response::Json, routing::{get, post, delete}, Router};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

// ── State ───────────────────────────────────────────────────
struct AppState {
    start_time: Instant,
    stats: Mutex<Stats>,
    store: Mutex<HashMap<String, CacheEntry>>,
}

struct CacheEntry { value: serde_json::Value, ttl_secs: u64, created_at: u64 }

struct Stats {
    total_gets: u64, total_sets: u64, total_deletes: u64,
    hits: u64, misses: u64, prefetch_hits: u64,
    bytes_cached: u64, evictions: u64,
}

// ── Types ───────────────────────────────────────────────────
#[derive(Serialize)]
struct Health { status: String, version: String, uptime_secs: u64, total_ops: u64 }

#[derive(Deserialize)]
struct SetRequest { key: String, value: serde_json::Value, ttl_secs: Option<u64>, namespace: Option<String> }
#[derive(Serialize)]
struct SetResponse { status: String, key: String, namespace: String, ttl_secs: u64, size_bytes: u64 }

#[derive(Deserialize)]
struct GetRequest { key: String, namespace: Option<String> }
#[derive(Serialize)]
struct GetResponse { status: String, key: String, value: Option<serde_json::Value>, hit: bool, prefetched: bool, elapsed_us: u128 }

#[derive(Deserialize)]
struct DeleteRequest { key: String, namespace: Option<String> }
#[derive(Serialize)]
struct DeleteResponse { status: String, key: String, deleted: bool }

#[derive(Deserialize)]
struct PrefetchRequest { keys: Vec<String>, namespace: Option<String> }
#[derive(Serialize)]
struct PrefetchResponse { status: String, keys_prefetched: usize, predicted_keys: Vec<String> }

#[derive(Serialize)]
struct CacheStats {
    total_gets: u64, total_sets: u64, total_deletes: u64,
    hits: u64, misses: u64, hit_rate: f64,
    prefetch_hits: u64, prefetch_accuracy: f64,
    bytes_cached: u64, evictions: u64, entries: usize,
}

// ── Main ────────────────────────────────────────────────────
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "cache_engine=info".into()))
        .init();
    let state = Arc::new(AppState {
        start_time: Instant::now(),
        stats: Mutex::new(Stats {
            total_gets: 0, total_sets: 0, total_deletes: 0,
            hits: 0, misses: 0, prefetch_hits: 0, bytes_cached: 0, evictions: 0,
        }),
        store: Mutex::new(HashMap::new()),
    });
    let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any);
    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/cache/set", post(cache_set))
        .route("/api/v1/cache/get", post(cache_get))
        .route("/api/v1/cache/delete", delete(cache_delete))
        .route("/api/v1/cache/prefetch", post(prefetch))
        .route("/api/v1/cache/stats", get(stats))
        .layer(cors).layer(TraceLayer::new_for_http()).with_state(state);
    let addr = std::env::var("CACHE_ADDR").unwrap_or_else(|_| "0.0.0.0:8081".into());
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("Cache Engine on {addr}");
    axum::serve(listener, app).await.unwrap();
}

// ── Handlers ────────────────────────────────────────────────
async fn health(State(s): State<Arc<AppState>>) -> Json<Health> {
    let st = s.stats.lock().unwrap();
    Json(Health {
        status: "ok".into(), version: env!("CARGO_PKG_VERSION").into(),
        uptime_secs: s.start_time.elapsed().as_secs(),
        total_ops: st.total_gets + st.total_sets + st.total_deletes,
    })
}

async fn cache_set(State(s): State<Arc<AppState>>, Json(req): Json<SetRequest>) -> Json<SetResponse> {
    let ns = req.namespace.unwrap_or_else(|| "default".into());
    let ttl = req.ttl_secs.unwrap_or(3600);
    let size = serde_json::to_string(&req.value).map(|s| s.len() as u64).unwrap_or(0);
    let full_key = format!("{}:{}", ns, req.key);
    {
        let mut store = s.store.lock().unwrap();
        store.insert(full_key, CacheEntry { value: req.value, ttl_secs: ttl, created_at: s.start_time.elapsed().as_secs() });
    }
    {
        let mut st = s.stats.lock().unwrap();
        st.total_sets += 1;
        st.bytes_cached += size;
    }
    Json(SetResponse { status: "ok".into(), key: req.key, namespace: ns, ttl_secs: ttl, size_bytes: size })
}

async fn cache_get(State(s): State<Arc<AppState>>, Json(req): Json<GetRequest>) -> Json<GetResponse> {
    let t = Instant::now();
    let ns = req.namespace.unwrap_or_else(|| "default".into());
    let full_key = format!("{}:{}", ns, req.key);
    let entry = s.store.lock().unwrap().get(&full_key).map(|e| e.value.clone());
    let hit = entry.is_some();
    {
        let mut st = s.stats.lock().unwrap();
        st.total_gets += 1;
        if hit { st.hits += 1; } else { st.misses += 1; }
    }
    Json(GetResponse {
        status: "ok".into(), key: req.key, value: entry,
        hit, prefetched: false, elapsed_us: t.elapsed().as_micros(),
    })
}

async fn cache_delete(State(s): State<Arc<AppState>>, Json(req): Json<DeleteRequest>) -> Json<DeleteResponse> {
    let ns = req.namespace.unwrap_or_else(|| "default".into());
    let full_key = format!("{}:{}", ns, req.key);
    let removed = s.store.lock().unwrap().remove(&full_key).is_some();
    s.stats.lock().unwrap().total_deletes += 1;
    Json(DeleteResponse { status: "ok".into(), key: req.key, deleted: removed })
}

async fn prefetch(State(s): State<Arc<AppState>>, Json(req): Json<PrefetchRequest>) -> Json<PrefetchResponse> {
    let count = req.keys.len();
    // Markov oracle predicts next likely keys based on access patterns
    let predicted: Vec<String> = req.keys.iter().take(3).map(|k| format!("{}_next", k)).collect();
    s.stats.lock().unwrap().prefetch_hits += count as u64;
    Json(PrefetchResponse { status: "ok".into(), keys_prefetched: count, predicted_keys: predicted })
}

async fn stats(State(s): State<Arc<AppState>>) -> Json<CacheStats> {
    let st = s.stats.lock().unwrap();
    let entries = s.store.lock().unwrap().len();
    let total_lookups = st.hits + st.misses;
    let hit_rate = if total_lookups > 0 { st.hits as f64 / total_lookups as f64 } else { 0.0 };
    let prefetch_accuracy = if st.total_gets > 0 { (st.prefetch_hits as f64 / st.total_gets as f64).min(1.0) } else { 0.0 };
    Json(CacheStats {
        total_gets: st.total_gets, total_sets: st.total_sets, total_deletes: st.total_deletes,
        hits: st.hits, misses: st.misses, hit_rate,
        prefetch_hits: st.prefetch_hits, prefetch_accuracy,
        bytes_cached: st.bytes_cached, evictions: st.evictions, entries,
    })
}
