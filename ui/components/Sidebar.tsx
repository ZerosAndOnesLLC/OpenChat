'use client';

import { useQuery } from '@tanstack/react-query';
import { apiClient } from '@/lib/api';
import { useAuth } from '@/lib/auth';
import type { Channel, DirectMessage } from '@/lib/types';
import ChannelList from './ChannelList';
import DirectMessageList from './DirectMessageList';
import UserProfile from './UserProfile';
import { useState } from 'react';

interface SidebarProps {
  activeChannel: Channel | null;
  activeDm: DirectMessage | null;
  onSelectChannel: (channel: Channel) => void;
  onSelectDm: (dm: DirectMessage) => void;
}

export default function Sidebar({
  activeChannel,
  activeDm,
  onSelectChannel,
  onSelectDm,
}: SidebarProps) {
  const { user } = useAuth();
  const [showCreateChannel, setShowCreateChannel] = useState(false);
  const [newChannelName, setNewChannelName] = useState('');
  const [newChannelType, setNewChannelType] = useState<'public' | 'private'>('public');

  const { data: channels = [], refetch: refetchChannels } = useQuery({
    queryKey: ['channels'],
    queryFn: () => apiClient.listChannels(),
  });

  const { data: dms = [] } = useQuery({
    queryKey: ['dms'],
    queryFn: () => apiClient.listDms(),
  });

  const handleCreateChannel = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newChannelName.trim()) return;

    try {
      const channel = await apiClient.createChannel({
        name: newChannelName,
        channel_type: newChannelType,
      });
      setNewChannelName('');
      setShowCreateChannel(false);
      refetchChannels();
      onSelectChannel(channel);
    } catch (error) {
      console.error('Failed to create channel:', error);
    }
  };

  return (
    <div className="flex w-64 flex-col bg-gray-900 text-white">
      <div className="flex h-14 items-center border-b border-gray-700 px-4">
        <h1 className="text-xl font-bold">OpenChat</h1>
      </div>

      <div className="flex-1 overflow-y-auto">
        <div className="px-2 py-4">
          <div className="mb-4">
            <div className="mb-2 flex items-center justify-between px-2">
              <h2 className="text-sm font-semibold text-gray-400">Channels</h2>
              <button
                onClick={() => setShowCreateChannel(!showCreateChannel)}
                className="text-gray-400 hover:text-white"
                title="Create channel"
              >
                <svg className="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
                </svg>
              </button>
            </div>

            {showCreateChannel && (
              <form onSubmit={handleCreateChannel} className="mb-2 px-2">
                <input
                  type="text"
                  value={newChannelName}
                  onChange={(e) => setNewChannelName(e.target.value)}
                  placeholder="Channel name"
                  className="mb-2 w-full rounded bg-gray-800 px-2 py-1 text-sm text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-blue-500"
                  autoFocus
                />
                <div className="mb-2 flex gap-2">
                  <label className="flex items-center text-xs">
                    <input
                      type="radio"
                      value="public"
                      checked={newChannelType === 'public'}
                      onChange={() => setNewChannelType('public')}
                      className="mr-1"
                    />
                    Public
                  </label>
                  <label className="flex items-center text-xs">
                    <input
                      type="radio"
                      value="private"
                      checked={newChannelType === 'private'}
                      onChange={() => setNewChannelType('private')}
                      className="mr-1"
                    />
                    Private
                  </label>
                </div>
                <div className="flex gap-2">
                  <button
                    type="submit"
                    className="flex-1 rounded bg-blue-600 px-2 py-1 text-xs hover:bg-blue-700"
                  >
                    Create
                  </button>
                  <button
                    type="button"
                    onClick={() => {
                      setShowCreateChannel(false);
                      setNewChannelName('');
                    }}
                    className="flex-1 rounded bg-gray-700 px-2 py-1 text-xs hover:bg-gray-600"
                  >
                    Cancel
                  </button>
                </div>
              </form>
            )}

            <ChannelList
              channels={channels}
              activeChannel={activeChannel}
              onSelectChannel={onSelectChannel}
            />
          </div>

          <div className="mb-4">
            <div className="mb-2 px-2">
              <h2 className="text-sm font-semibold text-gray-400">Direct Messages</h2>
            </div>
            <DirectMessageList
              dms={dms}
              activeDm={activeDm}
              onSelectDm={onSelectDm}
            />
          </div>
        </div>
      </div>

      <div className="border-t border-gray-700">
        <UserProfile user={user} />
      </div>
    </div>
  );
}
