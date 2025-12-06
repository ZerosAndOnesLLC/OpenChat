'use client';

import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { apiClient } from '@/lib/api';
import type { Channel, ChannelMetadata } from '@/lib/types';
import { useAuth } from '@/lib/auth';
import { useWebSocketStore } from '@/lib/websocket';

interface BrowseChannelsModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSelectChannel: (channel: Channel) => void;
  currentChannels: Channel[];
}

export default function BrowseChannelsModal({
  isOpen,
  onClose,
  onSelectChannel,
  currentChannels,
}: BrowseChannelsModalProps) {
  const { user } = useAuth();
  const [searchQuery, setSearchQuery] = useState('');
  const queryClient = useQueryClient();
  const addChannel = useWebSocketStore((state) => state.addChannel);

  const { data: publicChannels, isLoading } = useQuery({
    queryKey: ['public-channels'],
    queryFn: () => apiClient.listPublicChannels(),
    enabled: isOpen,
  });

  const joinChannelMutation = useMutation({
    mutationFn: async (channel: Channel) => {
      await apiClient.joinChannel(channel.id);
      return channel;
    },
    onSuccess: (channel) => {
      // Add to WebSocket store so it shows in sidebar immediately
      const channelMetadata: ChannelMetadata = {
        id: channel.id,
        name: channel.name,
        description: channel.description,
        channel_type: channel.channel_type,
        unread_count: 0,
      };
      addChannel(channelMetadata);
      queryClient.invalidateQueries({ queryKey: ['channels'] });
      queryClient.invalidateQueries({ queryKey: ['public-channels'] });
    },
  });

  const handleJoinChannel = async (channel: Channel) => {
    try {
      await joinChannelMutation.mutateAsync(channel);
      onSelectChannel(channel);
      onClose();
    } catch (error) {
      console.error('Failed to join channel:', error);
    }
  };

  if (!isOpen) return null;

  // API now returns only public channels user is NOT a member of
  const availableChannels = publicChannels || [];

  // Current channels that are public (for "Already Joined" section)
  const joinedPublicChannels = currentChannels.filter((c) => c.channel_type === 'public');

  // Filter by search query
  const filteredJoined = joinedPublicChannels.filter((c) =>
    c.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
    c.description?.toLowerCase().includes(searchQuery.toLowerCase())
  );
  const filteredAvailable = availableChannels.filter((c) =>
    c.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
    c.description?.toLowerCase().includes(searchQuery.toLowerCase())
  );

  return (
    <>
      {/* Backdrop */}
      <div
        className="fixed inset-0 z-40 bg-black bg-opacity-50"
        onClick={onClose}
      />

      {/* Modal */}
      <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
        <div className="w-full max-w-2xl rounded-lg bg-gray-900 shadow-xl">
          {/* Header */}
          <div className="flex items-center justify-between border-b border-gray-700 px-6 py-4">
            <h2 className="text-xl font-semibold text-white">Browse Channels</h2>
            <button
              onClick={onClose}
              className="text-gray-400 hover:text-white"
            >
              <svg className="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>

          {/* Search */}
          <div className="border-b border-gray-700 px-6 py-4">
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="Search channels..."
              className="w-full rounded-md border border-gray-600 bg-gray-800 px-4 py-2 text-sm text-white placeholder-gray-400 focus:border-blue-500 focus:outline-none"
              autoFocus
            />
          </div>

          {/* Content */}
          <div className="max-h-[60vh] overflow-y-auto px-6 py-4">
            {isLoading ? (
              <div className="py-8 text-center text-gray-400">Loading channels...</div>
            ) : (
              <>
                {/* Available to join */}
                {filteredAvailable.length > 0 && (
                  <div className="mb-6">
                    <h3 className="mb-3 text-sm font-semibold text-gray-400">
                      Available to Join ({filteredAvailable.length})
                    </h3>
                    <div className="space-y-2">
                      {filteredAvailable.map((channel) => (
                        <div
                          key={channel.id}
                          className="flex items-center justify-between rounded-lg border border-gray-700 bg-gray-800 p-3 transition-colors hover:border-gray-600"
                        >
                          <div className="flex-1">
                            <div className="flex items-center gap-2">
                              <span className="text-gray-400">#</span>
                              <h4 className="font-medium text-white">{channel.name}</h4>
                            </div>
                            {channel.description && (
                              <p className="mt-1 text-sm text-gray-400">{channel.description}</p>
                            )}
                          </div>
                          <button
                            onClick={() => handleJoinChannel(channel)}
                            disabled={joinChannelMutation.isPending}
                            className="ml-4 rounded-md bg-blue-600 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-blue-700 disabled:bg-gray-700 disabled:cursor-not-allowed"
                          >
                            {joinChannelMutation.isPending ? 'Joining...' : 'Join'}
                          </button>
                        </div>
                      ))}
                    </div>
                  </div>
                )}

                {/* Already joined */}
                {filteredJoined.length > 0 && (
                  <div>
                    <h3 className="mb-3 text-sm font-semibold text-gray-400">
                      Already Joined ({filteredJoined.length})
                    </h3>
                    <div className="space-y-2">
                      {filteredJoined.map((channel) => (
                        <div
                          key={channel.id}
                          className="flex cursor-pointer items-center justify-between rounded-lg border border-gray-700 bg-gray-800 p-3 transition-colors hover:border-gray-600 hover:bg-gray-750"
                          onClick={() => {
                            onSelectChannel(channel);
                            onClose();
                          }}
                        >
                          <div className="flex-1">
                            <div className="flex items-center gap-2">
                              <span className="text-gray-400">#</span>
                              <h4 className="font-medium text-white">{channel.name}</h4>
                            </div>
                            {channel.description && (
                              <p className="mt-1 text-sm text-gray-400">{channel.description}</p>
                            )}
                          </div>
                          <div className="ml-4 flex items-center gap-2 text-sm text-green-500">
                            <svg className="h-4 w-4" fill="currentColor" viewBox="0 0 20 20">
                              <path
                                fillRule="evenodd"
                                d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
                                clipRule="evenodd"
                              />
                            </svg>
                            <span>Joined</span>
                          </div>
                        </div>
                      ))}
                    </div>
                  </div>
                )}

                {/* No results */}
                {filteredAvailable.length === 0 && filteredJoined.length === 0 && (
                  <div className="py-8 text-center text-gray-400">
                    {searchQuery
                      ? 'No channels found matching your search'
                      : 'No public channels available'}
                  </div>
                )}
              </>
            )}
          </div>

          {/* Footer */}
          <div className="border-t border-gray-700 px-6 py-4">
            <button
              onClick={onClose}
              className="w-full rounded-md border border-gray-600 px-4 py-2 text-sm font-medium text-gray-300 transition-colors hover:bg-gray-800"
            >
              Close
            </button>
          </div>
        </div>
      </div>
    </>
  );
}
