'use client';

import { useEffect, useRef } from 'react';
import type { Message } from '@/lib/types';
import MessageItem from './MessageItem';

interface MessageListProps {
  messages: Message[];
  unreadCount?: number;
  onReply?: (message: Message) => void;
  onOpenThread?: (message: Message) => void;
  onPin?: (message: Message) => void;
  onBookmark?: (message: Message) => void;
  pinnedMessageIds?: Set<string>;
  bookmarkedMessageIds?: Set<string>;
}

export default function MessageList({ messages, unreadCount = 0, onReply, onOpenThread, onPin, onBookmark, pinnedMessageIds = new Set(), bookmarkedMessageIds = new Set() }: MessageListProps) {
  const scrollRef = useRef<HTMLDivElement>(null);
  const unreadMarkerRef = useRef<HTMLDivElement>(null);

  // Calculate the index of the first unread message
  const firstUnreadIndex = unreadCount > 0 && unreadCount < messages.length
    ? messages.length - unreadCount
    : -1;

  useEffect(() => {
    if (scrollRef.current) {
      // If there's an unread marker, scroll to it on mount
      if (unreadMarkerRef.current && firstUnreadIndex >= 0) {
        unreadMarkerRef.current.scrollIntoView({ behavior: 'smooth', block: 'center' });
      } else {
        // Otherwise scroll to bottom
        scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
      }
    }
  }, []); // Only run on mount

  // Scroll to bottom when new messages arrive (but not on mount)
  useEffect(() => {
    if (scrollRef.current && messages.length > 0) {
      const scrollElement = scrollRef.current;
      const isScrolledToBottom =
        scrollElement.scrollHeight - scrollElement.scrollTop - scrollElement.clientHeight < 100;

      // Auto-scroll to bottom only if user is already near the bottom
      if (isScrolledToBottom) {
        scrollElement.scrollTop = scrollElement.scrollHeight;
      }
    }
  }, [messages.length]); // Run when message count changes

  if (messages.length === 0) {
    return (
      <div className="flex flex-1 items-center justify-center">
        <p className="text-gray-400">No messages yet. Start the conversation!</p>
      </div>
    );
  }

  return (
    <div ref={scrollRef} className="flex-1 overflow-y-auto px-6 py-4">
      <div className="space-y-4">
        {messages.map((message, index) => (
          <div key={message.id}>
            {/* Show unread marker before the first unread message */}
            {index === firstUnreadIndex && (
              <div
                ref={unreadMarkerRef}
                className="relative my-4 flex items-center"
              >
                <div className="flex-grow border-t-2 border-red-500"></div>
                <span className="mx-4 flex-shrink whitespace-nowrap rounded-full bg-red-500 px-3 py-1 text-xs font-semibold text-white">
                  New messages
                </span>
                <div className="flex-grow border-t-2 border-red-500"></div>
              </div>
            )}
            <MessageItem
              message={message}
              onReply={onReply}
              onOpenThread={onOpenThread}
              onPin={onPin}
              onBookmark={onBookmark}
              isPinned={pinnedMessageIds.has(message.id)}
              isBookmarked={bookmarkedMessageIds.has(message.id)}
            />
          </div>
        ))}
      </div>
    </div>
  );
}
