'use client';
import { useState } from 'react';
export default function ConsolePage() {
  const [format, setFormat] = useState('opus');
  const [result, setResult] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const handleCompress = async () => {
    setLoading(true);
    try {
      const r = await fetch(`${process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/voice/compress`, {
        method: 'POST', headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ format, bitrate: 128000 }),
      });
      const data = await r.json();
      setResult(JSON.stringify(data, null, 2));
    } catch (e) { setResult(`Error: ${e}`); }
    finally { setLoading(false); }
  };
  return (
    <div className="p-6 space-y-6">
      <h1 className="text-2xl font-bold">Voice Console</h1>
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <div className="border border-border rounded-lg p-4 space-y-4">
          <h2 className="font-semibold">Compress Audio</h2>
          <div>
            <label className="text-sm font-medium">Output Format</label>
            <select value={format} onChange={(e) => setFormat(e.target.value)} className="mt-1 w-full px-3 py-2 border border-input rounded-md bg-background text-sm">
              <option value="opus">Opus</option><option value="flac">FLAC</option><option value="aac">AAC</option><option value="mp3">MP3</option>
            </select>
          </div>
          <button onClick={handleCompress} disabled={loading} className="px-4 py-2 bg-primary text-primary-foreground rounded-md text-sm font-medium hover:opacity-90 disabled:opacity-50">{loading ? 'Processing...' : 'Compress'}</button>
        </div>
        <div className="border border-border rounded-lg p-4 space-y-2">
          <h2 className="font-semibold">Result</h2>
          <pre className="bg-muted rounded-md p-3 text-xs font-mono overflow-auto max-h-64">{result || 'No result yet'}</pre>
        </div>
      </div>
    </div>
  );
}
