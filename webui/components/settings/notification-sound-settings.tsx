'use client';

import { useNotificationSound, previewSound, type SoundType } from '@/hooks/useNotificationSound';
import { useBrowserNotifications } from '@/hooks/useBrowserNotifications';
import { Volume2, VolumeX, Bell, BellOff, Play, Globe } from 'lucide-react';

const SOUND_OPTIONS: { value: SoundType; label: string }[] = [
  { value: 'default', label: 'Default' },
  { value: 'chime', label: 'Chime' },
  { value: 'bell', label: 'Bell' },
  { value: 'pop', label: 'Pop' },
  { value: 'ding', label: 'Ding' },
  { value: 'none', label: 'None' },
];

function SoundSelector({
  label,
  value,
  onChange,
  disabled,
}: {
  label: string;
  value: SoundType;
  onChange: (v: SoundType) => void;
  disabled: boolean;
}) {
  return (
    <div className="flex items-center justify-between gap-4">
      <span className="text-sm text-gray-300">{label}</span>
      <div className="flex items-center gap-2">
        <select
          value={value}
          onChange={(e) => onChange(e.target.value as SoundType)}
          disabled={disabled}
          className="rounded-lg border border-gray-700 bg-gray-800 px-3 py-1.5 text-sm text-white focus:border-blue-500 focus:outline-none disabled:opacity-50"
        >
          {SOUND_OPTIONS.map((opt) => (
            <option key={opt.value} value={opt.value}>
              {opt.label}
            </option>
          ))}
        </select>
        <button
          onClick={() => previewSound(value)}
          disabled={disabled || value === 'none'}
          className="rounded-lg p-1.5 text-gray-400 hover:bg-gray-700 hover:text-white disabled:opacity-30 disabled:cursor-not-allowed transition-colors"
          title="Preview sound"
        >
          <Play className="w-4 h-4" />
        </button>
      </div>
    </div>
  );
}

export default function NotificationSoundSettings() {
  const { settings, updateSettings } = useNotificationSound();
  const { enabled: browserEnabled, permissionState, toggle: toggleBrowser } = useBrowserNotifications();

  return (
    <div className="space-y-6">
      <p className="text-sm text-gray-400">
        Configure notification sounds and browser notifications.
      </p>

      {/* Sound Notifications */}
      <div className="bg-gray-900 rounded-xl border border-gray-800 p-5">
        <div className="flex items-start justify-between gap-4">
          <div className="flex items-start gap-4 flex-1">
            <div className={`p-2.5 rounded-lg ${settings.enabled ? 'bg-blue-500/10' : 'bg-gray-500/10'}`}>
              {settings.enabled ? (
                <Volume2 className="w-5 h-5 text-blue-400" />
              ) : (
                <VolumeX className="w-5 h-5 text-gray-400" />
              )}
            </div>
            <div className="flex-1">
              <h3 className="text-white font-medium">Notification Sounds</h3>
              <p className="text-sm text-gray-400 mt-1">
                Play a sound when new messages arrive.
              </p>
            </div>
          </div>

          <button
            onClick={() => updateSettings({ enabled: !settings.enabled })}
            className={`
              relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full
              border-2 border-transparent transition-colors duration-200 ease-in-out
              focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 focus:ring-offset-gray-900
              ${settings.enabled ? 'bg-blue-600' : 'bg-gray-600'}
            `}
          >
            <span
              className={`
                pointer-events-none inline-block h-5 w-5 transform rounded-full
                bg-white shadow ring-0 transition duration-200 ease-in-out
                ${settings.enabled ? 'translate-x-5' : 'translate-x-0'}
              `}
            />
          </button>
        </div>

        {/* Sound options - shown when enabled */}
        {settings.enabled && (
          <div className="mt-5 space-y-4 border-t border-gray-800 pt-5">
            <SoundSelector
              label="Message sound"
              value={settings.messageSound}
              onChange={(v) => updateSettings({ messageSound: v })}
              disabled={!settings.enabled}
            />
            <SoundSelector
              label="Mention sound"
              value={settings.mentionSound}
              onChange={(v) => updateSettings({ mentionSound: v })}
              disabled={!settings.enabled}
            />

            {/* Volume slider */}
            <div className="flex items-center gap-4">
              <span className="text-sm text-gray-300">Volume</span>
              <div className="flex flex-1 items-center gap-3">
                <VolumeX className="w-4 h-4 text-gray-500 flex-shrink-0" />
                <input
                  type="range"
                  min={0}
                  max={1}
                  step={0.05}
                  value={settings.volume}
                  onChange={(e) => updateSettings({ volume: parseFloat(e.target.value) })}
                  className="flex-1 accent-blue-500 h-1.5 bg-gray-700 rounded-full appearance-none cursor-pointer [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-4 [&::-webkit-slider-thumb]:h-4 [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-blue-500 [&::-webkit-slider-thumb]:cursor-pointer"
                />
                <Volume2 className="w-4 h-4 text-gray-500 flex-shrink-0" />
              </div>
              <span className="text-xs text-gray-500 w-10 text-right">
                {Math.round(settings.volume * 100)}%
              </span>
            </div>
          </div>
        )}
      </div>

      {/* Browser Notifications */}
      <div className="bg-gray-900 rounded-xl border border-gray-800 p-5">
        <div className="flex items-start justify-between gap-4">
          <div className="flex items-start gap-4 flex-1">
            <div className={`p-2.5 rounded-lg ${browserEnabled ? 'bg-blue-500/10' : 'bg-gray-500/10'}`}>
              {browserEnabled ? (
                <Bell className="w-5 h-5 text-blue-400" />
              ) : (
                <BellOff className="w-5 h-5 text-gray-400" />
              )}
            </div>
            <div className="flex-1">
              <h3 className="text-white font-medium">Browser Notifications</h3>
              <p className="text-sm text-gray-400 mt-1">
                Show desktop notifications when you receive new messages while the tab is in the background.
              </p>
              {permissionState === 'denied' && (
                <p className="text-xs text-red-400 mt-2">
                  Notifications are blocked by your browser. Please update your browser settings to allow notifications for this site.
                </p>
              )}
            </div>
          </div>

          <button
            onClick={() => toggleBrowser(!browserEnabled)}
            disabled={permissionState === 'denied'}
            className={`
              relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full
              border-2 border-transparent transition-colors duration-200 ease-in-out
              focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 focus:ring-offset-gray-900
              ${browserEnabled ? 'bg-blue-600' : 'bg-gray-600'}
              ${permissionState === 'denied' ? 'opacity-50 cursor-not-allowed' : ''}
            `}
          >
            <span
              className={`
                pointer-events-none inline-block h-5 w-5 transform rounded-full
                bg-white shadow ring-0 transition duration-200 ease-in-out
                ${browserEnabled ? 'translate-x-5' : 'translate-x-0'}
              `}
            />
          </button>
        </div>

        {/* Tab badge info */}
        <div className="mt-4 pt-4 border-t border-gray-800">
          <div className="flex items-center gap-2 text-xs text-gray-500">
            <Globe className="w-3.5 h-3.5" />
            <span>Tab title and favicon badge are always active when there are unread messages.</span>
          </div>
        </div>
      </div>
    </div>
  );
}
