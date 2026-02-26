'use client';

import { useState, useEffect } from 'react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { apiClient } from '@/lib/api';
import { useWebSocketStore } from '@/lib/websocket';
import type { Channel } from '@/lib/types';

interface EditChannelModalProps {
  channel: Channel;
  isOpen: boolean;
  onClose: () => void;
  onSuccess?: () => void;
}

export default function EditChannelModal({
  channel,
  isOpen,
  onClose,
  onSuccess,
}: EditChannelModalProps) {
  const [name, setName] = useState(channel.name);
  const [description, setDescription] = useState(channel.description || '');
  const [error, setError] = useState<string | null>(null);
  const queryClient = useQueryClient();
  const updateChannelInStore = useWebSocketStore((state) => state.updateChannel);

  useEffect(() => {
    if (isOpen) {
      setName(channel.name);
      setDescription(channel.description || '');
      setError(null);
    }
  }, [isOpen, channel]);

  const updateMutation = useMutation({
    mutationFn: () =>
      apiClient.updateChannel(channel.id, {
        name: name.trim(),
        description: description.trim() || undefined,
      }),
    onSuccess: () => {
      // Update WebSocket store so sidebar updates immediately
      updateChannelInStore(channel.id, {
        name: name.trim(),
        description: description.trim() || undefined,
      });
      queryClient.invalidateQueries({ queryKey: ['channels'] });
      onSuccess?.();
      onClose();
    },
    onError: (err: Error) => {
      setError(err.message || 'Failed to update channel');
    },
  });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!name.trim()) {
      setError('Channel name is required');
      return;
    }
    updateMutation.mutate();
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-50 animate-fade-in">
      <div className="w-full max-w-md rounded-lg bg-gray-900 p-6 shadow-xl animate-modal-in">
        <div className="mb-4 flex items-center justify-between">
          <h2 className="text-xl font-semibold text-white">Edit Channel</h2>
          <button
            onClick={onClose}
            className="rounded p-1 text-gray-400 hover:bg-gray-800 hover:text-white"
          >
            <svg className="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        <form onSubmit={handleSubmit}>
          <div className="mb-4">
            <label htmlFor="channel-name" className="mb-1 block text-sm font-medium text-gray-300">
              Channel Name
            </label>
            <input
              id="channel-name"
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              className="w-full rounded-lg border border-gray-700 bg-gray-800 px-3 py-2 text-white placeholder-gray-500 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
              placeholder="Enter channel name"
              maxLength={100}
              autoFocus
            />
          </div>

          <div className="mb-4">
            <label htmlFor="channel-description" className="mb-1 block text-sm font-medium text-gray-300">
              Description (optional)
            </label>
            <textarea
              id="channel-description"
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              className="w-full rounded-lg border border-gray-700 bg-gray-800 px-3 py-2 text-white placeholder-gray-500 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
              placeholder="What's this channel about?"
              rows={3}
            />
          </div>

          {error && (
            <div className="mb-4 rounded-lg bg-red-900 bg-opacity-50 px-3 py-2 text-sm text-red-300">
              {error}
            </div>
          )}

          <div className="flex justify-end gap-3">
            <button
              type="button"
              onClick={onClose}
              className="rounded-lg px-4 py-2 text-gray-300 hover:bg-gray-800"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={updateMutation.isPending || !name.trim()}
              className="rounded-lg bg-blue-600 px-4 py-2 text-white hover:bg-blue-700 disabled:cursor-not-allowed disabled:opacity-50"
            >
              {updateMutation.isPending ? 'Saving...' : 'Save Changes'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
