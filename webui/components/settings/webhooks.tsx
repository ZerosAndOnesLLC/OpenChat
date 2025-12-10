'use client';

import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { apiClient } from '@/lib/api';
import { IncomingWebhook, Channel } from '@/lib/types';
import { Webhook, Plus, Trash2, AlertCircle, CheckCircle, RefreshCw, Copy, Eye, EyeOff, Edit2, ToggleLeft, ToggleRight, Lock } from 'lucide-react';
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
  const { data: webhooks = [], isLoading, error } = useQuery({
    queryKey: ['webhooks'],
    queryFn: () => apiClient.listIncomingWebhooks(),
    retry: (failureCount, error) => {
      // Don't retry on permission errors
      const errorMessage = error instanceof Error ? error.message.toLowerCase() : '';
      if (errorMessage.includes('permission') ||
          errorMessage.includes('forbidden') ||
          errorMessage.includes('403') ||
          errorMessage.includes('unauthorized')) {
        return false;
      }
      // Default retry behavior for other errors (max 3 retries)
      return failureCount < 3;
    },
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
        <RefreshCw className="w-6 h-6 text-blue-400 animate-spin" />
      </div>
    );
  }

  // Check if there's a permission error
  const hasPermissionError = error && (() => {
    const errorMessage = error instanceof Error ? error.message.toLowerCase() : '';
    return errorMessage.includes('permission') ||
           errorMessage.includes('forbidden') ||
           errorMessage.includes('403') ||
           errorMessage.includes('unauthorized') ||
           errorMessage.includes('access denied');
  })();

  return (
    <div className="space-y-6">
      {/* Description and Create Button */}
      <div className="flex items-start justify-between gap-4">
        <p className="text-sm text-gray-400">
          Allow external services to post messages to your channels via webhook URLs.
        </p>
        {!hasPermissionError && (
          <button
            onClick={() => setShowCreateModal(true)}
            className="flex-shrink-0 inline-flex items-center gap-2 px-3 py-1.5 text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 rounded-lg transition-colors"
          >
            <Plus className="w-4 h-4" />
            Create
          </button>
        )}
      </div>

      {/* Success Message */}
      {successMessage && (
        <div className="p-4 bg-green-500/10 border border-green-500/20 rounded-lg flex items-start gap-3">
          <CheckCircle className="w-5 h-5 text-green-400 flex-shrink-0 mt-0.5" />
          <p className="text-sm text-green-400">{successMessage}</p>
        </div>
      )}

      {/* Error Message */}
      {error && (
        (() => {
          const errorMessage = error instanceof Error ? error.message : 'Failed to load webhooks';
          const errorLower = errorMessage.toLowerCase();
          const isPermissionError = errorLower.includes('permission') ||
                                    errorLower.includes('forbidden') ||
                                    errorLower.includes('403') ||
                                    errorLower.includes('unauthorized') ||
                                    errorLower.includes('access denied');

          if (isPermissionError) {
            return (
              <div className="text-center py-12 bg-gray-900 rounded-xl border border-gray-800">
                <Lock className="w-12 h-12 text-gray-600 mx-auto mb-4" />
                <p className="text-gray-400 font-medium">Permission Required</p>
                <p className="text-sm text-gray-500 mt-1 max-w-md mx-auto">
                  You don&apos;t have permission to manage webhooks. Contact your administrator to request access.
                </p>
              </div>
            );
          }

          return (
            <div className="p-4 bg-red-500/10 border border-red-500/20 rounded-lg flex items-start gap-3">
              <AlertCircle className="w-5 h-5 text-red-400 flex-shrink-0 mt-0.5" />
              <p className="text-sm text-red-400">{errorMessage}</p>
            </div>
          );
        })()
      )}

      {/* Webhook List */}
      {!hasPermissionError && webhooks.length === 0 && !error ? (
        <div className="text-center py-12 bg-gray-900 rounded-xl border border-gray-800">
          <Webhook className="w-12 h-12 text-gray-600 mx-auto mb-4" />
          <p className="text-gray-400 font-medium">No webhooks configured</p>
          <p className="text-sm text-gray-500 mt-1">
            Create a webhook to allow external services to post messages
          </p>
        </div>
      ) : !hasPermissionError && webhooks.length > 0 ? (
        <div className="space-y-3">
          {webhooks.map((webhook) => (
            <div
              key={webhook.id}
              className={`bg-gray-900 border rounded-xl p-4 ${
                webhook.enabled ? 'border-gray-800' : 'border-orange-500/30 bg-orange-500/5'
              }`}
            >
              <div className="flex items-start justify-between gap-4 mb-3">
                <div className="flex items-start gap-3 flex-1 min-w-0">
                  <div className={`p-2.5 rounded-lg flex-shrink-0 ${webhook.enabled ? 'bg-blue-500/10 text-blue-400' : 'bg-gray-800 text-gray-500'}`}>
                    <Webhook className="w-5 h-5" />
                  </div>
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2 flex-wrap mb-1">
                      <h4 className="font-medium text-white">
                        {webhook.display_name}
                      </h4>
                      {!webhook.enabled && (
                        <span className="px-2 py-0.5 text-xs font-medium bg-orange-500/20 text-orange-400 rounded">
                          Disabled
                        </span>
                      )}
                    </div>
                    <p className="text-sm text-gray-400">
                      Posts to <span className="text-gray-300">#{getChannelName(webhook.channel_id)}</span>
                    </p>
                    {webhook.description && (
                      <p className="text-sm text-gray-500 mt-1">{webhook.description}</p>
                    )}
                    <p className="text-xs text-gray-600 mt-2">
                      Created {formatDistanceToNow(new Date(webhook.created_at), { addSuffix: true })}
                    </p>
                  </div>
                </div>
                <div className="flex items-center gap-1 flex-shrink-0">
                  <button
                    onClick={() => updateMutation.mutate({ id: webhook.id, data: { enabled: !webhook.enabled } })}
                    className="p-2 text-gray-500 hover:text-gray-300 transition-colors"
                    title={webhook.enabled ? 'Disable webhook' : 'Enable webhook'}
                  >
                    {webhook.enabled ? <ToggleRight className="w-5 h-5 text-green-400" /> : <ToggleLeft className="w-5 h-5" />}
                  </button>
                  <button
                    onClick={() => setEditingWebhook(webhook)}
                    className="p-2 text-gray-500 hover:text-blue-400 transition-colors"
                    title="Edit webhook"
                  >
                    <Edit2 className="w-4 h-4" />
                  </button>
                  <button
                    onClick={() => setShowDeleteConfirm(webhook.id)}
                    className="p-2 text-gray-500 hover:text-red-400 transition-colors"
                    title="Delete webhook"
                  >
                    <Trash2 className="w-4 h-4" />
                  </button>
                </div>
              </div>

              {/* Webhook URL */}
              {webhook.url && (
                <div className="bg-gray-950 rounded-lg p-3">
                  <div className="flex items-center justify-between mb-1">
                    <span className="text-xs font-medium text-gray-500">Webhook URL</span>
                    <div className="flex items-center gap-1">
                      <button
                        onClick={() => toggleTokenVisibility(webhook.id)}
                        className="p-1 text-gray-500 hover:text-gray-300 transition-colors"
                        title={revealedTokens.has(webhook.id) ? 'Hide URL' : 'Show URL'}
                      >
                        {revealedTokens.has(webhook.id) ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
                      </button>
                      <button
                        onClick={() => copyToClipboard(webhook.url!, webhook.id)}
                        className="p-1 text-gray-500 hover:text-gray-300 transition-colors"
                        title="Copy URL"
                      >
                        {copiedId === webhook.id ? <CheckCircle className="w-4 h-4 text-green-400" /> : <Copy className="w-4 h-4" />}
                      </button>
                    </div>
                  </div>
                  <code className="text-xs font-mono text-gray-400 break-all block">
                    {revealedTokens.has(webhook.id) ? webhook.url : '••••••••••••••••••••••••••••••••'}
                  </code>
                  <button
                    onClick={() => {
                      if (confirm('Regenerate webhook token? The old URL will stop working immediately.')) {
                        regenerateMutation.mutate(webhook.id);
                      }
                    }}
                    className="mt-2 text-xs text-blue-400 hover:text-blue-300 transition-colors"
                  >
                    Regenerate token
                  </button>
                </div>
              )}
            </div>
          ))}
        </div>
      ) : null}

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
        <div className="fixed inset-0 bg-black/70 flex items-center justify-center z-50 p-4">
          <div className="bg-gray-900 border border-gray-800 rounded-2xl shadow-2xl max-w-md w-full p-6">
            <div className="flex items-start gap-4 mb-6">
              <div className="p-3 bg-red-500/10 rounded-full">
                <AlertCircle className="w-6 h-6 text-red-400" />
              </div>
              <div>
                <h3 className="text-lg font-semibold text-white mb-2">
                  Delete Webhook?
                </h3>
                <p className="text-sm text-gray-400">
                  This webhook will be permanently deleted. Any integrations using this URL will stop working.
                </p>
              </div>
            </div>
            <div className="flex gap-3">
              <button
                onClick={() => setShowDeleteConfirm(null)}
                className="flex-1 px-4 py-2.5 text-sm font-medium text-gray-300 bg-gray-800 border border-gray-700 rounded-lg hover:bg-gray-700 transition-colors"
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
      {!hasPermissionError && (
        <div className="bg-blue-500/10 border border-blue-500/20 rounded-xl p-4">
          <h4 className="text-sm font-semibold text-blue-400 mb-2">
            How to use webhooks
          </h4>
          <p className="text-sm text-blue-300/80 mb-2">
            Send a POST request to the webhook URL with a JSON body:
          </p>
          <code className="block text-xs bg-gray-950 p-3 rounded font-mono text-blue-300">
            {`{"text": "Hello from my integration!"}`}
          </code>
          <p className="text-xs text-blue-300/60 mt-2">
            Optional: Add <code className="bg-gray-950 px-1 rounded">username</code> and <code className="bg-gray-950 px-1 rounded">icon_url</code> to override defaults.
          </p>
        </div>
      )}
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
    <div className="fixed inset-0 bg-black/70 flex items-center justify-center z-50 p-4">
      <div className="bg-gray-900 border border-gray-800 rounded-2xl shadow-2xl max-w-lg w-full p-6">
        <h3 className="text-lg font-semibold text-white mb-6">
          {webhook ? 'Edit Webhook' : 'Create Incoming Webhook'}
        </h3>

        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-1">
              Channel *
            </label>
            <select
              value={formData.channel_id}
              onChange={(e) => setFormData({ ...formData, channel_id: e.target.value })}
              required
              className="w-full px-3 py-2 border border-gray-700 rounded-lg bg-gray-800 text-white focus:ring-2 focus:ring-blue-500 focus:border-transparent"
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
            <label className="block text-sm font-medium text-gray-300 mb-1">
              Display Name *
            </label>
            <input
              type="text"
              value={formData.display_name}
              onChange={(e) => setFormData({ ...formData, display_name: e.target.value })}
              required
              maxLength={100}
              placeholder="e.g., GitHub Notifications"
              className="w-full px-3 py-2 border border-gray-700 rounded-lg bg-gray-800 text-white placeholder-gray-500 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            />
            <p className="text-xs text-gray-500 mt-1">
              This name will be shown as the sender of webhook messages
            </p>
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-300 mb-1">
              Description
            </label>
            <textarea
              value={formData.description}
              onChange={(e) => setFormData({ ...formData, description: e.target.value })}
              rows={2}
              placeholder="What is this webhook for?"
              className="w-full px-3 py-2 border border-gray-700 rounded-lg bg-gray-800 text-white placeholder-gray-500 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-300 mb-1">
              Username Override
            </label>
            <input
              type="text"
              value={formData.username}
              onChange={(e) => setFormData({ ...formData, username: e.target.value })}
              maxLength={100}
              placeholder="Optional: Override display name for messages"
              className="w-full px-3 py-2 border border-gray-700 rounded-lg bg-gray-800 text-white placeholder-gray-500 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            />
          </div>

          <div className="flex gap-3 pt-4">
            <button
              type="button"
              onClick={onClose}
              className="flex-1 px-4 py-2.5 text-sm font-medium text-gray-300 bg-gray-800 border border-gray-700 rounded-lg hover:bg-gray-700 transition-colors"
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
