'use client';

import { useState, useEffect, useCallback } from 'react';
import { apiClient } from '@/lib/api';

type ConnectionType = 'api' | 'database';

interface MattermostConnection {
  type: ConnectionType;
  server_url?: string;
  access_token?: string;
  connection_string?: string;
}

interface UserMapping {
  mattermost_id: string;
  email: string;
  username: string;
  display_name: string | null;
  openchat_user_id: string | null;
  action: 'match' | 'create' | 'skip';
}

interface ChannelInfo {
  mattermost_id: string;
  name: string;
  display_name: string;
  channel_type: string;
  member_count: number;
  message_count: number;
  selected: boolean;
}

interface MigrationPreview {
  users: {
    total: number;
    will_create: number;
    will_match: number;
    users: UserMapping[];
  };
  channels: {
    public_count: number;
    private_count: number;
    channels: ChannelInfo[];
  };
  direct_messages: {
    direct_count: number;
    group_count: number;
  };
  messages: {
    total: number;
    with_attachments: number;
    with_reactions: number;
  };
  attachments: {
    total: number;
    total_size_bytes: number;
  };
  has_message_limit: boolean;
  message_limit_warning: string | null;
}

interface MigrationJob {
  id: string;
  org_id: string;
  status: 'pending' | 'running' | 'completed' | 'failed' | 'cancelled';
  progress: {
    phase: string;
    users_processed: number;
    users_total: number;
    channels_processed: number;
    channels_total: number;
    dms_processed: number;
    dms_total: number;
    messages_processed: number;
    messages_total: number;
    attachments_processed: number;
    attachments_total: number;
    current_item: string | null;
    errors: string[];
  };
  error: string | null;
  started_at: string;
  completed_at: string | null;
}

type Step = 'connection' | 'preview' | 'options' | 'running' | 'complete';

