'use client';

import { useState, useRef, useEffect } from 'react';
import { useWebSocketStore } from '@/lib/websocket';
import type { Message } from '@/lib/types';

interface MessageInputProps {
  channelId?: string;
  dmId?: string;
  replyTo?: Message;
  onClearReply?: () => void;
}

export default function MessageInput({ channelId, dmId, replyTo, onClearReply }: MessageInputProps) {
  const [message, setMessage] = useState('');
  const { sendMessage, sendTyping } = useWebSocketStore();
  const typingTimeoutRef = useRef<NodeJS.Timeout | null>(null);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!message.trim()) return;

    sendMessage(channelId, dmId, message.trim(), replyTo?.id);
    setMessage('');

    // Clear reply after sending
    if (onClearReply) {
      onClearReply();
    }

    if (typingTimeoutRef.current) {
      clearTimeout(typingTimeoutRef.current);
      typingTimeoutRef.current = null;
    }
  };

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setMessage(e.target.value);

    if (typingTimeoutRef.current) {
      clearTimeout(typingTimeoutRef.current);
    }

    if (e.target.value.trim()) {
      sendTyping(channelId, dmId);
      typingTimeoutRef.current = setTimeout(() => {
        typingTimeoutRef.current = null;
      }, 3000);
    }
  };

  useEffect(() => {
    return () => {
      if (typingTimeoutRef.current) {
        clearTimeout(typingTimeoutRef.current);
      }
    };
  }, []);

  return (
    <div className="border-t border-gray-800 px-6 py-4">
      {replyTo && (
        <div className="mb-2 flex items-center gap-2 rounded-lg bg-gray-800 px-3 py-2">
          <div className="flex-1">
            <div className="text-xs text-gray-400">
              Replying to <span className="font-semibold text-white">{replyTo.user?.display_name || 'Unknown User'}</span>
            </div>
            <div className="truncate text-sm text-gray-300">{replyTo.content}</div>
          </div>
          <button
            onClick={onClearReply}
            className="text-gray-400 hover:text-white"
            title="Cancel reply"
          >
            <svg className="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
      )}
      <form onSubmit={handleSubmit}>
        <div className="flex gap-2">
          <input
            type="text"
            value={message}
            onChange={handleChange}
            placeholder={replyTo ? "Type your reply..." : "Type a message..."}
            className="flex-1 rounded-lg border border-gray-600 bg-gray-900 px-4 py-2 text-white placeholder-gray-400 focus:border-blue-500 focus:outline-none"
          />
          <button
            type="submit"
            disabled={!message.trim()}
            className="rounded-lg bg-blue-600 px-6 py-2 font-medium text-white transition-colors hover:bg-blue-700 disabled:bg-gray-700 disabled:cursor-not-allowed"
          >
            Send
          </button>
        </div>
      </form>
    </div>
  );
}
