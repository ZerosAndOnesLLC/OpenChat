'use client';

import { useState, useRef, useEffect } from 'react';
import dynamic from 'next/dynamic';
import { useAuth } from '@/lib/auth';
import { apiClient } from '@/lib/api';
import { useWebSocketStore } from '@/lib/websocket';
import type { Message } from '@/lib/types';

// Dynamically import EmojiPicker to avoid SSR issues
const EmojiPicker = dynamic(() => import('emoji-picker-react'), { ssr: false });

// Import Theme type
import { Theme } from 'emoji-picker-react';

interface MessageItemProps {
  message: Message;
}

export default function MessageItem({ message }: MessageItemProps) {
  const { user } = useAuth();
  const { addReaction: addReactionToStore, removeReaction: removeReactionFromStore } = useWebSocketStore();
  const [showReactionPicker, setShowReactionPicker] = useState(false);
  const [showActions, setShowActions] = useState(false);
  const [isEditing, setIsEditing] = useState(false);
  const [editContent, setEditContent] = useState(message.content);
  const pickerRef = useRef<HTMLDivElement>(null);

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

  const isOwnMessage = user?.id === message.user_id;

  const formatTime = (timestamp: string) => {
    const date = new Date(timestamp);
    return date.toLocaleTimeString('en-US', {
      hour: 'numeric',
      minute: '2-digit',
      hour12: true,
    });
  };

  const handleAddReaction = async (emoji: string) => {
    if (!user) return;

    // Optimistically update UI immediately
    addReactionToStore(message.id, user.id, emoji);
    setShowReactionPicker(false);

    // Then make API call
    try {
      await apiClient.addReaction(message.id, { emoji });
    } catch (error) {
      console.error('Failed to add reaction:', error);
      // Rollback on error
      removeReactionFromStore(message.id, user.id, emoji);
    }
  };

  const handleEmojiClick = (emojiData: any) => {
    handleAddReaction(emojiData.emoji);
  };

  const handleRemoveReaction = async (emoji: string) => {
    if (!user) return;

    // Optimistically update UI immediately
    removeReactionFromStore(message.id, user.id, emoji);

    // Then make API call
    try {
      await apiClient.removeReaction(message.id, emoji);
    } catch (error) {
      console.error('Failed to remove reaction:', error);
      // Rollback on error
      addReactionToStore(message.id, user.id, emoji);
    }
  };

  const handleEdit = async () => {
    if (!editContent.trim() || editContent === message.content) {
      setIsEditing(false);
      setEditContent(message.content);
      return;
    }

    try {
      await apiClient.updateMessage(message.id, { content: editContent });
      setIsEditing(false);
    } catch (error) {
      console.error('Failed to edit message:', error);
    }
  };

  const handleDelete = async () => {
    if (confirm('Are you sure you want to delete this message?')) {
      try {
        await apiClient.deleteMessage(message.id);
      } catch (error) {
        console.error('Failed to delete message:', error);
      }
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
      className="group relative"
      onMouseEnter={() => setShowActions(true)}
      onMouseLeave={() => setShowActions(false)}
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
              <span className="text-xs text-gray-500">(edited)</span>
            )}
          </div>
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
            <p className="text-sm text-gray-200">{message.content}</p>
          )}

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
        </div>
      </div>

      {/* Message actions - Edit and Delete only (Reaction moved inline) */}
      {showActions && !isEditing && isOwnMessage && (
        <div className="absolute right-0 top-0 flex gap-1 rounded-lg border border-gray-700 bg-gray-900 p-1 shadow-sm">
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
        </div>
      )}
    </div>
  );
}
