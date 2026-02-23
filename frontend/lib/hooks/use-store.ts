import { create } from 'zustand';

interface VoiceState {
  format: string;
  bitrate: number;
  result: unknown | null;
  loading: boolean;
  setFormat: (f: string) => void;
  setBitrate: (b: number) => void;
  setResult: (r: unknown | null) => void;
  setLoading: (l: boolean) => void;
}

export const useVoiceStore = create<VoiceState>((set) => ({
  format: 'opus',
  bitrate: 128000,
  result: null,
  loading: false,
  setFormat: (format) => set({ format }),
  setBitrate: (bitrate) => set({ bitrate }),
  setResult: (result) => set({ result }),
  setLoading: (loading) => set({ loading }),
}));