export default function ImportPage() {
  const [step, setStep] = useState<Step>('connection');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Connection settings
  const [connectionType, setConnectionType] = useState<ConnectionType>('api');
  const [serverUrl, setServerUrl] = useState('');
  const [accessToken, setAccessToken] = useState('');
  const [connectionString, setConnectionString] = useState('');
  const [connectionValid, setConnectionValid] = useState(false);

  // Preview data
  const [preview, setPreview] = useState<MigrationPreview | null>(null);
  const [selectedChannels, setSelectedChannels] = useState<string[]>([]);
  const [includeDms, setIncludeDms] = useState(true);
  const [includeGroupDms, setIncludeGroupDms] = useState(true);
  const [includeAttachments, setIncludeAttachments] = useState(true);

  // Migration job
  const [currentJob, setCurrentJob] = useState<MigrationJob | null>(null);
  const [jobs, setJobs] = useState<MigrationJob[]>([]);

  // Fetch existing jobs on mount
  useEffect(() => {
    fetchJobs();
  }, []);

  // Poll for job status when running
  useEffect(() => {
    if (currentJob && (currentJob.status === 'pending' || currentJob.status === 'running')) {
      const interval = setInterval(() => {
        fetchJobStatus(currentJob.id);
      }, 2000);
      return () => clearInterval(interval);
    }
  }, [currentJob?.id, currentJob?.status]);

  const fetchJobs = async () => {
    try {
      const response = await fetch(
        `${process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080'}/api/settings/import/mattermost/jobs`,
        {
          headers: {
            'Authorization': `Bearer ${apiClient.getToken()}`,
          },
        }
      );
      if (response.ok) {
        const data = await response.json();
        setJobs(data);
        // Check for running job
        const runningJob = data.find((j: MigrationJob) => j.status === 'running' || j.status === 'pending');
        if (runningJob) {
          setCurrentJob(runningJob);
          setStep('running');
        }
      }
    } catch (err) {
      console.error('Failed to fetch jobs:', err);
    }
  };

  const fetchJobStatus = async (jobId: string) => {
    try {
      const response = await fetch(
        `${process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080'}/api/settings/import/mattermost/jobs/${jobId}`,
        {
          headers: {
            'Authorization': `Bearer ${apiClient.getToken()}`,
          },
        }
      );
      if (response.ok) {
        const data = await response.json();
        setCurrentJob(data);
        if (data.status === 'completed' || data.status === 'failed' || data.status === 'cancelled') {
          setStep('complete');
        }
      }
    } catch (err) {
      console.error('Failed to fetch job status:', err);
    }
  };

  const buildConnection = (): MattermostConnection => {
    if (connectionType === 'api') {
      return {
        type: 'api',
        server_url: serverUrl,
        access_token: accessToken,
      };
    } else {
      return {
        type: 'database',
        connection_string: connectionString,
      };
    }
  };

  const validateConnection = async () => {
    setError(null);
    setLoading(true);

    try {
      const response = await fetch(
        `${process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080'}/api/settings/import/mattermost/validate`,
        {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${apiClient.getToken()}`,
          },
          body: JSON.stringify({ connection: buildConnection() }),
        }
      );

      const data = await response.json();

      if (!response.ok) {
        throw new Error(data.message || 'Validation failed');
      }

      if (data.valid) {
        setConnectionValid(true);
      } else {
        setError(data.message || 'Connection invalid');
      }
    } catch (err: any) {
      setError(err.message || 'Failed to validate connection');
    } finally {
      setLoading(false);
    }
  };

  const fetchPreview = async () => {
    setError(null);
    setLoading(true);

    try {
      const response = await fetch(
        `${process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080'}/api/settings/import/mattermost/preview`,
        {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${apiClient.getToken()}`,
          },
          body: JSON.stringify({ connection: buildConnection() }),
        }
      );

      const data = await response.json();

      if (!response.ok) {
        throw new Error(data.message || 'Failed to get preview');
      }

      setPreview(data);
      setSelectedChannels(data.channels.channels.map((c: ChannelInfo) => c.mattermost_id));
      setStep('preview');
    } catch (err: any) {
      setError(err.message || 'Failed to fetch preview');
    } finally {
      setLoading(false);
    }
  };

  const startMigration = async () => {
    setError(null);
    setLoading(true);

    try {
      const response = await fetch(
        `${process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080'}/api/settings/import/mattermost/start`,
        {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${apiClient.getToken()}`,
          },
          body: JSON.stringify({
            connection: buildConnection(),
            options: {
              include_channels: selectedChannels,
              include_dms: includeDms,
              include_group_dms: includeGroupDms,
              include_attachments: includeAttachments,
              user_mappings: [],
            },
          }),
        }
      );

      const data = await response.json();

      if (!response.ok) {
        throw new Error(data.message || 'Failed to start migration');
      }

      // Fetch the job status
      fetchJobStatus(data.job_id);
      setStep('running');
    } catch (err: any) {
      setError(err.message || 'Failed to start migration');
    } finally {
      setLoading(false);
    }
  };

  const cancelMigration = async () => {
    if (!currentJob) return;

    try {
      await fetch(
        `${process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080'}/api/settings/import/mattermost/jobs/${currentJob.id}/cancel`,
        {
          method: 'POST',
          headers: {
            'Authorization': `Bearer ${apiClient.getToken()}`,
          },
        }
      );
      fetchJobStatus(currentJob.id);
    } catch (err) {
      console.error('Failed to cancel job:', err);
    }
  };

  const toggleChannel = (channelId: string) => {
    setSelectedChannels(prev =>
      prev.includes(channelId)
        ? prev.filter(id => id !== channelId)
        : [...prev, channelId]
    );
  };

  const formatBytes = (bytes: number) => {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  const getProgressPercent = () => {
    if (!currentJob) return 0;
    const p = currentJob.progress;
    const total = p.users_total + p.channels_total + p.dms_total + p.messages_total;
    const done = p.users_processed + p.channels_processed + p.dms_processed + p.messages_processed;
    return total > 0 ? Math.round((done / total) * 100) : 0;
  };

  return (
    <div className="min-h-screen bg-black p-8">
      <div className="mx-auto max-w-4xl">
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-white">Import from Mattermost</h1>
          <p className="mt-2 text-gray-400">
            Migrate your channels, messages, and files from Mattermost
          </p>
        </div>

        {error && (
          <div className="mb-6 rounded-lg border border-red-800 bg-red-900 bg-opacity-20 p-4">
            <p className="text-red-400">{error}</p>
          </div>
        )}

        {/* Step 1: Connection */}
        {step === 'connection' && (
          <div className="space-y-6">
            <div className="rounded-lg border border-gray-700 bg-gray-900 p-6">
              <h2 className="text-xl font-semibold text-white mb-4">Connection Method</h2>

              <div className="space-y-3 mb-6">
                <label className="flex items-center gap-3 rounded-lg border border-gray-700 bg-gray-800 p-4 cursor-pointer hover:bg-gray-750 transition-colors">
                  <input
                    type="radio"
                    name="connectionType"
                    value="api"
                    checked={connectionType === 'api'}
                    onChange={() => setConnectionType('api')}
                    className="h-4 w-4 text-blue-600"
                  />
                  <div>
                    <div className="font-medium text-white">API Connection (Recommended)</div>
                    <div className="text-sm text-gray-400">Connect using Mattermost API with an admin token</div>
                  </div>
                </label>

                <label className="flex items-center gap-3 rounded-lg border border-gray-700 bg-gray-800 p-4 cursor-pointer hover:bg-gray-750 transition-colors">
                  <input
                    type="radio"
                    name="connectionType"
                    value="database"
                    checked={connectionType === 'database'}
                    onChange={() => setConnectionType('database')}
                    className="h-4 w-4 text-blue-600"
                  />
                  <div>
                    <div className="font-medium text-white">Database Connection</div>
                    <div className="text-sm text-gray-400">Direct database access for full message history</div>
                  </div>
                </label>
              </div>

              {connectionType === 'api' ? (
                <div className="space-y-4">
                  <div>
                    <label className="block text-sm font-medium text-gray-300 mb-2">
                      Mattermost Server URL
                    </label>
                    <input
                      type="url"
                      value={serverUrl}
                      onChange={(e) => { setServerUrl(e.target.value); setConnectionValid(false); }}
                      placeholder="https://mattermost.example.com"
                      className="w-full rounded-lg border border-gray-600 bg-gray-800 px-4 py-2 text-white focus:border-blue-500 focus:outline-none"
                    />
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-gray-300 mb-2">
                      Admin Access Token
                    </label>
                    <input
                      type="password"
                      value={accessToken}
                      onChange={(e) => { setAccessToken(e.target.value); setConnectionValid(false); }}
                      placeholder="xxxxxxxxxxxxxxxxxxxxxxxxxxxx"
                      className="w-full rounded-lg border border-gray-600 bg-gray-800 px-4 py-2 text-white focus:border-blue-500 focus:outline-none"
                    />
                    <p className="mt-1 text-xs text-gray-500">
                      Generate a personal access token in Mattermost: Profile → Security → Personal Access Tokens
                    </p>
                  </div>
                </div>
              ) : (
                <div className="space-y-4">
                  <div>
                    <label className="block text-sm font-medium text-gray-300 mb-2">
                      PostgreSQL Connection String
                    </label>
                    <input
                      type="password"
                      value={connectionString}
                      onChange={(e) => { setConnectionString(e.target.value); setConnectionValid(false); }}
                      placeholder="postgres://user:password@host:5432/mattermost"
                      className="w-full rounded-lg border border-gray-600 bg-gray-800 px-4 py-2 text-white focus:border-blue-500 focus:outline-none"
                    />
                  </div>
                  <div className="rounded-lg border border-yellow-800 bg-yellow-900 bg-opacity-20 p-4">
                    <p className="text-sm text-yellow-400">
                      <strong>Note:</strong> Database connection bypasses Mattermost free tier message limits but requires direct database access.
                    </p>
                  </div>
                </div>
              )}
            </div>

            <div className="flex gap-4">
              {!connectionValid ? (
                <button
                  onClick={validateConnection}
                  disabled={loading || (connectionType === 'api' ? !serverUrl || !accessToken : !connectionString)}
                  className="rounded-lg bg-gray-700 px-6 py-3 font-medium text-white transition-colors hover:bg-gray-600 disabled:bg-gray-800 disabled:cursor-not-allowed"
                >
                  {loading ? 'Validating...' : 'Validate Connection'}
                </button>
              ) : (
                <button
                  onClick={fetchPreview}
                  disabled={loading}
                  className="rounded-lg bg-blue-600 px-6 py-3 font-medium text-white transition-colors hover:bg-blue-700 disabled:bg-gray-700"
                >
                  {loading ? 'Loading...' : 'Continue to Preview'}
                </button>
              )}
            </div>

            {connectionValid && (
              <div className="rounded-lg border border-green-800 bg-green-900 bg-opacity-20 p-4">
                <p className="text-green-400">Connection validated successfully!</p>
              </div>
            )}
          </div>
        )}

        {/* Step 2: Preview */}
        {step === 'preview' && preview && (
          <div className="space-y-6">
            {preview.message_limit_warning && (
              <div className="rounded-lg border border-yellow-800 bg-yellow-900 bg-opacity-20 p-4">
                <p className="text-yellow-400">{preview.message_limit_warning}</p>
              </div>
            )}

            {/* Summary Stats */}
            <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
              <div className="rounded-lg border border-gray-700 bg-gray-900 p-4">
                <div className="text-2xl font-bold text-white">{preview.users.total}</div>
                <div className="text-sm text-gray-400">Users</div>
                <div className="text-xs text-gray-500 mt-1">
                  {preview.users.will_match} match, {preview.users.will_create} new
                </div>
              </div>
              <div className="rounded-lg border border-gray-700 bg-gray-900 p-4">
                <div className="text-2xl font-bold text-white">
                  {preview.channels.public_count + preview.channels.private_count}
                </div>
                <div className="text-sm text-gray-400">Channels</div>
                <div className="text-xs text-gray-500 mt-1">
                  {preview.channels.public_count} public, {preview.channels.private_count} private
                </div>
              </div>
              <div className="rounded-lg border border-gray-700 bg-gray-900 p-4">
                <div className="text-2xl font-bold text-white">{preview.messages.total.toLocaleString()}</div>
                <div className="text-sm text-gray-400">Messages</div>
              </div>
              <div className="rounded-lg border border-gray-700 bg-gray-900 p-4">
                <div className="text-2xl font-bold text-white">{preview.attachments.total}</div>
                <div className="text-sm text-gray-400">Attachments</div>
                <div className="text-xs text-gray-500 mt-1">
                  {formatBytes(preview.attachments.total_size_bytes)}
                </div>
              </div>
            </div>

            {/* Channel Selection */}
            <div className="rounded-lg border border-gray-700 bg-gray-900 p-6">
              <h3 className="text-lg font-semibold text-white mb-4">Select Channels to Import</h3>
              <div className="space-y-2 max-h-64 overflow-y-auto">
                {preview.channels.channels.map((channel) => (
                  <label
                    key={channel.mattermost_id}
                    className="flex items-center gap-3 rounded-lg border border-gray-700 bg-gray-800 p-3 cursor-pointer hover:bg-gray-750"
                  >
                    <input
                      type="checkbox"
                      checked={selectedChannels.includes(channel.mattermost_id)}
                      onChange={() => toggleChannel(channel.mattermost_id)}
                      className="h-4 w-4 text-blue-600 rounded"
                    />
                    <div className="flex-1">
                      <div className="text-white font-medium">{channel.display_name}</div>
                      <div className="text-xs text-gray-500">
                        {channel.channel_type} · {channel.member_count} members · {channel.message_count.toLocaleString()} messages
                      </div>
                    </div>
                  </label>
                ))}
              </div>
            </div>

            {/* DM and Attachment Options */}
            <div className="rounded-lg border border-gray-700 bg-gray-900 p-6">
              <h3 className="text-lg font-semibold text-white mb-4">Import Options</h3>
              <div className="space-y-3">
                <label className="flex items-center gap-3">
                  <input
                    type="checkbox"
                    checked={includeDms}
                    onChange={(e) => setIncludeDms(e.target.checked)}
                    className="h-4 w-4 text-blue-600 rounded"
                  />
                  <div>
                    <div className="text-white">Import Direct Messages</div>
                    <div className="text-xs text-gray-500">{preview.direct_messages.direct_count} conversations</div>
                  </div>
                </label>
                <label className="flex items-center gap-3">
                  <input
                    type="checkbox"
                    checked={includeGroupDms}
                    onChange={(e) => setIncludeGroupDms(e.target.checked)}
                    className="h-4 w-4 text-blue-600 rounded"
                  />
                  <div>
                    <div className="text-white">Import Group DMs</div>
                    <div className="text-xs text-gray-500">{preview.direct_messages.group_count} group conversations</div>
                  </div>
                </label>
                <label className="flex items-center gap-3">
                  <input
                    type="checkbox"
                    checked={includeAttachments}
                    onChange={(e) => setIncludeAttachments(e.target.checked)}
                    className="h-4 w-4 text-blue-600 rounded"
                  />
                  <div>
                    <div className="text-white">Import Attachments</div>
                    <div className="text-xs text-gray-500">{preview.attachments.total} files ({formatBytes(preview.attachments.total_size_bytes)})</div>
                  </div>
                </label>
              </div>
            </div>

            {/* User Mapping Preview */}
            <div className="rounded-lg border border-gray-700 bg-gray-900 p-6">
              <h3 className="text-lg font-semibold text-white mb-4">User Mapping</h3>
              <div className="space-y-2 max-h-48 overflow-y-auto">
                {preview.users.users.slice(0, 10).map((user) => (
                  <div key={user.mattermost_id} className="flex items-center justify-between py-2 border-b border-gray-700">
                    <div>
                      <div className="text-white">{user.display_name || user.username}</div>
                      <div className="text-xs text-gray-500">{user.email}</div>
                    </div>
                    <span className={`px-2 py-1 rounded text-xs ${
                      user.action === 'match' ? 'bg-green-900 text-green-400' :
                      user.action === 'create' ? 'bg-blue-900 text-blue-400' :
                      'bg-gray-700 text-gray-400'
                    }`}>
                      {user.action === 'match' ? 'Existing User' : user.action === 'create' ? 'Will Create' : 'Skip'}
                    </span>
                  </div>
                ))}
                {preview.users.users.length > 10 && (
                  <p className="text-gray-500 text-sm">... and {preview.users.users.length - 10} more users</p>
                )}
              </div>
            </div>

            <div className="flex gap-4">
              <button
                onClick={() => setStep('connection')}
                className="rounded-lg border border-gray-600 bg-gray-800 px-6 py-3 font-medium text-white transition-colors hover:bg-gray-700"
              >
                Back
              </button>
              <button
                onClick={startMigration}
                disabled={loading || selectedChannels.length === 0}
                className="rounded-lg bg-blue-600 px-6 py-3 font-medium text-white transition-colors hover:bg-blue-700 disabled:bg-gray-700 disabled:cursor-not-allowed"
              >
                {loading ? 'Starting...' : 'Start Import'}
              </button>
            </div>
          </div>
        )}

        {/* Step 3: Running */}
        {step === 'running' && currentJob && (
          <div className="space-y-6">
            <div className="rounded-lg border border-gray-700 bg-gray-900 p-6">
              <div className="flex items-center justify-between mb-4">
                <h2 className="text-xl font-semibold text-white">Import in Progress</h2>
                <span className={`px-3 py-1 rounded-full text-sm ${
                  currentJob.status === 'running' ? 'bg-blue-900 text-blue-400' :
                  currentJob.status === 'pending' ? 'bg-yellow-900 text-yellow-400' :
                  'bg-gray-700 text-gray-400'
                }`}>
                  {currentJob.status}
                </span>
              </div>

              {/* Progress Bar */}
              <div className="mb-6">
                <div className="flex justify-between text-sm text-gray-400 mb-2">
                  <span>Phase: {currentJob.progress.phase}</span>
                  <span>{getProgressPercent()}%</span>
                </div>
                <div className="h-3 bg-gray-700 rounded-full overflow-hidden">
                  <div
                    className="h-full bg-blue-600 transition-all duration-500"
                    style={{ width: `${getProgressPercent()}%` }}
                  />
                </div>
                {currentJob.progress.current_item && (
                  <p className="text-sm text-gray-500 mt-2">{currentJob.progress.current_item}</p>
                )}
              </div>

              {/* Progress Details */}
              <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-6">
                <div className="text-center p-3 bg-gray-800 rounded-lg">
                  <div className="text-lg font-bold text-white">
                    {currentJob.progress.users_processed}/{currentJob.progress.users_total}
                  </div>
                  <div className="text-xs text-gray-500">Users</div>
                </div>
                <div className="text-center p-3 bg-gray-800 rounded-lg">
                  <div className="text-lg font-bold text-white">
                    {currentJob.progress.channels_processed}/{currentJob.progress.channels_total}
                  </div>
                  <div className="text-xs text-gray-500">Channels</div>
                </div>
                <div className="text-center p-3 bg-gray-800 rounded-lg">
                  <div className="text-lg font-bold text-white">
                    {currentJob.progress.dms_processed}/{currentJob.progress.dms_total}
                  </div>
                  <div className="text-xs text-gray-500">DMs</div>
                </div>
                <div className="text-center p-3 bg-gray-800 rounded-lg">
                  <div className="text-lg font-bold text-white">
                    {currentJob.progress.messages_processed.toLocaleString()}/{currentJob.progress.messages_total.toLocaleString()}
                  </div>
                  <div className="text-xs text-gray-500">Messages</div>
                </div>
              </div>

              {/* Errors */}
              {currentJob.progress.errors.length > 0 && (
                <div className="rounded-lg border border-yellow-800 bg-yellow-900 bg-opacity-20 p-4 mb-6">
                  <h4 className="text-yellow-400 font-medium mb-2">Warnings ({currentJob.progress.errors.length})</h4>
                  <div className="max-h-32 overflow-y-auto space-y-1">
                    {currentJob.progress.errors.map((err, i) => (
                      <p key={i} className="text-xs text-yellow-300">{err}</p>
                    ))}
                  </div>
                </div>
              )}

              <button
                onClick={cancelMigration}
                className="rounded-lg border border-red-600 bg-transparent px-6 py-2 font-medium text-red-400 transition-colors hover:bg-red-900 hover:bg-opacity-20"
              >
                Cancel Import
              </button>
            </div>
          </div>
        )}

        {/* Step 4: Complete */}
        {step === 'complete' && currentJob && (
          <div className="space-y-6">
            <div className={`rounded-lg border p-6 ${
              currentJob.status === 'completed'
                ? 'border-green-800 bg-green-900 bg-opacity-20'
                : currentJob.status === 'cancelled'
                ? 'border-yellow-800 bg-yellow-900 bg-opacity-20'
                : 'border-red-800 bg-red-900 bg-opacity-20'
            }`}>
              <h2 className={`text-xl font-semibold mb-4 ${
                currentJob.status === 'completed' ? 'text-green-400' :
                currentJob.status === 'cancelled' ? 'text-yellow-400' :
                'text-red-400'
              }`}>
                {currentJob.status === 'completed' ? 'Import Complete!' :
                 currentJob.status === 'cancelled' ? 'Import Cancelled' :
                 'Import Failed'}
              </h2>

              {currentJob.error && (
                <p className="text-red-300 mb-4">{currentJob.error}</p>
              )}

              <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-6">
                <div className="text-center p-3 bg-gray-800 bg-opacity-50 rounded-lg">
                  <div className="text-lg font-bold text-white">{currentJob.progress.users_processed}</div>
                  <div className="text-xs text-gray-400">Users Imported</div>
                </div>
                <div className="text-center p-3 bg-gray-800 bg-opacity-50 rounded-lg">
                  <div className="text-lg font-bold text-white">{currentJob.progress.channels_processed}</div>
                  <div className="text-xs text-gray-400">Channels Imported</div>
                </div>
                <div className="text-center p-3 bg-gray-800 bg-opacity-50 rounded-lg">
                  <div className="text-lg font-bold text-white">{currentJob.progress.dms_processed}</div>
                  <div className="text-xs text-gray-400">DMs Imported</div>
                </div>
                <div className="text-center p-3 bg-gray-800 bg-opacity-50 rounded-lg">
                  <div className="text-lg font-bold text-white">{currentJob.progress.messages_processed.toLocaleString()}</div>
                  <div className="text-xs text-gray-400">Messages Imported</div>
                </div>
              </div>

              {currentJob.progress.errors.length > 0 && (
                <div className="rounded-lg border border-yellow-800 bg-yellow-900 bg-opacity-20 p-4 mb-6">
                  <h4 className="text-yellow-400 font-medium mb-2">Warnings ({currentJob.progress.errors.length})</h4>
                  <div className="max-h-32 overflow-y-auto space-y-1">
                    {currentJob.progress.errors.map((err, i) => (
                      <p key={i} className="text-xs text-yellow-300">{err}</p>
                    ))}
                  </div>
                </div>
              )}
            </div>

            <button
              onClick={() => {
                setStep('connection');
                setCurrentJob(null);
                setPreview(null);
                setConnectionValid(false);
              }}
              className="rounded-lg bg-gray-700 px-6 py-3 font-medium text-white transition-colors hover:bg-gray-600"
            >
              Start New Import
            </button>
          </div>
        )}

        {/* Previous Jobs */}
        {jobs.length > 0 && step === 'connection' && (
          <div className="mt-8 rounded-lg border border-gray-700 bg-gray-900 p-6">
            <h3 className="text-lg font-semibold text-white mb-4">Previous Imports</h3>
            <div className="space-y-2">
              {jobs.slice(0, 5).map((job) => (
                <div key={job.id} className="flex items-center justify-between py-2 border-b border-gray-700">
                  <div>
                    <div className="text-sm text-white">
                      {new Date(job.started_at).toLocaleDateString()} {new Date(job.started_at).toLocaleTimeString()}
                    </div>
                    <div className="text-xs text-gray-500">
                      {job.progress.messages_processed.toLocaleString()} messages imported
                    </div>
                  </div>
                  <span className={`px-2 py-1 rounded text-xs ${
                    job.status === 'completed' ? 'bg-green-900 text-green-400' :
                    job.status === 'failed' ? 'bg-red-900 text-red-400' :
                    job.status === 'cancelled' ? 'bg-yellow-900 text-yellow-400' :
                    'bg-blue-900 text-blue-400'
                  }`}>
                    {job.status}
                  </span>
                </div>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
