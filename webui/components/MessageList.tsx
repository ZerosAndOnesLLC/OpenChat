'use client';

import { useEffect, useRef, useState, useCallback } from 'react';
import type { Message } from '@/lib/types';
import MessageItem from './MessageItem';

// Track the latest new message for screen reader announcement
function useNewMessageAnnouncement(messages: Message[]) {
  const [announcement, setAnnouncement] = useState('');
  const prevCountRef = useRef(messages.length);

  useEffect(() => {
    if (messages.length > prevCountRef.current) {
      const newest = messages[messages.length - 1];
      if (newest) {
        const name = newest.user?.display_name || 'Someone';
        const preview = newest.content.length > 60 ? newest.content.slice(0, 60) + '...' : newest.content;
        setAnnouncement(`${name}: ${preview}`);
      }
    }
    prevCountRef.current = messages.length;
  }, [messages]);

  return announcement;
}

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
  const announcement = useNewMessageAnnouncement(messages);

  // Keyboard navigation for message list
  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    const target = e.target as HTMLElement;
    if (!target.hasAttribute('data-message-idx')) return;

    const idx = parseInt(target.getAttribute('data-message-idx') || '0', 10);

    if (e.key === 'ArrowDown') {
      e.preventDefault();
      const next = scrollRef.current?.querySelector<HTMLElement>(`[data-message-idx="${idx + 1}"]`);
      next?.focus();
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      const prev = scrollRef.current?.querySelector<HTMLElement>(`[data-message-idx="${idx - 1}"]`);
      prev?.focus();
    } else if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      if (idx >= 0 && idx < messages.length) {
        onOpenThread?.(messages[idx]);
      }
    } else if (e.key === 'r' && !e.ctrlKey && !e.metaKey) {
      if (idx >= 0 && idx < messages.length) {
        onReply?.(messages[idx]);
      }
    }
  }, [messages, onOpenThread, onReply]);

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
    <div ref={scrollRef} className="flex-1 overflow-y-auto px-6 py-4" onKeyDown={handleKeyDown}>
      {/* Screen reader announcement for new messages */}
      <div aria-live="polite" className="sr-only">{announcement}</div>
      <div role="list" className="space-y-4">
        {messages.map((message, index) => (
          <div
            key={message.id}
            role="listitem"
            tabIndex={0}
            data-message-idx={index}
            ref={index === lastReadIndex ? lastReadMessageRef : undefined}
            className="focus-visible:outline-2 focus-visible:outline-blue-500 focus-visible:outline-offset-2 rounded"
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
