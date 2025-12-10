'use client';

import { useState, useEffect, useCallback } from 'react';
import { apiClient } from '@/lib/api';
import { User } from '@/lib/types';
import { Eye, EyeOff, CheckCircle, AlertCircle, RefreshCw } from 'lucide-react';

interface PrivacySettingsProps {
  user: User;
}

export default function PrivacySettings({ user }: PrivacySettingsProps) {
  const [disableReadReceipts, setDisableReadReceipts] = useState(false);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState<{ type: 'success' | 'error'; text: string } | null>(null);

  const loadSettings = useCallback(async () => {
    if (!user) return;
    setLoading(true);
    try {
      const userData = await apiClient.getUser(user.id);
      setDisableReadReceipts((userData as { disable_read_receipts?: boolean }).disable_read_receipts || false);
    } catch (err) {
      console.error('Failed to load settings:', err);
    } finally {
      setLoading(false);
    }
  }, [user]);

  useEffect(() => {
    loadSettings();
  }, [loadSettings]);

  const handleToggle = async () => {
    if (!user) return;

    const newValue = !disableReadReceipts;
    setDisableReadReceipts(newValue);
    setSaving(true);
    setMessage(null);

    try {
      await apiClient.updateUser(user.id, {
        disable_read_receipts: newValue,
      } as Parameters<typeof apiClient.updateUser>[1]);
      setMessage({ type: 'success', text: 'Settings saved' });
      setTimeout(() => setMessage(null), 3000);
    } catch (err) {
      console.error('Failed to save settings:', err);
      setDisableReadReceipts(!newValue); // Revert on error
      setMessage({ type: 'error', text: 'Failed to save settings' });
    } finally {
      setSaving(false);
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center py-12">
        <RefreshCw className="w-6 h-6 text-blue-400 animate-spin" />
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <p className="text-sm text-gray-400">
        Control how your information is shared with others.
      </p>

      {/* Status Message */}
      {message && (
        <div
          className={`flex items-center gap-3 p-4 rounded-lg ${
            message.type === 'success'
              ? 'bg-green-500/10 border border-green-500/20'
              : 'bg-red-500/10 border border-red-500/20'
          }`}
        >
          {message.type === 'success' ? (
            <CheckCircle className="w-5 h-5 text-green-400" />
          ) : (
            <AlertCircle className="w-5 h-5 text-red-400" />
          )}
          <span className={message.type === 'success' ? 'text-green-400' : 'text-red-400'}>
            {message.text}
          </span>
        </div>
      )}

      {/* Read Receipts Toggle */}
      <div className="bg-gray-900 rounded-xl border border-gray-800 p-5">
        <div className="flex items-start justify-between gap-4">
          <div className="flex items-start gap-4 flex-1">
            <div className={`p-2.5 rounded-lg ${disableReadReceipts ? 'bg-orange-500/10' : 'bg-blue-500/10'}`}>
              {disableReadReceipts ? (
                <EyeOff className="w-5 h-5 text-orange-400" />
              ) : (
                <Eye className="w-5 h-5 text-blue-400" />
              )}
            </div>
            <div className="flex-1">
              <h3 className="text-white font-medium">Read Receipts</h3>
              <p className="text-sm text-gray-400 mt-1">
                {disableReadReceipts
                  ? 'Read receipts are disabled. Others won\'t see when you\'ve read their messages, and you won\'t see when others have read yours.'
                  : 'Read receipts are enabled. Others can see when you\'ve read their messages.'}
              </p>
            </div>
          </div>

          <button
            onClick={handleToggle}
            disabled={saving}
            className={`
              relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full
              border-2 border-transparent transition-colors duration-200 ease-in-out
              focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 focus:ring-offset-gray-900
              ${disableReadReceipts ? 'bg-orange-500' : 'bg-blue-600'}
              ${saving ? 'opacity-50 cursor-not-allowed' : ''}
            `}
          >
            <span
              className={`
                pointer-events-none inline-block h-5 w-5 transform rounded-full
                bg-white shadow ring-0 transition duration-200 ease-in-out
                ${disableReadReceipts ? 'translate-x-5' : 'translate-x-0'}
              `}
            />
          </button>
        </div>

        <div className="mt-4 pt-4 border-t border-gray-800">
          <p className="text-xs text-gray-500">
            {disableReadReceipts ? 'Currently: Hidden' : 'Currently: Visible'}
          </p>
        </div>
      </div>
    </div>
  );
}
