import { useCallback, useSyncExternalStore } from 'react';

export type SoundType = 'default' | 'chime' | 'bell' | 'pop' | 'ding' | 'none';

export interface NotificationSoundSettings {
  enabled: boolean;
  messageSound: SoundType;
  mentionSound: SoundType;
  volume: number; // 0-1
}

const DEFAULT_SETTINGS: NotificationSoundSettings = {
  enabled: true,
  messageSound: 'default',
  mentionSound: 'bell',
  volume: 0.5,
};

const STORAGE_KEY = 'notificationSoundSettings';

// Sound definitions using Web Audio API parameters
const SOUND_DEFS: Record<Exclude<SoundType, 'none'>, { freq: number[]; type: OscillatorType; durations: number[]; gaps?: number[] }> = {
  default: { freq: [800], type: 'sine', durations: [0.15] },
  chime: { freq: [523, 659, 784], type: 'sine', durations: [0.12, 0.12, 0.18], gaps: [0.05, 0.05] },
  bell: { freq: [880, 660], type: 'sine', durations: [0.2, 0.3], gaps: [0.02] },
  pop: { freq: [400, 600], type: 'triangle', durations: [0.05, 0.08], gaps: [0.01] },
  ding: { freq: [1200], type: 'sine', durations: [0.35] },
};

let cachedSettings: NotificationSoundSettings | null = null;
const listeners = new Set<() => void>();

function getSettings(): NotificationSoundSettings {
  if (cachedSettings) return cachedSettings;
  if (typeof window === 'undefined') return DEFAULT_SETTINGS;
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      cachedSettings = { ...DEFAULT_SETTINGS, ...JSON.parse(stored) };
    } else {
      // Migrate from old key
      const oldEnabled = localStorage.getItem('notificationSoundEnabled');
      cachedSettings = {
        ...DEFAULT_SETTINGS,
        enabled: oldEnabled === null ? true : oldEnabled === 'true',
      };
    }
  } catch {
    cachedSettings = DEFAULT_SETTINGS;
  }
  return cachedSettings!;
}

function setSettings(next: NotificationSoundSettings) {
  cachedSettings = next;
  if (typeof window !== 'undefined') {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(next));
  }
  listeners.forEach((l) => l());
}

function subscribe(listener: () => void) {
  listeners.add(listener);
  return () => { listeners.delete(listener); };
}

function getSnapshot() {
  return getSettings();
}

function getServerSnapshot() {
  return DEFAULT_SETTINGS;
}

/** Play a specific sound type at a given volume using Web Audio API */
function playSoundType(soundType: Exclude<SoundType, 'none'>, volume: number) {
  try {
    const ctx = new (window.AudioContext || (window as unknown as { webkitAudioContext: typeof AudioContext }).webkitAudioContext)();
    const def = SOUND_DEFS[soundType];
    let offset = 0;

    for (let i = 0; i < def.freq.length; i++) {
      const oscillator = ctx.createOscillator();
      const gain = ctx.createGain();
      oscillator.connect(gain);
      gain.connect(ctx.destination);

      oscillator.type = def.type;
      oscillator.frequency.value = def.freq[i];

      const dur = def.durations[i];
      gain.gain.setValueAtTime(volume * 0.6, ctx.currentTime + offset);
      gain.gain.exponentialRampToValueAtTime(0.001, ctx.currentTime + offset + dur);

      oscillator.start(ctx.currentTime + offset);
      oscillator.stop(ctx.currentTime + offset + dur + 0.01);

      offset += dur + (def.gaps?.[i] ?? 0);
    }

    // Close context after sounds finish
    setTimeout(() => ctx.close(), (offset + 0.5) * 1000);
  } catch (err) {
    console.error('Failed to play notification sound:', err);
  }
}

/** Play a sound for the given category (message or mention) respecting settings */
export function playSound(category: 'message' | 'mention') {
  const settings = getSettings();
  if (!settings.enabled) return;
  const soundType = category === 'mention' ? settings.mentionSound : settings.messageSound;
  if (soundType === 'none') return;
  playSoundType(soundType, settings.volume);
}

/** Preview a sound type at the current volume (ignores enabled flag) */
export function previewSound(soundType: SoundType) {
  if (soundType === 'none') return;
  const settings = getSettings();
  playSoundType(soundType, settings.volume);
}

export function useNotificationSound() {
  const settings = useSyncExternalStore(subscribe, getSnapshot, getServerSnapshot);

  const updateSettings = useCallback((partial: Partial<NotificationSoundSettings>) => {
    setSettings({ ...getSettings(), ...partial });
  }, []);

  return {
    settings,
    updateSettings,
    playSound,
    previewSound,
  };
}
