'use client';

import { useState, useEffect } from 'react';
import { apiClient } from '@/lib/api';

interface RetentionPolicy {
  id: string;
  org_id: string;
  policy_type: string;
  retention_days: number;
  enabled: boolean;
  created_at: string;
  updated_at: string;
}

export default function RetentionPoliciesPage() {
  const [policies, setPolicies] = useState<RetentionPolicy[]>([]);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);

  const [messageRetentionDays, setMessageRetentionDays] = useState(365);
  const [messageRetentionEnabled, setMessageRetentionEnabled] = useState(false);
  const [fileRetentionDays, setFileRetentionDays] = useState(365);
  const [fileRetentionEnabled, setFileRetentionEnabled] = useState(false);

  useEffect(() => {
    fetchPolicies();
  }, []);

  const fetchPolicies = async () => {
    try {
      setLoading(true);
      setError(null);
      const response = await fetch(`${process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080'}/api/settings/retention`, {
        headers: {
          'Authorization': `Bearer ${apiClient.getToken()}`,
        },
      });

      if (!response.ok) {
        throw new Error('Failed to fetch retention policies');
      }

      const data = await response.json();
      setPolicies(data.policies);

      // Set form values from existing policies
      const messagePolicy = data.policies.find((p: RetentionPolicy) => p.policy_type === 'messages');
      const filePolicy = data.policies.find((p: RetentionPolicy) => p.policy_type === 'files');

      if (messagePolicy) {
        setMessageRetentionDays(messagePolicy.retention_days);
        setMessageRetentionEnabled(messagePolicy.enabled);
      }

      if (filePolicy) {
        setFileRetentionDays(filePolicy.retention_days);
        setFileRetentionEnabled(filePolicy.enabled);
      }
    } catch (err) {
      console.error('Failed to fetch policies:', err);
      setError('Failed to load retention policies');
    } finally {
      setLoading(false);
    }
  };

  const handleSavePolicy = async (policyType: 'messages' | 'files') => {
    setError(null);
    setSuccess(false);
    setSaving(true);

    const retentionDays = policyType === 'messages' ? messageRetentionDays : fileRetentionDays;
    const enabled = policyType === 'messages' ? messageRetentionEnabled : fileRetentionEnabled;

    // Validation
    if (retentionDays <= 0) {
      setError('Retention days must be greater than 0');
      setSaving(false);
      return;
    }

    try {
      const response = await fetch(`${process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080'}/api/settings/retention`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${apiClient.getToken()}`,
        },
        body: JSON.stringify({
          policy_type: policyType,
          retention_days: retentionDays,
          enabled: enabled,
        }),
      });

      if (!response.ok) {
        const errorData = await response.json();
        throw new Error(errorData.message || 'Failed to update retention policy');
      }

      setSuccess(true);
      await fetchPolicies();
    } catch (err: any) {
      console.error('Failed to save policy:', err);
      setError(err.message || 'Failed to save retention policy');
    } finally {
      setSaving(false);
    }
  };

  if (loading) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-black">
        <p className="text-gray-400">Loading retention policies...</p>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-black p-8">
      <div className="mx-auto max-w-4xl">
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-white">Data Retention Policies</h1>
          <p className="mt-2 text-gray-400">
            Configure how long messages and files are retained before automatic deletion
          </p>
        </div>

        {error && (
          <div className="mb-6 rounded-lg border border-red-800 bg-red-900 bg-opacity-20 p-4">
            <p className="text-red-400">{error}</p>
          </div>
        )}

        {success && (
          <div className="mb-6 rounded-lg border border-green-800 bg-green-900 bg-opacity-20 p-4">
            <p className="text-green-400">Retention policy updated successfully!</p>
          </div>
        )}

        <div className="space-y-6">
          {/* Message Retention Policy */}
          <div className="rounded-lg border border-gray-700 bg-gray-900 p-6">
            <h2 className="text-xl font-semibold text-white mb-4">Message Retention</h2>
            <p className="text-sm text-gray-400 mb-6">
              Automatically delete messages older than the specified retention period.
              Messages in channels with active legal holds will not be deleted.
            </p>

            <div className="space-y-4">
              <div>
                <label htmlFor="messageRetentionDays" className="block text-sm font-medium text-gray-300 mb-2">
                  Retention Period (days)
                </label>
                <input
                  type="number"
                  id="messageRetentionDays"
                  value={messageRetentionDays}
                  onChange={(e) => setMessageRetentionDays(parseInt(e.target.value) || 0)}
                  min="1"
                  className="w-full rounded-lg border border-gray-600 bg-gray-800 px-4 py-2 text-white focus:border-blue-500 focus:outline-none"
                />
                <p className="mt-1 text-xs text-gray-500">
                  Messages older than {messageRetentionDays} days will be permanently deleted
                </p>
              </div>

              <div className="flex items-center justify-between">
                <div>
                  <div className="font-medium text-white">Enable Message Retention</div>
                  <div className="text-sm text-gray-400">
                    Turn on automatic message deletion based on the retention period
                  </div>
                </div>
                <button
                  onClick={() => setMessageRetentionEnabled(!messageRetentionEnabled)}
                  className={`relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none ${
                    messageRetentionEnabled ? 'bg-blue-600' : 'bg-gray-700'
                  }`}
                >
                  <span
                    className={`pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out ${
                      messageRetentionEnabled ? 'translate-x-5' : 'translate-x-0'
                    }`}
                  />
                </button>
              </div>

              <button
                onClick={() => handleSavePolicy('messages')}
                disabled={saving}
                className="rounded-lg bg-blue-600 px-6 py-3 font-medium text-white transition-colors hover:bg-blue-700 disabled:bg-gray-700 disabled:cursor-not-allowed"
              >
                {saving ? 'Saving...' : 'Save Message Policy'}
              </button>
            </div>
          </div>

          {/* File Retention Policy */}
          <div className="rounded-lg border border-gray-700 bg-gray-900 p-6">
            <h2 className="text-xl font-semibold text-white mb-4">File Retention</h2>
            <p className="text-sm text-gray-400 mb-6">
              Automatically delete file attachments older than the specified retention period.
              Files in channels with active legal holds will not be deleted.
            </p>

            <div className="space-y-4">
              <div>
                <label htmlFor="fileRetentionDays" className="block text-sm font-medium text-gray-300 mb-2">
                  Retention Period (days)
                </label>
                <input
                  type="number"
                  id="fileRetentionDays"
                  value={fileRetentionDays}
                  onChange={(e) => setFileRetentionDays(parseInt(e.target.value) || 0)}
                  min="1"
                  className="w-full rounded-lg border border-gray-600 bg-gray-800 px-4 py-2 text-white focus:border-blue-500 focus:outline-none"
                />
                <p className="mt-1 text-xs text-gray-500">
                  Files older than {fileRetentionDays} days will be permanently deleted
                </p>
              </div>

              <div className="flex items-center justify-between">
                <div>
                  <div className="font-medium text-white">Enable File Retention</div>
                  <div className="text-sm text-gray-400">
                    Turn on automatic file deletion based on the retention period
                  </div>
                </div>
                <button
                  onClick={() => setFileRetentionEnabled(!fileRetentionEnabled)}
                  className={`relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none ${
                    fileRetentionEnabled ? 'bg-blue-600' : 'bg-gray-700'
                  }`}
                >
                  <span
                    className={`pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out ${
                      fileRetentionEnabled ? 'translate-x-5' : 'translate-x-0'
                    }`}
                  />
                </button>
              </div>

              <button
                onClick={() => handleSavePolicy('files')}
                disabled={saving}
                className="rounded-lg bg-blue-600 px-6 py-3 font-medium text-white transition-colors hover:bg-blue-700 disabled:bg-gray-700 disabled:cursor-not-allowed"
              >
                {saving ? 'Saving...' : 'Save File Policy'}
              </button>
            </div>
          </div>

          {/* Legal Hold Information */}
          <div className="rounded-lg border border-yellow-700 bg-yellow-900 bg-opacity-20 p-6">
            <h3 className="text-lg font-semibold text-yellow-400 mb-2 flex items-center">
              <svg className="h-5 w-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
              </svg>
              Legal Holds
            </h3>
            <p className="text-sm text-gray-300">
              Legal holds can be placed on individual channels to prevent automatic deletion of messages and files.
              To manage legal holds, go to the channel settings.
            </p>
            <p className="mt-2 text-xs text-gray-400">
              Note: Retention policies will not affect channels with active legal holds.
            </p>
          </div>

          {/* Warning */}
          <div className="rounded-lg border border-red-700 bg-red-900 bg-opacity-20 p-6">
            <h3 className="text-lg font-semibold text-red-400 mb-2">Warning</h3>
            <p className="text-sm text-gray-300">
              Once messages and files are deleted by retention policies, they cannot be recovered.
              Ensure your retention periods comply with your organization's data retention requirements and applicable laws.
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}
