#![allow(dead_code)]
use axum::{extract::State, response::Json, routing::{get, post}, Router};
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
}

struct Stats {
    total_compressions: u64,
    total_decompressions: u64,
    total_analyses: u64,
    total_tts: u64,
    total_stt: u64,
    bytes_processed: u64,
}

// ── Types ───────────────────────────────────────────────────
#[derive(Serialize)]
struct Health { status: String, version: String, uptime_secs: u64, total_jobs: u64 }

// Compress
#[derive(Deserialize)]
struct CompressRequest { format: Option<String>, bitrate: Option<u32>, sample_rate: Option<u32>, channels: Option<u8>, duration_ms: Option<u64> }
#[derive(Serialize)]
struct CompressResponse {
    job_id: String, status: String, output_format: String,
    original_size_bytes: u64, compressed_size_bytes: u64,
    compression_ratio: f64, bitrate_kbps: u32, elapsed_us: u128,
}

// Decompress
#[derive(Deserialize)]
struct DecompressRequest { format: Option<String>, target_sample_rate: Option<u32>, target_channels: Option<u8> }
#[derive(Serialize)]
struct DecompressResponse {
    job_id: String, status: String, output_format: String,
    output_sample_rate: u32, output_channels: u8,
    output_size_bytes: u64, elapsed_us: u128,
}

// Analyze
#[derive(Deserialize)]
struct AnalyzeRequest { format: Option<String>, duration_ms: Option<u64> }
#[derive(Serialize)]
struct AnalyzeResponse {
    duration_ms: u64, sample_rate: u32, channels: u8, bitrate_kbps: u32,
    codec: String, bit_depth: u8, peak_db: f64, rms_db: f64,
    silence_pct: f64, clipping_detected: bool,
    frequency_bands: HashMap<String, f64>,
}

// TTS
#[derive(Deserialize)]
struct TtsRequest { text: String, voice: Option<String>, language: Option<String>, speed: Option<f64> }
#[derive(Serialize)]
struct TtsResponse {
    job_id: String, status: String, voice: String, language: String,
    text_length: usize, estimated_duration_ms: u64,
    audio_format: String, sample_rate: u32, elapsed_us: u128,
}

// STT
#[derive(Deserialize)]
#[allow(dead_code)]
struct SttRequest { format: Option<String>, language: Option<String>, duration_ms: Option<u64> }
#[derive(Serialize)]
struct SttResponse {
    job_id: String, status: String, language: String,
    transcript: String, confidence: f64, word_count: usize,
    duration_ms: u64, elapsed_us: u128,
}

// Formats
#[derive(Serialize)]
struct FormatInfo { name: String, extension: String, lossy: bool, typical_bitrate_kbps: u32, description: String }

// Stats
#[derive(Serialize)]
struct StatsResponse {
    total_compressions: u64, total_decompressions: u64, total_analyses: u64,
    total_tts: u64, total_stt: u64, bytes_processed: u64,
}

// ── Main ────────────────────────────────────────────────────
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "voice_engine=info".into()))
        .init();
    let state = Arc::new(AppState {
        start_time: Instant::now(),
        stats: Mutex::new(Stats {
            total_compressions: 0, total_decompressions: 0, total_analyses: 0,
            total_tts: 0, total_stt: 0, bytes_processed: 0,
        }),
    });
    let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any);
    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/voice/compress", post(compress))
        .route("/api/v1/voice/decompress", post(decompress))
        .route("/api/v1/voice/analyze", post(analyze))
        .route("/api/v1/voice/tts", post(tts))
        .route("/api/v1/voice/stt", post(stt))
        .route("/api/v1/voice/formats", get(formats))
        .route("/api/v1/voice/stats", get(stats))
        .layer(cors).layer(TraceLayer::new_for_http()).with_state(state);
    let addr = std::env::var("VOICE_ADDR").unwrap_or_else(|_| "0.0.0.0:8081".into());
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("Voice Engine on {addr}");
    axum::serve(listener, app).await.unwrap();
}

// ── Handlers ────────────────────────────────────────────────
async fn health(State(s): State<Arc<AppState>>) -> Json<Health> {
    let st = s.stats.lock().unwrap();
    Json(Health {
        status: "ok".into(), version: env!("CARGO_PKG_VERSION").into(),
        uptime_secs: s.start_time.elapsed().as_secs(),
        total_jobs: st.total_compressions + st.total_decompressions + st.total_analyses + st.total_tts + st.total_stt,
    })
}

