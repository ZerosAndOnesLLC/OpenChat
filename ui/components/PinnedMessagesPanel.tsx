'use client';

import { useQuery } from '@tanstack/react-query';
import { apiClient } from '@/lib/api';
import type { PinnedMessage } from '@/lib/types';
import { useState } from 'react';

interface PinnedMessagesPanelProps {
  channelId: string;
  onUnpin: (messageId: string) => void;
}

export default function PinnedMessagesPanel({ channelId, onUnpin }: PinnedMessagesPanelProps) {
  const [isExpanded, setIsExpanded] = useState(false);

  const { data: pinnedMessages = [], isLoading } = useQuery({
    queryKey: ['pinned-messages', channelId],
    queryFn: () => apiClient.getChannelPins(channelId),
    enabled: !!channelId,
  });

  if (isLoading || pinnedMessages.length === 0) {
    return null;
  }

  const formatTime = (timestamp: string) => {
    const date = new Date(timestamp);
    return date.toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      year: 'numeric',
    });
  };

  return (
    <div className="border-b border-gray-800 bg-gray-900">
      <div
        className="flex cursor-pointer items-center justify-between px-6 py-3 hover:bg-gray-800"
        onClick={() => setIsExpanded(!isExpanded)}
      >
        <div className="flex items-center gap-2">
          <svg
            className="h-4 w-4 text-yellow-400"
            fill="currentColor"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M5 5a2 2 0 012-2h10a2 2 0 012 2v16l-7-3.5L5 21V5z"
            />
          </svg>
          <span className="text-sm font-semibold text-white">
            {pinnedMessages.length} Pinned {pinnedMessages.length === 1 ? 'Message' : 'Messages'}
          </span>
        </div>
        <svg
          className={`h-4 w-4 text-gray-400 transition-transform ${isExpanded ? 'rotate-180' : ''}`}
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
        </svg>
      </div>

      {isExpanded && (
        <div className="max-h-64 overflow-y-auto border-t border-gray-800 px-6 py-3">
          {pinnedMessages.map((pin) => (
            <div
              key={pin.id}
              className="group mb-3 rounded-md border border-gray-700 bg-gray-950 p-3 last:mb-0 hover:border-gray-600"
            >
              <div className="mb-2 flex items-start justify-between">
                <div className="flex-1">
                  <div className="mb-1 flex items-center gap-2">
                    <span className="text-sm font-semibold text-white">
                      {pin.message?.user?.display_name || 'Unknown User'}
                    </span>
                    <span className="text-xs text-gray-500">
                      {pin.message?.created_at && formatTime(pin.message.created_at)}
                    </span>
                  </div>
                  <p className="text-sm text-gray-300 line-clamp-2">{pin.message?.content}</p>
                </div>
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    onUnpin(pin.message_id);
                  }}
                  className="ml-2 rounded p-1 opacity-0 hover:bg-gray-800 group-hover:opacity-100"
                  title="Unpin message"
                >
                  <svg className="h-4 w-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                  </svg>
                </button>
              </div>
              <div className="text-xs text-gray-500">
                Pinned {formatTime(pin.pinned_at)}
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
