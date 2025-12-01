'use client';

import { useQuery } from '@tanstack/react-query';
import { apiClient } from '@/lib/api';
import { useAuth } from '@/lib/auth';
import type { Channel, DirectMessage } from '@/lib/types';
import { useWebSocketStore } from '@/lib/websocket';
import ChannelList from './ChannelList';
import DirectMessageList from './DirectMessageList';
import BookmarksList from './BookmarksList';
import UserProfile from './UserProfile';
import BrowseChannelsModal from './BrowseChannelsModal';
import NewDmModal from './NewDmModal';
import { useState, useMemo } from 'react';

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
  const [showBrowseChannels, setShowBrowseChannels] = useState(false);
  const [showNewDmModal, setShowNewDmModal] = useState(false);
  const [newChannelName, setNewChannelName] = useState('');
  const [newChannelType, setNewChannelType] = useState<'public' | 'private'>('public');

  const wsChannels = useWebSocketStore((state) => state.channels);
  const wsDms = useWebSocketStore((state) => state.dms);
  const initialStateLoaded = useWebSocketStore((state) => state.initialStateLoaded);

  // Fallback to HTTP if WebSocket initial state hasn't loaded yet
  const { data: httpChannels, refetch: refetchChannels } = useQuery({
    queryKey: ['channels'],
    queryFn: () => apiClient.listChannels(),
    enabled: !initialStateLoaded,
  });

  const { data: httpDms } = useQuery({
    queryKey: ['dms'],
    queryFn: () => apiClient.listDms(),
    enabled: !initialStateLoaded,
  });

  // Convert WebSocket metadata to Channel objects and sort alphabetically
  const channelsList = useMemo(() => {
    let channels: Channel[];
    if (initialStateLoaded && wsChannels.length > 0) {
      channels = wsChannels.map(ch => ({
        id: ch.id,
        name: ch.name,
        description: ch.description,
        channel_type: ch.channel_type,
        org_id: '',
        created_by: '',
        created_at: '',
        updated_at: '',
      } as Channel));
    } else {
      channels = Array.isArray(httpChannels) ? httpChannels : [];
    }
    // Sort channels alphabetically by name (case-insensitive)
    return channels.sort((a, b) => a.name.toLowerCase().localeCompare(b.name.toLowerCase()));
  }, [initialStateLoaded, wsChannels, httpChannels]);

  // Convert WebSocket DM metadata to DirectMessage objects and sort alphabetically
  const dmsList = useMemo(() => {
    let dms: DirectMessage[];
    if (initialStateLoaded && wsDms.length > 0) {
      dms = wsDms.map(dm => ({
        id: dm.id,
        org_id: '',
        created_by: '',
        created_at: '',
        participants: [{
          id: dm.other_user_id,
          display_name: dm.other_user_name,
          email: '',
          org_id: '',
          tv_user_id: '',
          status: 'online',
          created_at: '',
          updated_at: '',
        }],
      } as DirectMessage));
    } else {
      dms = Array.isArray(httpDms) ? httpDms : [];
    }
    // Sort DMs alphabetically by participant name (case-insensitive)
    return dms.sort((a, b) => {
      const nameA = a.participants?.[0]?.display_name || '';
      const nameB = b.participants?.[0]?.display_name || '';
      return nameA.toLowerCase().localeCompare(nameB.toLowerCase());
    });
  }, [initialStateLoaded, wsDms, httpDms]);

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
              <div className="flex items-center gap-1">
                <button
                  onClick={() => setShowBrowseChannels(true)}
                  className="rounded p-1 text-gray-400 hover:bg-gray-800 hover:text-white"
                  title="Browse channels"
                >
                  <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                  </svg>
                </button>
                <button
                  onClick={() => setShowCreateChannel(!showCreateChannel)}
                  className="rounded p-1 text-gray-400 hover:bg-gray-800 hover:text-white"
                  title="Create channel"
                >
                  <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
                  </svg>
                </button>
              </div>
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
              channels={channelsList}
              activeChannel={activeChannel}
              onSelectChannel={onSelectChannel}
            />
          </div>

          <div className="mb-4">
            <div className="mb-2 flex items-center justify-between px-2">
              <h2 className="text-sm font-semibold text-gray-400">Direct Messages</h2>
              <button
                onClick={() => setShowNewDmModal(true)}
                className="rounded p-1 text-gray-400 hover:bg-gray-800 hover:text-white"
                title="New message"
              >
                <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
                </svg>
              </button>
            </div>
            <DirectMessageList
              dms={dmsList}
              activeDm={activeDm}
              onSelectDm={onSelectDm}
            />
          </div>

          <div className="mb-4">
            <div className="mb-2 px-2">
              <h2 className="text-sm font-semibold text-gray-400">Bookmarks</h2>
            </div>
            <BookmarksList />
          </div>
        </div>
      </div>

      <div className="border-t border-gray-700">
        <UserProfile user={user} />
      </div>

      <BrowseChannelsModal
        isOpen={showBrowseChannels}
        onClose={() => setShowBrowseChannels(false)}
        onSelectChannel={onSelectChannel}
        currentChannels={channelsList}
      />

      <NewDmModal
        isOpen={showNewDmModal}
        onClose={() => setShowNewDmModal(false)}
        onSelectDm={onSelectDm}
        currentDms={dmsList}
      />
    </div>
  );
}