async fn compress(State(s): State<Arc<AppState>>, Json(req): Json<CompressRequest>) -> Json<CompressResponse> {
    let t = Instant::now();
    let fmt = req.format.unwrap_or_else(|| "opus".into());
    let sample_rate = req.sample_rate.unwrap_or(48000);
    let channels = req.channels.unwrap_or(2) as u64;
    let duration_ms = req.duration_ms.unwrap_or(5000);
    let bit_depth = 16u64;

    // PCM original size
    let original_size = sample_rate as u64 * channels * (bit_depth / 8) * duration_ms / 1000;

    // Compression ratio depends on codec and target bitrate
    let target_bitrate = req.bitrate.unwrap_or_else(|| codec_default_bitrate(&fmt));
    let compressed_size = (target_bitrate as u64 / 8) * duration_ms / 1000;
    let ratio = if compressed_size > 0 { original_size as f64 / compressed_size as f64 } else { 1.0 };

    {
        let mut st = s.stats.lock().unwrap();
        st.total_compressions += 1;
        st.bytes_processed += original_size;
    }

    Json(CompressResponse {
        job_id: uuid::Uuid::new_v4().to_string(), status: "completed".into(),
        output_format: fmt, original_size_bytes: original_size,
        compressed_size_bytes: compressed_size, compression_ratio: ratio,
        bitrate_kbps: target_bitrate / 1000, elapsed_us: t.elapsed().as_micros(),
    })
}

async fn decompress(State(s): State<Arc<AppState>>, Json(req): Json<DecompressRequest>) -> Json<DecompressResponse> {
    let t = Instant::now();
    let fmt = req.format.unwrap_or_else(|| "wav".into());
    let sample_rate = req.target_sample_rate.unwrap_or(48000);
    let channels = req.target_channels.unwrap_or(2);

    // Estimate output PCM size (5 seconds default)
    let output_size = sample_rate as u64 * channels as u64 * 2 * 5;

    s.stats.lock().unwrap().total_decompressions += 1;

    Json(DecompressResponse {
        job_id: uuid::Uuid::new_v4().to_string(), status: "completed".into(),
        output_format: fmt, output_sample_rate: sample_rate,
        output_channels: channels, output_size_bytes: output_size,
        elapsed_us: t.elapsed().as_micros(),
    })
}

async fn analyze(State(s): State<Arc<AppState>>, Json(req): Json<AnalyzeRequest>) -> Json<AnalyzeResponse> {
    let fmt = req.format.as_deref().unwrap_or("opus");
    let duration = req.duration_ms.unwrap_or(5000);
    let bitrate = codec_default_bitrate(fmt);
    let sample_rate = if fmt == "mp3" || fmt == "aac" { 44100 } else { 48000 };

    // Simulate audio analysis with deterministic values based on format hash
    let hash = simple_hash(fmt);
    let peak = -0.5 - (hash % 100) as f64 * 0.1;
    let rms = peak - 6.0 - (hash % 50) as f64 * 0.1;
    let silence_pct = (hash % 15) as f64 + 2.0;

    let mut bands = HashMap::new();
    bands.insert("sub_bass_20_60".into(), -12.0 + (hash % 8) as f64);
    bands.insert("bass_60_250".into(), -6.0 + (hash % 6) as f64);
    bands.insert("low_mid_250_500".into(), -3.0 + (hash % 4) as f64);
    bands.insert("mid_500_2k".into(), 0.0 + (hash % 3) as f64);
    bands.insert("upper_mid_2k_4k".into(), -2.0 + (hash % 5) as f64);
    bands.insert("presence_4k_6k".into(), -4.0 + (hash % 7) as f64);
    bands.insert("brilliance_6k_20k".into(), -8.0 + (hash % 10) as f64);

    s.stats.lock().unwrap().total_analyses += 1;

    Json(AnalyzeResponse {
        duration_ms: duration, sample_rate, channels: 2,
        bitrate_kbps: bitrate / 1000, codec: fmt.into(),
        bit_depth: 16, peak_db: peak, rms_db: rms,
        silence_pct, clipping_detected: peak > -0.3,
        frequency_bands: bands,
    })
}

async fn tts(State(s): State<Arc<AppState>>, Json(req): Json<TtsRequest>) -> Json<TtsResponse> {
    let t = Instant::now();
    let voice = req.voice.unwrap_or_else(|| "alice-neural-v2".into());
    let language = req.language.unwrap_or_else(|| "en-US".into());
    let speed = req.speed.unwrap_or(1.0);
    let text_len = req.text.len();

    // Estimate: ~150 words per minute at speed 1.0, average word = 5 chars
    let word_count = text_len as f64 / 5.0;
    let duration_ms = ((word_count / 150.0) * 60000.0 / speed) as u64;

    s.stats.lock().unwrap().total_tts += 1;

    Json(TtsResponse {
        job_id: uuid::Uuid::new_v4().to_string(), status: "completed".into(),
        voice, language, text_length: text_len,
        estimated_duration_ms: duration_ms.max(500),
        audio_format: "opus".into(), sample_rate: 24000,
        elapsed_us: t.elapsed().as_micros(),
    })
}

