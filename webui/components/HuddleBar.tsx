'use client';

import { useState } from 'react';
import { Headphones, Mic, MicOff, PhoneOff } from 'lucide-react';
import { useWebSocketStore } from '@/lib/websocket';
import { apiClient } from '@/lib/api';

interface HuddleBarProps {
  channelId: string;
}

export default function HuddleBar({ channelId }: HuddleBarProps) {
  const { activeCalls, currentCall, setCurrentCall } = useWebSocketStore();
  const [joining, setJoining] = useState(false);

  // Find active huddle for this channel
  const huddle = Object.values(activeCalls).find(
    (c) => c.channel_id === channelId && c.is_huddle && c.status !== 'ended'
  );

  if (!huddle) return null;

  const isInHuddle = currentCall?.call_id === huddle.id;

  const handleJoin = async () => {
    setJoining(true);
    try {
      const resp = await apiClient.joinHuddle(channelId);
      setCurrentCall({
        call_id: resp.call_id,
        channel_id: channelId,
        call_type: 'audio',
        token: resp.token,
        livekit_url: resp.livekit_url,
        livekit_room_name: resp.livekit_room_name,
        started_at: new Date().toISOString(),
      });
    } catch (err) {
      console.error('Failed to join huddle:', err);
    } finally {
      setJoining(false);
    }
  };

  const handleLeave = async () => {
    try {
      await apiClient.leaveHuddle(channelId);
    } catch {
      // Ignore
    }
    setCurrentCall(null);
  };

  return (
    <div className="flex h-12 items-center border-t border-gray-800 bg-gray-900 px-4">
      <Headphones className="mr-2 h-4 w-4 text-green-400" />
      <span className="text-sm text-gray-300">
        Huddle ({huddle.participant_count} {huddle.participant_count === 1 ? 'person' : 'people'})
      </span>
      <div className="ml-auto flex items-center gap-2">
        {isInHuddle ? (
          <button
            onClick={handleLeave}
            className="flex items-center gap-1.5 rounded-md bg-red-600 px-3 py-1 text-sm text-white hover:bg-red-700"
          >
            <PhoneOff className="h-3.5 w-3.5" />
            Leave
          </button>
        ) : (
          <button
            onClick={handleJoin}
            disabled={joining}
            className="flex items-center gap-1.5 rounded-md bg-green-600 px-3 py-1 text-sm text-white hover:bg-green-700 disabled:opacity-50"
          >
            <Headphones className="h-3.5 w-3.5" />
            {joining ? 'Joining...' : 'Join'}
          </button>
        )}
      </div>
    </div>
  );
}
