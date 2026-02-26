'use client';

import { useState, useEffect } from 'react';
import { apiClient } from '@/lib/api';
import type { SlashCommand, SlashCommandFull } from '@/lib/types';

export default function SlashCommandsPage() {
  const [builtins, setBuiltins] = useState<SlashCommand[]>([]);
  const [customs, setCustoms] = useState<SlashCommandFull[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Create form
  const [showCreate, setShowCreate] = useState(false);
  const [newName, setNewName] = useState('');
  const [newDescription, setNewDescription] = useState('');
  const [newUsageHint, setNewUsageHint] = useState('');
  const [newWebhookUrl, setNewWebhookUrl] = useState('');
  const [newResponseType, setNewResponseType] = useState('in_channel');
  const [creating, setCreating] = useState(false);

  useEffect(() => {
    fetchCommands();
  }, []);

  const fetchCommands = async () => {
    try {
      setLoading(true);
      setError(null);
      const all = await apiClient.listCommands();
      setBuiltins(all.filter((c) => c.handler_type === 'builtin'));
      // For customs we need to re-fetch since listCommands returns simplified info
      // We'll use the ones with IDs as custom
      const customList = all.filter((c) => c.handler_type === 'webhook' && c.id);
      setCustoms(customList.map((c) => ({
        id: c.id!,
        org_id: '',
        command_name: c.name,
        description: c.description,
        usage_hint: c.usage_hint,
        handler_type: c.handler_type,
        response_type: 'in_channel',
        created_by: '',
        enabled: true,
        created_at: '',
      })));
    } catch (err) {
      setError((err as Error).message);
    } finally {
      setLoading(false);
    }
  };

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newName.trim() || !newWebhookUrl.trim()) return;

    try {
      setCreating(true);
      setError(null);
      await apiClient.createCommand({
        command_name: newName.trim(),
        description: newDescription.trim(),
        usage_hint: newUsageHint.trim() || undefined,
        webhook_url: newWebhookUrl.trim(),
        response_type: newResponseType,
      });
      setNewName('');
      setNewDescription('');
      setNewUsageHint('');
      setNewWebhookUrl('');
      setNewResponseType('in_channel');
      setShowCreate(false);
      fetchCommands();
    } catch (err) {
      setError((err as Error).message);
    } finally {
      setCreating(false);
    }
  };

  const handleDelete = async (id: string) => {
    if (!confirm('Delete this command?')) return;
    try {
      await apiClient.deleteCommand(id);
      fetchCommands();
    } catch (err) {
      setError((err as Error).message);
    }
  };

  const handleToggle = async (cmd: SlashCommandFull) => {
    try {
      await apiClient.updateCommand(cmd.id, { enabled: !cmd.enabled });
      fetchCommands();
    } catch (err) {
      setError((err as Error).message);
    }
  };

  if (loading) {
    return (
      <div className="p-6">
        <div className="animate-pulse text-gray-400">Loading commands...</div>
      </div>
    );
  }

  return (
    <div className="p-6 max-w-4xl">
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-2xl font-bold text-white">Slash Commands</h1>
        <button
          onClick={() => setShowCreate(!showCreate)}
          className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
        >
          {showCreate ? 'Cancel' : 'Create Command'}
        </button>
      </div>

      {error && (
        <div className="mb-4 p-3 bg-red-900/50 border border-red-700 rounded-lg text-red-200 text-sm">
          {error}
        </div>
      )}

      {showCreate && (
        <form onSubmit={handleCreate} className="mb-6 p-4 bg-gray-800 rounded-lg border border-gray-700">
          <h3 className="text-lg font-semibold text-white mb-4">Create Custom Command</h3>
          <div className="space-y-3">
            <div>
              <label className="block text-sm text-gray-400 mb-1">Command Name</label>
              <div className="flex items-center">
                <span className="text-gray-500 mr-1">/</span>
                <input
                  type="text"
                  value={newName}
                  onChange={(e) => setNewName(e.target.value.replace(/[^a-z0-9_-]/gi, '').toLowerCase())}
                  placeholder="mycommand"
                  className="flex-1 px-3 py-2 bg-gray-900 border border-gray-700 rounded text-white text-sm"
                  maxLength={50}
                />
              </div>
            </div>
            <div>
              <label className="block text-sm text-gray-400 mb-1">Description</label>
              <input
                type="text"
                value={newDescription}
                onChange={(e) => setNewDescription(e.target.value)}
                placeholder="What does this command do?"
                className="w-full px-3 py-2 bg-gray-900 border border-gray-700 rounded text-white text-sm"
              />
            </div>
            <div>
              <label className="block text-sm text-gray-400 mb-1">Usage Hint (optional)</label>
              <input
                type="text"
                value={newUsageHint}
                onChange={(e) => setNewUsageHint(e.target.value)}
                placeholder="[text]"
                className="w-full px-3 py-2 bg-gray-900 border border-gray-700 rounded text-white text-sm"
              />
            </div>
            <div>
              <label className="block text-sm text-gray-400 mb-1">Webhook URL</label>
              <input
                type="url"
                value={newWebhookUrl}
                onChange={(e) => setNewWebhookUrl(e.target.value)}
                placeholder="https://example.com/webhook"
                className="w-full px-3 py-2 bg-gray-900 border border-gray-700 rounded text-white text-sm"
              />
            </div>
            <div>
              <label className="block text-sm text-gray-400 mb-1">Response Type</label>
              <select
                value={newResponseType}
                onChange={(e) => setNewResponseType(e.target.value)}
                className="w-full px-3 py-2 bg-gray-900 border border-gray-700 rounded text-white text-sm"
              >
                <option value="in_channel">In Channel (visible to all)</option>
                <option value="ephemeral">Ephemeral (visible to sender only)</option>
              </select>
            </div>
            <button
              type="submit"
              disabled={creating || !newName.trim() || !newWebhookUrl.trim()}
              className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:bg-gray-700 disabled:text-gray-400 transition-colors"
            >
              {creating ? 'Creating...' : 'Create'}
            </button>
          </div>
        </form>
      )}

      {/* Built-in Commands */}
      <div className="mb-6">
        <h2 className="text-lg font-semibold text-gray-300 mb-3">Built-in Commands</h2>
        <div className="space-y-2">
          {builtins.map((cmd) => (
            <div key={cmd.name} className="flex items-center justify-between p-3 bg-gray-800 rounded-lg border border-gray-700">
              <div>
                <span className="text-white font-medium">/{cmd.name}</span>
                {cmd.usage_hint && <span className="text-gray-500 ml-2">{cmd.usage_hint}</span>}
                <p className="text-sm text-gray-400">{cmd.description}</p>
              </div>
              <span className="text-xs text-gray-500 bg-gray-700 px-2 py-1 rounded">builtin</span>
            </div>
          ))}
        </div>
      </div>

      {/* Custom Commands */}
      <div>
        <h2 className="text-lg font-semibold text-gray-300 mb-3">Custom Commands</h2>
        {customs.length === 0 ? (
          <p className="text-gray-500 text-sm">No custom commands configured.</p>
        ) : (
          <div className="space-y-2">
            {customs.map((cmd) => (
              <div key={cmd.id} className="flex items-center justify-between p-3 bg-gray-800 rounded-lg border border-gray-700">
                <div>
                  <span className="text-white font-medium">/{cmd.command_name}</span>
                  {cmd.usage_hint && <span className="text-gray-500 ml-2">{cmd.usage_hint}</span>}
                  <p className="text-sm text-gray-400">{cmd.description}</p>
                </div>
                <div className="flex items-center gap-2">
                  <button
                    onClick={() => handleToggle(cmd)}
                    className={`px-3 py-1 rounded text-xs transition-colors ${
                      cmd.enabled ? 'bg-green-900/50 text-green-400 hover:bg-green-900' : 'bg-gray-700 text-gray-400 hover:bg-gray-600'
                    }`}
                  >
                    {cmd.enabled ? 'Enabled' : 'Disabled'}
                  </button>
                  <button
                    onClick={() => handleDelete(cmd.id)}
                    className="px-3 py-1 bg-red-900/50 text-red-400 rounded text-xs hover:bg-red-900 transition-colors"
                  >
                    Delete
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