async fn stt(State(s): State<Arc<AppState>>, Json(req): Json<SttRequest>) -> Json<SttResponse> {
    let t = Instant::now();
    let language = req.language.unwrap_or_else(|| "en-US".into());
    let duration = req.duration_ms.unwrap_or(5000);

    // Simulate: ~2.5 words per second of audio
    let word_count = ((duration as f64 / 1000.0) * 2.5) as usize;
    let confidence = 0.85 + (simple_hash(&language) % 10) as f64 * 0.01;

    let transcript = generate_placeholder_transcript(word_count, &language);

    s.stats.lock().unwrap().total_stt += 1;

    Json(SttResponse {
        job_id: uuid::Uuid::new_v4().to_string(), status: "completed".into(),
        language, transcript, confidence: confidence.min(0.99),
        word_count, duration_ms: duration, elapsed_us: t.elapsed().as_micros(),
    })
}

async fn formats() -> Json<Vec<FormatInfo>> {
    Json(vec![
        FormatInfo { name: "Opus".into(), extension: "opus".into(), lossy: true, typical_bitrate_kbps: 64, description: "Modern codec, best quality-to-size ratio for speech and music".into() },
        FormatInfo { name: "FLAC".into(), extension: "flac".into(), lossy: false, typical_bitrate_kbps: 800, description: "Lossless compression, ~50-60% size reduction".into() },
        FormatInfo { name: "WAV".into(), extension: "wav".into(), lossy: false, typical_bitrate_kbps: 1411, description: "Uncompressed PCM, maximum quality".into() },
        FormatInfo { name: "AAC".into(), extension: "aac".into(), lossy: true, typical_bitrate_kbps: 128, description: "Apple ecosystem standard, good quality at 128-256 kbps".into() },
        FormatInfo { name: "MP3".into(), extension: "mp3".into(), lossy: true, typical_bitrate_kbps: 192, description: "Universal compatibility, adequate quality at 192+ kbps".into() },
        FormatInfo { name: "Vorbis".into(), extension: "ogg".into(), lossy: true, typical_bitrate_kbps: 96, description: "Open source alternative to MP3/AAC".into() },
        FormatInfo { name: "ALAC".into(), extension: "m4a".into(), lossy: false, typical_bitrate_kbps: 700, description: "Apple lossless, similar to FLAC".into() },
        FormatInfo { name: "Speex".into(), extension: "spx".into(), lossy: true, typical_bitrate_kbps: 24, description: "Optimized for speech at very low bitrates".into() },
    ])
}

async fn stats(State(s): State<Arc<AppState>>) -> Json<StatsResponse> {
    let st = s.stats.lock().unwrap();
    Json(StatsResponse {
        total_compressions: st.total_compressions, total_decompressions: st.total_decompressions,
        total_analyses: st.total_analyses, total_tts: st.total_tts, total_stt: st.total_stt,
        bytes_processed: st.bytes_processed,
    })
}

// ── Helpers ─────────────────────────────────────────────────
fn codec_default_bitrate(codec: &str) -> u32 {
    match codec {
        "opus" => 64_000,
        "aac" => 128_000,
        "mp3" => 192_000,
        "vorbis" | "ogg" => 96_000,
        "flac" => 800_000,
        "alac" | "m4a" => 700_000,
        "speex" | "spx" => 24_000,
        "wav" => 1_411_200,
        _ => 128_000,
    }
}

fn simple_hash(s: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in s.as_bytes() { h ^= b as u64; h = h.wrapping_mul(0x0100_0000_01b3); }
    h
}

fn generate_placeholder_transcript(word_count: usize, lang: &str) -> String {
    let words_en = ["the", "audio", "signal", "was", "processed", "through", "neural", "network", "voice", "recognition", "system", "detected", "speech", "patterns"];
    let words_ja = ["音声", "認識", "処理", "完了", "ニューラル", "ネットワーク", "信号", "検出", "パターン", "解析"];

    let words: &[&str] = if lang.starts_with("ja") { &words_ja } else { &words_en };
    let mut result = Vec::with_capacity(word_count);
    for i in 0..word_count {
        result.push(words[i % words.len()]);
    }
    result.join(" ")
}
