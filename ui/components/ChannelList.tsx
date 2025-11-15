'use client';

import type { Channel } from '@/lib/types';

interface ChannelListProps {
  channels: Channel[];
  activeChannel: Channel | null;
  onSelectChannel: (channel: Channel) => void;
}

export default function ChannelList({
  channels,
  activeChannel,
  onSelectChannel,
}: ChannelListProps) {
  return (
    <div className="space-y-1">
      {channels.map((channel) => (
        <button
          key={channel.id}
          onClick={() => onSelectChannel(channel)}
          className={`w-full rounded px-2 py-1.5 text-left text-sm transition-colors ${
            activeChannel?.id === channel.id
              ? 'bg-blue-600 text-white'
              : 'text-gray-300 hover:bg-gray-800'
          }`}
        >
          <div className="flex items-center">
            <span className="mr-1.5">
              {channel.channel_type === 'private' ? '🔒' : '#'}
            </span>
            <span className="truncate">{channel.name}</span>
          </div>
        </button>
      ))}
      {channels.length === 0 && (
        <p className="px-2 py-2 text-xs text-gray-500">No channels yet</p>
      )}
    </div>
  );
}
