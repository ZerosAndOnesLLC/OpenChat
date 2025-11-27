'use client';

import { useState, useEffect } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { apiClient } from '@/lib/api';
import { IncomingWebhook, Channel } from '@/lib/types';
import { Webhook, Plus, Trash2, AlertCircle, CheckCircle, RefreshCw, Copy, Eye, EyeOff, Edit2, ToggleLeft, ToggleRight } from 'lucide-react';
import { formatDistanceToNow } from 'date-fns';

export default function WebhookManagement() {
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [editingWebhook, setEditingWebhook] = useState<IncomingWebhook | null>(null);
  const [showDeleteConfirm, setShowDeleteConfirm] = useState<string | null>(null);
  const [revealedTokens, setRevealedTokens] = useState<Set<string>>(new Set());
  const [successMessage, setSuccessMessage] = useState<string | null>(null);
  const [copiedId, setCopiedId] = useState<string | null>(null);
  const queryClient = useQueryClient();

  // Fetch webhooks
  const { data: webhooks = [], isLoading, error, refetch } = useQuery({
    queryKey: ['webhooks'],
    queryFn: () => apiClient.listIncomingWebhooks(),
  });

  // Fetch channels for the dropdown
  const { data: channels = [] } = useQuery({
    queryKey: ['channels'],
    queryFn: () => apiClient.listChannels(),
  });

  // Create webhook mutation
  const createMutation = useMutation({
    mutationFn: (data: { channel_id: string; display_name: string; description?: string; username?: string }) =>
      apiClient.createIncomingWebhook(data),
    onSuccess: (newWebhook) => {
      queryClient.invalidateQueries({ queryKey: ['webhooks'] });
      setShowCreateModal(false);
      setSuccessMessage('Webhook created successfully');
      // Auto-reveal the new webhook URL
      setRevealedTokens(prev => new Set([...prev, newWebhook.id]));
      setTimeout(() => setSuccessMessage(null), 5000);
    },
  });

  // Update webhook mutation
  const updateMutation = useMutation({
    mutationFn: ({ id, data }: { id: string; data: { display_name?: string; description?: string; username?: string; channel_id?: string; enabled?: boolean } }) =>
      apiClient.updateIncomingWebhook(id, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['webhooks'] });
      setEditingWebhook(null);
      setSuccessMessage('Webhook updated successfully');
      setTimeout(() => setSuccessMessage(null), 3000);
    },
  });

  // Delete webhook mutation
  const deleteMutation = useMutation({
    mutationFn: (id: string) => apiClient.deleteIncomingWebhook(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['webhooks'] });
      setShowDeleteConfirm(null);
      setSuccessMessage('Webhook deleted successfully');
      setTimeout(() => setSuccessMessage(null), 3000);
    },
  });

  // Regenerate token mutation
  const regenerateMutation = useMutation({
    mutationFn: (id: string) => apiClient.regenerateWebhookToken(id),
    onSuccess: (updatedWebhook) => {
      queryClient.invalidateQueries({ queryKey: ['webhooks'] });
      setRevealedTokens(prev => new Set([...prev, updatedWebhook.id]));
      setSuccessMessage('Webhook token regenerated');
      setTimeout(() => setSuccessMessage(null), 3000);
    },
  });

  const copyToClipboard = async (text: string, webhookId: string) => {
    await navigator.clipboard.writeText(text);
    setCopiedId(webhookId);
    setTimeout(() => setCopiedId(null), 2000);
  };

  const toggleTokenVisibility = (webhookId: string) => {
    setRevealedTokens(prev => {
      const newSet = new Set(prev);
      if (newSet.has(webhookId)) {
        newSet.delete(webhookId);
      } else {
        newSet.add(webhookId);
      }
      return newSet;
    });
  };

  const getChannelName = (channelId: string) => {
    const channel = channels.find(c => c.id === channelId);
    return channel?.name || 'Unknown Channel';
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-12">
        <RefreshCw className="w-8 h-8 text-blue-600 dark:text-blue-400 animate-spin" />
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white">Incoming Webhooks</h3>
          <p className="text-sm text-gray-600 dark:text-gray-400 mt-1">
            Allow external services to post messages to your channels
          </p>
        </div>
        <button
          onClick={() => setShowCreateModal(true)}
          className="inline-flex items-center gap-2 px-4 py-2 text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 rounded-lg transition-colors"
        >
          <Plus className="w-4 h-4" />
          Create Webhook
        </button>
      </div>

      {/* Success Message */}
      {successMessage && (
        <div className="p-4 bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-lg flex items-start gap-3">
          <CheckCircle className="w-5 h-5 text-green-600 dark:text-green-400 flex-shrink-0 mt-0.5" />
          <p className="text-sm text-green-800 dark:text-green-200">{successMessage}</p>
        </div>
      )}

      {/* Error Message */}
      {error && (
        <div className="p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg flex items-start gap-3">
          <AlertCircle className="w-5 h-5 text-red-600 dark:text-red-400 flex-shrink-0 mt-0.5" />
          <p className="text-sm text-red-800 dark:text-red-200">{error instanceof Error ? error.message : 'Failed to load webhooks'}</p>
        </div>
      )}

      {/* Webhook List */}
      {webhooks.length === 0 ? (
        <div className="text-center py-12 bg-gray-50 dark:bg-gray-900/50 rounded-xl border border-gray-200 dark:border-gray-700">
          <Webhook className="w-12 h-12 text-gray-400 dark:text-gray-600 mx-auto mb-4" />
          <p className="text-gray-600 dark:text-gray-400 font-medium">No webhooks configured</p>
          <p className="text-sm text-gray-500 dark:text-gray-500 mt-1">
            Create a webhook to allow external services to post messages
          </p>
        </div>
      ) : (
        <div className="space-y-4">
          {webhooks.map((webhook) => (
            <div
              key={webhook.id}
              className={`bg-white dark:bg-gray-800 border rounded-xl p-5 ${
                webhook.enabled ? 'border-gray-200 dark:border-gray-700' : 'border-orange-200 dark:border-orange-800 bg-orange-50/50 dark:bg-orange-900/10'
              }`}
            >
              <div className="flex items-start justify-between mb-4">
                <div className="flex items-start gap-4 flex-1">
                  <div className={`p-3 rounded-xl ${webhook.enabled ? 'bg-blue-100 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400' : 'bg-gray-100 dark:bg-gray-700 text-gray-400'}`}>
                    <Webhook className="w-5 h-5" />
                  </div>
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-3 mb-1">
                      <h4 className="font-semibold text-gray-900 dark:text-white">
                        {webhook.display_name}
                      </h4>
                      {!webhook.enabled && (
                        <span className="px-2 py-0.5 text-xs font-medium bg-orange-100 dark:bg-orange-900/30 text-orange-700 dark:text-orange-400 rounded">
                          Disabled
                        </span>
                      )}
                    </div>
                    <p className="text-sm text-gray-600 dark:text-gray-400 mb-2">
                      Posts to <span className="font-medium">#{getChannelName(webhook.channel_id)}</span>
                    </p>
                    {webhook.description && (
                      <p className="text-sm text-gray-500 dark:text-gray-500">{webhook.description}</p>
                    )}
                    <p className="text-xs text-gray-400 dark:text-gray-600 mt-2">
                      Created {formatDistanceToNow(new Date(webhook.created_at), { addSuffix: true })}
                    </p>
                  </div>
                </div>
                <div className="flex items-center gap-2">
                  <button
                    onClick={() => updateMutation.mutate({ id: webhook.id, data: { enabled: !webhook.enabled } })}
                    className="p-2 text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
                    title={webhook.enabled ? 'Disable webhook' : 'Enable webhook'}
                  >
                    {webhook.enabled ? <ToggleRight className="w-5 h-5 text-green-500" /> : <ToggleLeft className="w-5 h-5" />}
                  </button>
                  <button
                    onClick={() => setEditingWebhook(webhook)}
                    className="p-2 text-gray-500 hover:text-blue-600 dark:text-gray-400 dark:hover:text-blue-400"
                    title="Edit webhook"
                  >
                    <Edit2 className="w-4 h-4" />
                  </button>
                  <button
                    onClick={() => setShowDeleteConfirm(webhook.id)}
                    className="p-2 text-gray-500 hover:text-red-600 dark:text-gray-400 dark:hover:text-red-400"
                    title="Delete webhook"
                  >
                    <Trash2 className="w-4 h-4" />
                  </button>
                </div>
              </div>

              {/* Webhook URL */}
              {webhook.url && (
                <div className="bg-gray-50 dark:bg-gray-900/50 rounded-lg p-3 mt-3">
                  <div className="flex items-center justify-between mb-1">
                    <span className="text-xs font-medium text-gray-500 dark:text-gray-400">Webhook URL</span>
                    <div className="flex items-center gap-1">
                      <button
                        onClick={() => toggleTokenVisibility(webhook.id)}
                        className="p-1 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
                        title={revealedTokens.has(webhook.id) ? 'Hide URL' : 'Show URL'}
                      >
                        {revealedTokens.has(webhook.id) ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
                      </button>
                      <button
                        onClick={() => copyToClipboard(webhook.url!, webhook.id)}
                        className="p-1 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
                        title="Copy URL"
                      >
                        {copiedId === webhook.id ? <CheckCircle className="w-4 h-4 text-green-500" /> : <Copy className="w-4 h-4" />}
                      </button>
                    </div>
                  </div>
                  <code className="text-xs font-mono text-gray-700 dark:text-gray-300 break-all">
                    {revealedTokens.has(webhook.id) ? webhook.url : '••••••••••••••••••••••••••••••••'}
                  </code>
                  <button
                    onClick={() => {
                      if (confirm('Regenerate webhook token? The old URL will stop working immediately.')) {
                        regenerateMutation.mutate(webhook.id);
                      }
                    }}
                    className="mt-2 text-xs text-blue-600 dark:text-blue-400 hover:underline"
                  >
                    Regenerate token
                  </button>
                </div>
              )}
            </div>
          ))}
        </div>
      )}

      {/* Create Webhook Modal */}
      {showCreateModal && (
        <WebhookFormModal
          channels={channels}
          onClose={() => setShowCreateModal(false)}
          onSubmit={(data) => createMutation.mutate(data)}
          isLoading={createMutation.isPending}
        />
      )}

      {/* Edit Webhook Modal */}
      {editingWebhook && (
        <WebhookFormModal
          channels={channels}
          webhook={editingWebhook}
          onClose={() => setEditingWebhook(null)}
          onSubmit={(data) => updateMutation.mutate({ id: editingWebhook.id, data })}
          isLoading={updateMutation.isPending}
        />
      )}

      {/* Delete Confirmation */}
      {showDeleteConfirm && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
          <div className="bg-white dark:bg-gray-800 rounded-2xl shadow-2xl max-w-md w-full p-6">
            <div className="flex items-start gap-4 mb-6">
              <div className="p-3 bg-red-100 dark:bg-red-900/30 rounded-full">
                <AlertCircle className="w-6 h-6 text-red-600 dark:text-red-400" />
              </div>
              <div>
                <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-2">
                  Delete Webhook?
                </h3>
                <p className="text-sm text-gray-600 dark:text-gray-400">
                  This webhook will be permanently deleted. Any integrations using this URL will stop working.
                </p>
              </div>
            </div>
            <div className="flex gap-3">
              <button
                onClick={() => setShowDeleteConfirm(null)}
                className="flex-1 px-4 py-2.5 text-sm font-medium text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-600 transition-colors"
              >
                Cancel
              </button>
              <button
                onClick={() => deleteMutation.mutate(showDeleteConfirm)}
                disabled={deleteMutation.isPending}
                className="flex-1 px-4 py-2.5 text-sm font-medium text-white bg-red-600 hover:bg-red-700 rounded-lg transition-colors disabled:opacity-50"
              >
                {deleteMutation.isPending ? 'Deleting...' : 'Delete Webhook'}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Info Box */}
      <div className="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-xl p-4">
        <h4 className="text-sm font-semibold text-blue-900 dark:text-blue-100 mb-2">
          How to use webhooks
        </h4>
        <p className="text-sm text-blue-800 dark:text-blue-200 mb-2">
          Send a POST request to the webhook URL with a JSON body:
        </p>
        <code className="block text-xs bg-blue-100 dark:bg-blue-900/40 p-3 rounded font-mono text-blue-900 dark:text-blue-100">
          {`{"text": "Hello from my integration!"}`}
        </code>
        <p className="text-xs text-blue-700 dark:text-blue-300 mt-2">
          Optional: Add <code className="bg-blue-100 dark:bg-blue-900/40 px-1 rounded">username</code> and <code className="bg-blue-100 dark:bg-blue-900/40 px-1 rounded">icon_url</code> to override defaults.
        </p>
      </div>
    </div>
  );
}

