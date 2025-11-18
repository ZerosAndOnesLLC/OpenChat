'use client';

import { useState, useEffect } from 'react';
import { apiClient } from '@/lib/api';

interface StorageSettings {
  org_id: string;
  storage_type: string;
  s3_bucket?: string | null;
  s3_region?: string | null;
  s3_endpoint?: string | null;
}

export default function StorageSettingsPage() {
  const [settings, setSettings] = useState<StorageSettings | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);

  const [storageType, setStorageType] = useState('local');
  const [s3Bucket, setS3Bucket] = useState('');
  const [s3Region, setS3Region] = useState('');
  const [s3AccessKeyId, setS3AccessKeyId] = useState('');
  const [s3SecretKey, setS3SecretKey] = useState('');
  const [s3Endpoint, setS3Endpoint] = useState('');

  useEffect(() => {
    fetchSettings();
  }, []);

  const fetchSettings = async () => {
    try {
      setLoading(true);
      const response = await fetch(`${process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080'}/api/settings/storage`, {
        headers: {
          'Authorization': `Bearer ${apiClient.getToken()}`,
        },
      });

      if (!response.ok) {
        throw new Error('Failed to fetch storage settings');
      }

      const data = await response.json();
      setSettings(data);
      setStorageType(data.storage_type || 'local');
      setS3Bucket(data.s3_bucket || '');
      setS3Region(data.s3_region || '');
      setS3Endpoint(data.s3_endpoint || '');
    } catch (err) {
      console.error('Failed to fetch settings:', err);
      setError('Failed to load storage settings');
    } finally {
      setLoading(false);
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    setSuccess(false);

    // Validation
    if (storageType === 's3') {
      if (!s3Bucket || !s3Region) {
        setError('S3 bucket and region are required for S3 storage');
        return;
      }
      if (!s3AccessKeyId || !s3SecretKey) {
        setError('S3 credentials are required for S3 storage');
        return;
      }
    }

    try {
      setSaving(true);

      const payload: any = {
        storage_type: storageType,
      };

      if (storageType === 's3') {
        payload.s3_bucket = s3Bucket;
        payload.s3_region = s3Region;
        payload.s3_access_key_id = s3AccessKeyId;
        payload.s3_secret_key = s3SecretKey;
        if (s3Endpoint) {
          payload.s3_endpoint = s3Endpoint;
        }
      }

      const response = await fetch(`${process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080'}/api/settings/storage`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${apiClient.getToken()}`,
        },
        body: JSON.stringify(payload),
      });

      if (!response.ok) {
        const errorData = await response.json();
        throw new Error(errorData.message || 'Failed to update storage settings');
      }

      setSuccess(true);
      // Clear sensitive fields after successful save
      setS3AccessKeyId('');
      setS3SecretKey('');

      // Refresh settings
      await fetchSettings();
    } catch (err: any) {
      console.error('Failed to save settings:', err);
      setError(err.message || 'Failed to save storage settings');
    } finally {
      setSaving(false);
    }
  };

  if (loading) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-black">
        <p className="text-gray-400">Loading storage settings...</p>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-black p-8">
      <div className="mx-auto max-w-3xl">
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-white">Storage Settings</h1>
          <p className="mt-2 text-gray-400">
            Configure where OpenChat stores file attachments
          </p>
        </div>

        {error && (
          <div className="mb-6 rounded-lg border border-red-800 bg-red-900 bg-opacity-20 p-4">
            <p className="text-red-400">{error}</p>
          </div>
        )}

        {success && (
          <div className="mb-6 rounded-lg border border-green-800 bg-green-900 bg-opacity-20 p-4">
            <p className="text-green-400">Storage settings updated successfully!</p>
          </div>
        )}

        <form onSubmit={handleSubmit} className="space-y-6">
          {/* Storage Type Selector */}
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-2">
              Storage Type
            </label>
            <div className="space-y-3">
              <label className="flex items-center gap-3 rounded-lg border border-gray-700 bg-gray-900 p-4 cursor-pointer hover:bg-gray-800 transition-colors">
                <input
                  type="radio"
                  name="storageType"
                  value="local"
                  checked={storageType === 'local'}
                  onChange={(e) => setStorageType(e.target.value)}
                  className="h-4 w-4 text-blue-600"
                />
                <div>
                  <div className="font-medium text-white">Local File System</div>
                  <div className="text-sm text-gray-400">Store files on the server's local disk</div>
                </div>
              </label>

              <label className="flex items-center gap-3 rounded-lg border border-gray-700 bg-gray-900 p-4 cursor-pointer hover:bg-gray-800 transition-colors">
                <input
                  type="radio"
                  name="storageType"
                  value="s3"
                  checked={storageType === 's3'}
                  onChange={(e) => setStorageType(e.target.value)}
                  className="h-4 w-4 text-blue-600"
                />
                <div>
                  <div className="font-medium text-white">Amazon S3</div>
                  <div className="text-sm text-gray-400">Store files in AWS S3 bucket</div>
                </div>
              </label>
            </div>
          </div>

          {/* S3 Configuration */}
          {storageType === 's3' && (
            <div className="space-y-4 rounded-lg border border-gray-700 bg-gray-900 p-6">
              <h3 className="text-lg font-semibold text-white">S3 Configuration</h3>

              <div>
                <label htmlFor="s3Bucket" className="block text-sm font-medium text-gray-300 mb-2">
                  S3 Bucket Name *
                </label>
                <input
                  type="text"
                  id="s3Bucket"
                  value={s3Bucket}
                  onChange={(e) => setS3Bucket(e.target.value)}
                  className="w-full rounded-lg border border-gray-600 bg-gray-800 px-4 py-2 text-white focus:border-blue-500 focus:outline-none"
                  placeholder="my-openchat-bucket"
                  required={storageType === 's3'}
                />
              </div>

              <div>
                <label htmlFor="s3Region" className="block text-sm font-medium text-gray-300 mb-2">
                  AWS Region *
                </label>
                <input
                  type="text"
                  id="s3Region"
                  value={s3Region}
                  onChange={(e) => setS3Region(e.target.value)}
                  className="w-full rounded-lg border border-gray-600 bg-gray-800 px-4 py-2 text-white focus:border-blue-500 focus:outline-none"
                  placeholder="us-east-1"
                  required={storageType === 's3'}
                />
              </div>

              <div>
                <label htmlFor="s3AccessKeyId" className="block text-sm font-medium text-gray-300 mb-2">
                  Access Key ID *
                </label>
                <input
                  type="text"
                  id="s3AccessKeyId"
                  value={s3AccessKeyId}
                  onChange={(e) => setS3AccessKeyId(e.target.value)}
                  className="w-full rounded-lg border border-gray-600 bg-gray-800 px-4 py-2 text-white focus:border-blue-500 focus:outline-none"
                  placeholder="AKIAIOSFODNN7EXAMPLE"
                  required={storageType === 's3'}
                  autoComplete="off"
                />
              </div>

              <div>
                <label htmlFor="s3SecretKey" className="block text-sm font-medium text-gray-300 mb-2">
                  Secret Access Key *
                </label>
                <input
                  type="password"
                  id="s3SecretKey"
                  value={s3SecretKey}
                  onChange={(e) => setS3SecretKey(e.target.value)}
                  className="w-full rounded-lg border border-gray-600 bg-gray-800 px-4 py-2 text-white focus:border-blue-500 focus:outline-none"
                  placeholder="wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"
                  required={storageType === 's3'}
                  autoComplete="off"
                />
              </div>

              <div>
                <label htmlFor="s3Endpoint" className="block text-sm font-medium text-gray-300 mb-2">
                  Custom Endpoint (Optional)
                </label>
                <input
                  type="text"
                  id="s3Endpoint"
                  value={s3Endpoint}
                  onChange={(e) => setS3Endpoint(e.target.value)}
                  className="w-full rounded-lg border border-gray-600 bg-gray-800 px-4 py-2 text-white focus:border-blue-500 focus:outline-none"
                  placeholder="https://s3.example.com"
                />
                <p className="mt-1 text-xs text-gray-500">
                  For S3-compatible services like MinIO or DigitalOcean Spaces
                </p>
              </div>

              <div className="rounded-lg border border-yellow-800 bg-yellow-900 bg-opacity-20 p-4">
                <p className="text-sm text-yellow-400">
                  <strong>Security Note:</strong> Credentials are encrypted before being stored in the database.
                  For production use, consider using IAM roles instead of access keys.
                </p>
              </div>
            </div>
          )}

          <div className="flex gap-4">
            <button
              type="submit"
              disabled={saving}
              className="rounded-lg bg-blue-600 px-6 py-3 font-medium text-white transition-colors hover:bg-blue-700 disabled:bg-gray-700 disabled:cursor-not-allowed"
            >
              {saving ? 'Saving...' : 'Save Settings'}
            </button>

            <button
              type="button"
              onClick={fetchSettings}
              className="rounded-lg border border-gray-600 bg-gray-800 px-6 py-3 font-medium text-white transition-colors hover:bg-gray-700"
            >
              Cancel
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
