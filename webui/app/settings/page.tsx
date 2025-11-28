'use client';

import { useState, useEffect } from 'react';
import { useRouter } from 'next/navigation';
import { useAuth } from '@/lib/auth';
import { apiClient } from '@/lib/api';
import DesktopLogin from '@/components/desktop-login';
import DeviceManagement from '@/components/settings/devices';
import WebhookManagement from '@/components/settings/webhooks';
import packageJson from '@/../package.json';

export default function SettingsPage() {
  const router = useRouter();
  const { user } = useAuth();
  const [disableReadReceipts, setDisableReadReceipts] = useState(false);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState<{ type: 'success' | 'error'; text: string } | null>(null);

  useEffect(() => {
    loadSettings();
  }, [user]);

  const loadSettings = async () => {
    if (!user) return;

    setLoading(true);
    try {
      const userData = await apiClient.getUser(user.id);
      // Assuming the user model has disable_read_receipts field
      setDisableReadReceipts((userData as any).disable_read_receipts || false);
    } catch (err) {
      console.error('Failed to load settings:', err);
    } finally {
      setLoading(false);
    }
  };

  const handleSave = async () => {
    if (!user) return;

    setSaving(true);
    setMessage(null);
    try {
      await apiClient.updateUser(user.id, {
        disable_read_receipts: disableReadReceipts,
      } as any);
      setMessage({ type: 'success', text: 'Settings saved successfully' });
    } catch (err) {
      console.error('Failed to save settings:', err);
      setMessage({ type: 'error', text: 'Failed to save settings' });
    } finally {
      setSaving(false);
    }
  };

  if (!user) {
    return null;
  }

  return (
    <div className="flex h-screen flex-col bg-gray-950">
      {/* Header */}
      <div className="border-b border-gray-800 bg-gray-900 p-4">
        <div className="flex items-center gap-4">
          <button
            onClick={() => router.back()}
            className="text-gray-400 hover:text-white"
            title="Go back"
          >
            <svg className="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
            </svg>
          </button>
          <h1 className="text-xl font-semibold text-white">Settings</h1>
        </div>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto p-6">
        <div className="mx-auto max-w-2xl space-y-8">
          {/* Privacy Section */}
          <div className="rounded-lg border border-gray-800 bg-gray-900 p-6">
            <h2 className="mb-4 text-lg font-semibold text-white">Privacy</h2>

            {loading ? (
              <div className="flex items-center justify-center py-8">
                <div className="h-8 w-8 animate-spin rounded-full border-b-2 border-blue-500"></div>
              </div>
            ) : (
              <div className="space-y-4">
                {/* Read Receipts Toggle */}
                <div className="flex items-start justify-between">
                  <div className="flex-1">
                    <div className="font-medium text-white">Disable Read Receipts</div>
                    <div className="mt-1 text-sm text-gray-400">
                      When enabled, others won't see when you've read their messages. Note: You also won't see when others have read your messages.
                    </div>
                  </div>
                  <button
                    onClick={() => setDisableReadReceipts(!disableReadReceipts)}
                    className={`relative ml-4 inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none ${
                      disableReadReceipts ? 'bg-blue-600' : 'bg-gray-700'
                    }`}
                  >
                    <span
                      className={`pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out ${
                        disableReadReceipts ? 'translate-x-5' : 'translate-x-0'
                      }`}
                    />
                  </button>
                </div>

                {/* Save Button */}
                <div className="flex items-center gap-3 pt-4">
                  <button
                    onClick={handleSave}
                    disabled={saving}
                    className="rounded-md bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700 disabled:opacity-50"
                  >
                    {saving ? 'Saving...' : 'Save Changes'}
                  </button>

                  {message && (
                    <div className={`text-sm ${message.type === 'success' ? 'text-green-400' : 'text-red-400'}`}>
                      {message.text}
                    </div>
                  )}
                </div>
              </div>
            )}
          </div>

          {/* Profile Section */}
          <div className="rounded-lg border border-gray-800 bg-gray-900 p-6">
            <h2 className="mb-4 text-lg font-semibold text-white">Profile</h2>
            <div className="space-y-3 text-sm text-gray-300">
              <div className="flex justify-between">
                <span className="text-gray-400">Display Name:</span>
                <span className="font-medium">{user.display_name}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-400">Email:</span>
                <span className="font-medium">{user.email}</span>
              </div>
            </div>
          </div>

          {/* Desktop App Section */}
          <div className="rounded-lg border border-gray-800 bg-gray-900 p-6">
            <h2 className="mb-6 text-lg font-semibold text-white">Desktop App</h2>
            <DesktopLogin />
          </div>

          {/* Device Management Section */}
          <div className="rounded-lg border border-gray-800 bg-gray-900 p-6">
            <DeviceManagement />
          </div>

          {/* Webhooks Section */}
          <div className="rounded-lg border border-gray-800 bg-gray-900 p-6">
            <WebhookManagement />
          </div>

          {/* About Section */}
          <div className="rounded-lg border border-gray-800 bg-gray-900 p-6">
            <h2 className="mb-4 text-lg font-semibold text-white">About</h2>
            <div className="space-y-3 text-sm text-gray-300">
              <div className="flex justify-between">
                <span className="text-gray-400">Version:</span>
                <span className="font-medium font-mono">{packageJson.version}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