interface WebhookFormModalProps {
  channels: Channel[];
  webhook?: IncomingWebhook;
  onClose: () => void;
  onSubmit: (data: { channel_id: string; display_name: string; description?: string; username?: string }) => void;
  isLoading: boolean;
}

function WebhookFormModal({ channels, webhook, onClose, onSubmit, isLoading }: WebhookFormModalProps) {
  const [formData, setFormData] = useState({
    channel_id: webhook?.channel_id || '',
    display_name: webhook?.display_name || '',
    description: webhook?.description || '',
    username: webhook?.username || '',
  });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onSubmit({
      channel_id: formData.channel_id,
      display_name: formData.display_name,
      description: formData.description || undefined,
      username: formData.username || undefined,
    });
  };

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
      <div className="bg-white dark:bg-gray-800 rounded-2xl shadow-2xl max-w-lg w-full p-6">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-6">
          {webhook ? 'Edit Webhook' : 'Create Incoming Webhook'}
        </h3>

        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
              Channel *
            </label>
            <select
              value={formData.channel_id}
              onChange={(e) => setFormData({ ...formData, channel_id: e.target.value })}
              required
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500"
            >
              <option value="">Select a channel</option>
              {channels.map((channel) => (
                <option key={channel.id} value={channel.id}>
                  #{channel.name}
                </option>
              ))}
            </select>
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
              Display Name *
            </label>
            <input
              type="text"
              value={formData.display_name}
              onChange={(e) => setFormData({ ...formData, display_name: e.target.value })}
              required
              maxLength={100}
              placeholder="e.g., GitHub Notifications"
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500"
            />
            <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
              This name will be shown as the sender of webhook messages
            </p>
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
              Description
            </label>
            <textarea
              value={formData.description}
              onChange={(e) => setFormData({ ...formData, description: e.target.value })}
              rows={2}
              placeholder="What is this webhook for?"
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500"
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
              Username Override
            </label>
            <input
              type="text"
              value={formData.username}
              onChange={(e) => setFormData({ ...formData, username: e.target.value })}
              maxLength={100}
              placeholder="Optional: Override display name for messages"
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500"
            />
          </div>

          <div className="flex gap-3 pt-4">
            <button
              type="button"
              onClick={onClose}
              className="flex-1 px-4 py-2.5 text-sm font-medium text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-600 transition-colors"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={isLoading || !formData.channel_id || !formData.display_name}
              className="flex-1 px-4 py-2.5 text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {isLoading ? 'Saving...' : webhook ? 'Save Changes' : 'Create Webhook'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
