'use client';

import { useState, useRef, useEffect } from 'react';
import dynamic from 'next/dynamic';
import { useAuth } from '@/lib/auth';
import { useWebSocketStore } from '@/lib/websocket';
import { extractUrls } from '@/lib/url-utils';
import type { Message } from '@/lib/types';
import MarkdownRenderer from './MarkdownRenderer';
import AttachmentDisplay from './AttachmentDisplay';
import LinkPreview from './LinkPreview';
import ReadReceiptModal from './ReadReceiptModal';
import EditHistoryModal from './EditHistoryModal';
import ReminderPickerModal from './ReminderPickerModal';
import PollDisplay from './PollDisplay';

// Dynamically import EmojiPicker to avoid SSR issues
const EmojiPicker = dynamic(() => import('emoji-picker-react'), { ssr: false });

// Import Theme type and EmojiClickData type
import { Theme, EmojiClickData } from 'emoji-picker-react';

interface MessageItemProps {
  message: Message;
  onReply?: (message: Message) => void;
  onOpenThread?: (message: Message) => void;
  onPin?: (message: Message) => void;
  onBookmark?: (message: Message) => void;
  onForward?: (message: Message) => void;
  isPinned?: boolean;
  isBookmarked?: boolean;
}

export default function MessageItem({ message, onReply, onOpenThread, onPin, onBookmark, onForward, isPinned = false, isBookmarked = false }: MessageItemProps) {
  const { user } = useAuth();
  const { addReaction: addReactionToStore, removeReaction: removeReactionFromStore, wsAddReaction, wsRemoveReaction, wsEditMessage, wsDeleteMessage, updateMessage: updateMessageInStore, deleteMessage: deleteMessageFromStore } = useWebSocketStore();
  const [showReactionPicker, setShowReactionPicker] = useState(false);
  const [showActions, setShowActions] = useState(false);
  const [isEditing, setIsEditing] = useState(false);
  const [editContent, setEditContent] = useState(message.content);
  const [showReadReceiptModal, setShowReadReceiptModal] = useState(false);
  const [showEditHistoryModal, setShowEditHistoryModal] = useState(false);
  const [showReminderPicker, setShowReminderPicker] = useState(false);
  const pickerRef = useRef<HTMLDivElement>(null);
  const hoverTimeoutRef = useRef<NodeJS.Timeout | null>(null);

  // Close picker when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (pickerRef.current && !pickerRef.current.contains(event.target as Node)) {
        setShowReactionPicker(false);
      }
    };

    if (showReactionPicker) {
      document.addEventListener('mousedown', handleClickOutside);
      return () => document.removeEventListener('mousedown', handleClickOutside);
    }
  }, [showReactionPicker]);

  // Cleanup hover timeout on unmount
  useEffect(() => {
    return () => {
      if (hoverTimeoutRef.current) {
        clearTimeout(hoverTimeoutRef.current);
      }
    };
  }, []);

  const handleMouseEnter = () => {
    hoverTimeoutRef.current = setTimeout(() => {
      setShowActions(true);
    }, 1000);
  };

  const handleMouseLeave = () => {
    if (hoverTimeoutRef.current) {
      clearTimeout(hoverTimeoutRef.current);
      hoverTimeoutRef.current = null;
    }
    setShowActions(false);
  };

  const isOwnMessage = user?.id === message.user_id;

  const formatTime = (timestamp: string) => {
    const date = new Date(timestamp);
    return date.toLocaleTimeString('en-US', {
      hour: 'numeric',
      minute: '2-digit',
      hour12: true,
    });
  };

  const handleAddReaction = (emoji: string) => {
    if (!user) return;

    // Optimistically update UI immediately
    addReactionToStore(message.id, user.id, emoji);
    setShowReactionPicker(false);

    // Send via WebSocket (no need to handle errors - the server will broadcast the result)
    wsAddReaction(message.id, emoji);
  };

  const handleEmojiClick = (emojiData: EmojiClickData) => {
    handleAddReaction(emojiData.emoji);
  };

  const handleRemoveReaction = (emoji: string) => {
    if (!user) return;

    // Optimistically update UI immediately
    removeReactionFromStore(message.id, user.id, emoji);

    // Send via WebSocket (no need to handle errors - the server will broadcast the result)
    wsRemoveReaction(message.id, emoji);
  };

  const handleEdit = () => {
    if (!editContent.trim() || editContent === message.content) {
      setIsEditing(false);
      setEditContent(message.content);
      return;
    }

    // Optimistically update UI
    updateMessageInStore(message.id, editContent, new Date().toISOString());
    setIsEditing(false);

    // Send via WebSocket
    wsEditMessage(message.id, editContent);
  };

  const handleDelete = () => {
    if (confirm('Are you sure you want to delete this message?')) {
      // Optimistically update UI
      deleteMessageFromStore(message.id);

      // Send via WebSocket
      wsDeleteMessage(message.id);
    }
  };

  // Group reactions by emoji
  const reactionCounts: Record<string, { count: number; userIds: string[] }> = {};
  if (message.reactions) {
    message.reactions.forEach((reaction) => {
      if (!reactionCounts[reaction.emoji]) {
        reactionCounts[reaction.emoji] = { count: 0, userIds: [] };
      }
      reactionCounts[reaction.emoji].count++;
      reactionCounts[reaction.emoji].userIds.push(reaction.user_id);
    });
  }

  return (
    <div
      className="group relative animate-slide-up"
      onMouseEnter={handleMouseEnter}
      onMouseLeave={handleMouseLeave}
    >
      <div className="flex gap-3">
        <div className="flex h-8 w-8 flex-shrink-0 items-center justify-center rounded-full bg-blue-600 text-sm font-semibold text-white">
          {message.user?.display_name?.charAt(0).toUpperCase() || '?'}
        </div>
        <div className="flex-1">
          <div className="mb-1 flex items-baseline gap-2">
            <span className="font-semibold text-white">
              {message.user?.display_name || 'Unknown User'}
            </span>
            <span className="text-xs text-gray-400">{formatTime(message.created_at)}</span>
            {message.edited_at && (
              <button
                onClick={() => setShowEditHistoryModal(true)}
                className="text-xs text-gray-500 hover:text-gray-300 hover:underline"
                title="View edit history"
              >
                (edited)
              </button>
            )}
          </div>
          {/* Forwarded attribution bar */}
          {message.forwarded_from_message_id && (
            <div className="mb-1 flex items-center gap-1 text-xs text-gray-400">
              <svg className="h-3 w-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 7l5 5m0 0l-5 5m5-5H6" />
              </svg>
              <span>
                Forwarded{message.forwarded_from_channel_name ? ` from #${message.forwarded_from_channel_name}` : ' from a direct message'}
              </span>
            </div>
          )}
          {isEditing ? (
            <div className="mb-2">
              <input
                type="text"
                value={editContent}
                onChange={(e) => setEditContent(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === 'Enter') handleEdit();
                  if (e.key === 'Escape') {
                    setIsEditing(false);
                    setEditContent(message.content);
                  }
                }}
                className="w-full rounded border border-gray-600 bg-gray-900 px-2 py-1 text-sm text-white focus:border-blue-500 focus:outline-none"
                autoFocus
              />
              <div className="mt-1 flex gap-2">
                <button
                  onClick={handleEdit}
                  className="text-xs text-blue-400 hover:underline"
                >
                  Save
                </button>
                <button
                  onClick={() => {
                    setIsEditing(false);
                    setEditContent(message.content);
                  }}
                  className="text-xs text-gray-400 hover:underline"
                >
                  Cancel
                </button>
              </div>
            </div>
          ) : (
            <div className="text-sm">
              <MarkdownRenderer content={message.content} />
            </div>
          )}

          {/* Attachments */}
          {message.attachments && message.attachments.length > 0 && (
            <AttachmentDisplay attachments={message.attachments} />
          )}

          {/* Poll Display */}
          {message.poll && (
            <PollDisplay poll={message.poll} messageId={message.id} />
          )}

          {/* Link Previews */}
          {!isEditing && extractUrls(message.content).map((url) => (
            <LinkPreview key={url} url={url} />
          ))}

          {/* Reactions - Mattermost style with inline + button */}
          <div className="relative mt-1 flex flex-wrap items-center gap-1">
            {Object.entries(reactionCounts).map(([emoji, data]) => {
              const hasReacted = user && data.userIds.includes(user.id);
              return (
                <button
                  key={emoji}
                  onClick={() => {
                    console.log('Reaction clicked:', { emoji, hasReacted, userId: user?.id, userIds: data.userIds });
                    if (hasReacted) {
                      handleRemoveReaction(emoji);
                    } else {
                      handleAddReaction(emoji);
                    }
                  }}
                  className={`rounded-full border px-2 py-0.5 text-xs transition-colors ${
                    hasReacted
                      ? 'border-blue-500 bg-blue-900 text-white'
                      : 'border-gray-600 bg-gray-800 text-gray-200 hover:border-gray-500'
                  }`}
                >
                  {emoji} {data.count}
                </button>
              );
            })}

            {/* Add reaction button - shows on hover (Mattermost style) */}
            {showActions && (
              <button
                onClick={() => setShowReactionPicker(!showReactionPicker)}
                className="rounded-full border border-gray-600 bg-gray-800 px-2 py-0.5 text-xs text-gray-400 transition-colors hover:border-gray-500 hover:bg-gray-700 hover:text-gray-300"
                title="Add reaction"
              >
                +
              </button>
            )}

            {/* Emoji Picker */}
            {showReactionPicker && (
              <div ref={pickerRef} className="absolute left-0 top-8 z-50">
                <EmojiPicker
                  onEmojiClick={handleEmojiClick}
                  theme={Theme.DARK}
                  width={350}
                  height={400}
                />
              </div>
            )}
          </div>

          {/* Thread indicator - show if message has replies */}
          {(message.reply_count ?? 0) > 0 && (
            <button
              onClick={() => onOpenThread?.(message)}
              className="mt-2 rounded-md border border-gray-700 bg-gray-900 p-2 text-left hover:border-gray-600 hover:bg-gray-800"
            >
              <div className="flex items-center gap-2">
                <svg className="h-4 w-4 text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M3 10h10a8 8 0 018 8v2M3 10l6 6m-6-6l6-6"
                  />
                </svg>
                <span className="text-xs font-semibold text-blue-400">
                  {message.reply_count} {message.reply_count === 1 ? 'reply' : 'replies'}
                </span>
              </div>
              {message.first_reply && (
                <div className="ml-6 mt-1 text-xs text-gray-400">
                  <span className="font-semibold text-gray-300">
                    {message.first_reply.user?.display_name || 'Unknown'}:
                  </span>{' '}
                  <span className="line-clamp-1">{message.first_reply.content}</span>
                </div>
              )}
            </button>
          )}

        </div>
      </div>

      {/* Modals */}
      <ReadReceiptModal
        messageId={message.id}
        isOpen={showReadReceiptModal}
        onClose={() => setShowReadReceiptModal(false)}
      />
      <EditHistoryModal
        messageId={message.id}
        currentContent={message.content}
        isOpen={showEditHistoryModal}
        onClose={() => setShowEditHistoryModal(false)}
      />

      {/* Message actions - Reply, Pin, Bookmark, Edit, and Delete */}
      {showActions && !isEditing && (
        <div className="absolute right-0 top-0 flex gap-1 rounded-lg border border-gray-700 bg-gray-900 p-1 shadow-sm">
          <button
            onClick={() => onReply?.(message)}
            className="rounded p-1 hover:bg-gray-800"
            title="Reply in thread"
          >
            <svg
              className="h-4 w-4 text-gray-300"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M3 10h10a8 8 0 018 8v2M3 10l6 6m-6-6l6-6"
              />
            </svg>
          </button>
          <button
            onClick={() => onPin?.(message)}
            className="rounded p-1 hover:bg-gray-800"
            title={isPinned ? "Unpin message" : "Pin message"}
          >
            <svg
              className={`h-4 w-4 ${isPinned ? 'text-yellow-400' : 'text-gray-300'}`}
              fill={isPinned ? "currentColor" : "none"}
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
          </button>
          <button
            onClick={() => onBookmark?.(message)}
            className="rounded p-1 hover:bg-gray-800"
            title={isBookmarked ? "Remove bookmark" : "Bookmark message"}
          >
            <svg
              className={`h-4 w-4 ${isBookmarked ? 'text-blue-400' : 'text-gray-300'}`}
              fill={isBookmarked ? "currentColor" : "none"}
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
          </button>
          <button
            onClick={() => onForward?.(message)}
            className="rounded p-1 hover:bg-gray-800"
            title="Forward message"
          >
            <svg
              className="h-4 w-4 text-gray-300"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M13 7l5 5m0 0l-5 5m5-5H6"
              />
            </svg>
          </button>
          <div className="relative">
            <button
              onClick={() => setShowReminderPicker(!showReminderPicker)}
              className="rounded p-1 hover:bg-gray-800"
              title="Remind me"
            >
              <svg
                className="h-4 w-4 text-gray-300"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"
                />
              </svg>
            </button>
            <ReminderPickerModal
              isOpen={showReminderPicker}
              onClose={() => setShowReminderPicker(false)}
              messageId={message.id}
            />
          </div>
          {isOwnMessage && (
            <>
              <button
                onClick={() => setIsEditing(true)}
                className="rounded p-1 hover:bg-gray-800"
                title="Edit message"
              >
                <svg
                  className="h-4 w-4 text-gray-300"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"
                  />
                </svg>
              </button>
              <button
                onClick={handleDelete}
                className="rounded p-1 hover:bg-gray-800"
                title="Delete message"
              >
                <svg
                  className="h-4 w-4 text-red-400"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
                  />
                </svg>
              </button>
            </>
          )}
        </div>
      )}
    </div>
  );
}
