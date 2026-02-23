# ALICE Voice Cloud

Cloud voice processing powered by ALICE-Voice. Compress, decompress, and analyze audio with industrial-strength codecs via a simple REST API.

[![License: AGPL-3.0](https://img.shields.io/badge/License-AGPL%203.0-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)

## Status

| Check | Status |
|-------|--------|
| `cargo check` | passing |
| `tsc --noEmit` | passing |
| API health | `/health` |

## Quick Start

```bash
docker compose up -d
```

Frontend: http://localhost:3000
API Gateway: http://localhost:8080
Voice Engine: http://localhost:8081

## Architecture

```
Browser / Client
      |
      v
Frontend (Next.js)  :3000
      |
      v
API Gateway         :8080
      |
      v
Voice Engine        :8081
(ALICE-Voice core)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/api/v1/voice/compress` | Compress audio to target format |
| `POST` | `/api/v1/voice/decompress` | Decompress audio to WAV/PCM |
| `POST` | `/api/v1/voice/analyze` | Analyze audio metadata |
| `GET` | `/api/v1/voice/formats` | List supported formats |
| `GET` | `/health` | Service health check |

### compress

```json
POST /api/v1/voice/compress
{
  "format": "opus",
  "bitrate": 128000,
  "sample_rate": 48000
}
```

Response:
```json
{
  "job_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "completed",
  "output_format": "opus",
  "estimated_ratio": 0.12
}
```

### analyze

```json
POST /api/v1/voice/analyze
{
  "format": "opus"
}
```

Response:
```json
{
  "duration_ms": 3200,
  "sample_rate": 48000,
  "channels": 2,
  "bitrate": 128000,
  "codec": "opus"
}
```

## Supported Formats

| Format | Extension | Lossy |
|--------|-----------|-------|
| Opus | `.opus` | Yes |
| FLAC | `.flac` | No |
| WAV | `.wav` | No |
| AAC | `.aac` | Yes |
| MP3 | `.mp3` | Yes |
| Vorbis | `.ogg` | Yes |

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `VOICE_ADDR` | `0.0.0.0:8081` | Voice engine bind address |
| `NEXT_PUBLIC_API_URL` | `http://localhost:8080` | API gateway URL for frontend |

## License

AGPL-3.0. Commercial dual-license available — contact for pricing.
