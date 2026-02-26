'use client';

import { useEffect, useRef } from 'react';
import { useWebSocketStore } from '@/lib/websocket';
import { playSound } from '@/hooks/useNotificationSound';
import { useBrowserNotifications } from '@/hooks/useBrowserNotifications';

/**
 * Invisible component that listens to WebSocket events and triggers
 * notification sounds and browser notifications for new messages.
 */
export default function NotificationManager() {
  const { showNotification } = useBrowserNotifications();
  const currentUserId = useWebSocketStore((state) => state.currentUserId);
  const activeChannelId = useWebSocketStore((state) => state.activeChannelId);
  const activeDmId = useWebSocketStore((state) => state.activeDmId);

  // Refs to get latest values in the subscription callback
  const currentUserIdRef = useRef(currentUserId);
  const activeChannelIdRef = useRef(activeChannelId);
  const activeDmIdRef = useRef(activeDmId);

  useEffect(() => { currentUserIdRef.current = currentUserId; }, [currentUserId]);
  useEffect(() => { activeChannelIdRef.current = activeChannelId; }, [activeChannelId]);
  useEffect(() => { activeDmIdRef.current = activeDmId; }, [activeDmId]);

  // Keep track of last seen message count per key to detect new messages
  const lastMessageCountRef = useRef<Record<string, number>>({});
  const initializedRef = useRef(false);

  useEffect(() => {
    // Subscribe to store changes to detect new messages
    const unsub = useWebSocketStore.subscribe((state, prevState) => {
      // Skip until initial state is loaded
      if (!state.initialStateLoaded) return;

      // On first load, just record counts without playing sounds
      if (!initializedRef.current) {
        const counts: Record<string, number> = {};
        for (const key of Object.keys(state.messages)) {
          counts[key] = state.messages[key].length;
        }
        lastMessageCountRef.current = counts;
        initializedRef.current = true;
        return;
      }

      // Check each key for new messages
      for (const key of Object.keys(state.messages)) {
        const currentCount = state.messages[key].length;
        const prevCount = lastMessageCountRef.current[key] ?? 0;

        if (currentCount > prevCount) {
          // New message(s) arrived
          const newMessages = state.messages[key].slice(prevCount);

          for (const msg of newMessages) {
            // Skip messages from the current user
            if (msg.user_id === currentUserIdRef.current) continue;

            // Skip messages for the currently active/viewed channel or DM
            const isActiveChannel = activeChannelIdRef.current && msg.channel_id === activeChannelIdRef.current;
            const isActiveDm = activeDmIdRef.current && msg.dm_id === activeDmIdRef.current;
            if (isActiveChannel || isActiveDm) continue;

            // Check if this is a mention (content contains @username or is a DM)
            const isMention = msg.dm_id != null;
            const category = isMention ? 'mention' : 'message';

            playSound(category);
            showNotification(
              msg.user?.display_name || 'New message',
              msg.content.length > 100 ? msg.content.slice(0, 100) + '...' : msg.content,
            );

            // Only play one sound per batch to avoid cacophony
            break;
          }
        }

        lastMessageCountRef.current[key] = currentCount;
      }
    });

    return unsub;
  }, [showNotification]);

  // This component renders nothing
  return null;
}
