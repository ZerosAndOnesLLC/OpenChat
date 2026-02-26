'use client';

import { useEffect, useRef, useState, useCallback } from 'react';
import type { Message } from '@/lib/types';
import MessageItem from './MessageItem';

interface MessageListProps {
  messages: Message[];
  unreadCount?: number;
  lastReadMessageId?: string;
  onReply?: (message: Message) => void;
  onOpenThread?: (message: Message) => void;
  onPin?: (message: Message) => void;
  onBookmark?: (message: Message) => void;
  onForward?: (message: Message) => void;
  pinnedMessageIds?: Set<string>;
  bookmarkedMessageIds?: Set<string>;
}

export default function MessageList({ messages, unreadCount = 0, lastReadMessageId, onReply, onOpenThread, onPin, onBookmark, onForward, pinnedMessageIds = new Set(), bookmarkedMessageIds = new Set() }: MessageListProps) {
  const scrollRef = useRef<HTMLDivElement>(null);
  const unreadMarkerRef = useRef<HTMLDivElement>(null);
  const lastReadMessageRef = useRef<HTMLDivElement>(null);
  const [hasInitialScrolled, setHasInitialScrolled] = useState(false);

  // Calculate the index of the last read message and first unread message
  let lastReadIndex = -1;
  let firstUnreadIndex = -1;
  if (lastReadMessageId) {
    lastReadIndex = messages.findIndex((msg) => msg.id === lastReadMessageId);
    if (lastReadIndex >= 0 && lastReadIndex < messages.length - 1) {
      firstUnreadIndex = lastReadIndex + 1;
    }
  } else if (unreadCount > 0 && unreadCount < messages.length) {
    // Fallback to unread count if no last read message ID
    firstUnreadIndex = messages.length - unreadCount;
    lastReadIndex = firstUnreadIndex - 1;
  }

  // Initial scroll - scroll to last read message or bottom
  useEffect(() => {
    if (hasInitialScrolled || !scrollRef.current || messages.length === 0) return;

    // If there's a last read message with unread messages after it, scroll to the last read message
    // This shows the user where they left off, with new messages visible below
    if (lastReadMessageRef.current && lastReadIndex >= 0 && firstUnreadIndex >= 0) {
      lastReadMessageRef.current.scrollIntoView({ behavior: 'smooth', block: 'start' });
    } else {
      // All messages read (or no messages) - scroll to bottom
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
    setHasInitialScrolled(true);
  }, [messages.length, lastReadIndex, firstUnreadIndex, hasInitialScrolled]);

  // Track if user is near the bottom before new messages arrive
  const wasNearBottomRef = useRef(true);

  // Helper to check if scrolled to bottom
  const isAtBottom = useCallback(() => {
    const scrollElement = scrollRef.current;
    if (!scrollElement) return true;
    return scrollElement.scrollHeight - scrollElement.scrollTop - scrollElement.clientHeight < 150;
  }, []);

  // Helper to scroll to bottom
  const scrollToBottom = useCallback(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, []);

  // Update wasNearBottom on scroll
  useEffect(() => {
    const scrollElement = scrollRef.current;
    if (!scrollElement) return;

    const handleScroll = () => {
      wasNearBottomRef.current = isAtBottom();
    };

    scrollElement.addEventListener('scroll', handleScroll);
    return () => scrollElement.removeEventListener('scroll', handleScroll);
  }, [isAtBottom]);

  // Maintain scroll position at bottom during resize
  useEffect(() => {
    const scrollElement = scrollRef.current;
    if (!scrollElement || !hasInitialScrolled) return;

    const resizeObserver = new ResizeObserver(() => {
      // If user was at bottom before resize, keep them at bottom
      if (wasNearBottomRef.current) {
        requestAnimationFrame(scrollToBottom);
      }
    });

    resizeObserver.observe(scrollElement);
    return () => resizeObserver.disconnect();
  }, [hasInitialScrolled, scrollToBottom]);

  // Scroll to bottom when new messages arrive (if user was near bottom)
  useEffect(() => {
    if (!scrollRef.current || messages.length === 0 || !hasInitialScrolled) return;

    // Auto-scroll to bottom if user was near the bottom
    if (wasNearBottomRef.current) {
      // Use requestAnimationFrame to ensure DOM has updated
      requestAnimationFrame(scrollToBottom);
    }
  }, [messages.length, hasInitialScrolled, scrollToBottom]);

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
          <div
            key={message.id}
            ref={index === lastReadIndex ? lastReadMessageRef : undefined}
          >
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
              onForward={onForward}
              isPinned={pinnedMessageIds.has(message.id)}
              isBookmarked={bookmarkedMessageIds.has(message.id)}
            />
          </div>
        ))}
      </div>
    </div>
  );
}
