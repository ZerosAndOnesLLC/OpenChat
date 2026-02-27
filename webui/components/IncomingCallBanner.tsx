'use client';

import { useEffect, useRef } from 'react';
import { Phone, Video, X } from 'lucide-react';
import { useWebSocketStore } from '@/lib/websocket';
import { apiClient } from '@/lib/api';
import { useNotificationSound } from '@/hooks/useNotificationSound';

export default function IncomingCallBanner() {
  const { incomingCall, dismissIncomingCall, setCurrentCall } = useWebSocketStore();
  const timeoutRef = useRef<NodeJS.Timeout | null>(null);
  const { playSound } = useNotificationSound();

  useEffect(() => {
    if (incomingCall) {
      playSound('mention');
      timeoutRef.current = setTimeout(() => {
        dismissIncomingCall();
      }, 30000);
    }
    return () => {
      if (timeoutRef.current) clearTimeout(timeoutRef.current);
    };
  }, [incomingCall, dismissIncomingCall, playSound]);

  if (!incomingCall) return null;

  const handleAccept = async () => {
    try {
      const resp = await apiClient.joinCall(incomingCall.call_id);
      setCurrentCall({
        call_id: resp.call_id,
        channel_id: incomingCall.channel_id,
        dm_id: incomingCall.dm_id,
        call_type: incomingCall.call_type,
        token: resp.token,
        livekit_url: resp.livekit_url,
        livekit_room_name: resp.livekit_room_name,
        started_at: new Date().toISOString(),
      });
      dismissIncomingCall();
    } catch (err) {
      console.error('Failed to join call:', err);
    }
  };

  const handleDecline = () => {
    dismissIncomingCall();
  };

  const isVideo = incomingCall.call_type === 'video';

  return (
    <div
      className="animate-slide-down fixed left-0 right-0 top-0 z-[60] flex items-center justify-between bg-gray-800 px-4 py-3 shadow-lg"
      role="alert"
      aria-live="assertive"
    >
      <div className="flex items-center gap-3">
        {isVideo ? (
          <Video className="h-5 w-5 text-blue-400" />
        ) : (
          <Phone className="h-5 w-5 text-green-400" />
        )}
        <span className="text-sm text-white">
          <strong>{incomingCall.started_by_name}</strong> is calling ({isVideo ? 'video' : 'audio'})
        </span>
      </div>
      <div className="flex items-center gap-2">
        <button
          onClick={handleAccept}
          className="flex items-center gap-1.5 rounded-md bg-green-600 px-3 py-1.5 text-sm font-medium text-white hover:bg-green-700"
          aria-label="Accept call"
        >
          <Phone className="h-4 w-4" />
          Accept
        </button>
        <button
          onClick={handleDecline}
          className="flex items-center gap-1.5 rounded-md bg-red-600 px-3 py-1.5 text-sm font-medium text-white hover:bg-red-700"
          aria-label="Decline call"
        >
          <X className="h-4 w-4" />
          Decline
        </button>
      </div>
    </div>
  );
}
