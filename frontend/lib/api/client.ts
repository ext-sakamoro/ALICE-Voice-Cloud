const BASE = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';

async function authFetch(path: string, init?: RequestInit) {
  const headers: Record<string, string> = { 'Content-Type': 'application/json', ...(init?.headers as Record<string, string>) };
  const token = typeof window !== 'undefined' ? document.cookie.match(/sb-access-token=([^;]+)/)?.[1] : undefined;
  if (token) headers['Authorization'] = `Bearer ${token}`;
  return fetch(`${BASE}${path}`, { ...init, headers });
}

export const VoiceClient = {
  compress: (data: { format?: string; bitrate?: number }) => authFetch('/api/v1/voice/compress', { method: 'POST', body: JSON.stringify(data) }).then(r => r.json()),
  decompress: (data: { format?: string }) => authFetch('/api/v1/voice/decompress', { method: 'POST', body: JSON.stringify(data) }).then(r => r.json()),
  analyze: (data: { format?: string }) => authFetch('/api/v1/voice/analyze', { method: 'POST', body: JSON.stringify(data) }).then(r => r.json()),
  formats: () => authFetch('/api/v1/voice/formats').then(r => r.json()),
};
