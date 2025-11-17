'use client';

import { useState } from 'react';
import { useAuth } from '@/lib/auth';
import { apiClient } from '@/lib/api';
import type { Message } from '@/lib/types';

interface MessageItemProps {
  message: Message;
}

export default function MessageItem({ message }: MessageItemProps) {
  const { user } = useAuth();
  const [showReactionPicker, setShowReactionPicker] = useState(false);
  const [showActions, setShowActions] = useState(false);
  const [isEditing, setIsEditing] = useState(false);
  const [editContent, setEditContent] = useState(message.content);

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
    try {
      await apiClient.addReaction(message.id, { emoji });
      setShowReactionPicker(false);
    } catch (error) {
      console.error('Failed to add reaction:', error);
    }
  };

  const handleRemoveReaction = async (emoji: string) => {
    try {
      await apiClient.removeReaction(message.id, emoji);
    } catch (error) {
      console.error('Failed to remove reaction:', error);
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

  const commonEmojis = ['👍', '❤️', '😊', '🎉', '👏', '🔥'];

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
            <span className="font-semibold text-gray-900">
              {message.user?.display_name || 'Unknown User'}
            </span>
            <span className="text-xs text-gray-500">{formatTime(message.created_at)}</span>
            {message.edited_at && (
              <span className="text-xs text-gray-400">(edited)</span>
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
                className="w-full rounded border border-gray-300 px-2 py-1 text-sm focus:border-blue-500 focus:outline-none"
                autoFocus
              />
              <div className="mt-1 flex gap-2">
                <button
                  onClick={handleEdit}
                  className="text-xs text-blue-600 hover:underline"
                >
                  Save
                </button>
                <button
                  onClick={() => {
                    setIsEditing(false);
                    setEditContent(message.content);
                  }}
                  className="text-xs text-gray-600 hover:underline"
                >
                  Cancel
                </button>
              </div>
            </div>
          ) : (
            <p className="text-sm text-gray-900">{message.content}</p>
          )}

          {Object.keys(reactionCounts).length > 0 && (
            <div className="mt-1 flex flex-wrap gap-1">
              {Object.entries(reactionCounts).map(([emoji, data]) => {
                const hasReacted = user && data.userIds.includes(user.id);
                return (
                  <button
                    key={emoji}
                    onClick={() => hasReacted ? handleRemoveReaction(emoji) : handleAddReaction(emoji)}
                    className={`rounded-full border px-2 py-0.5 text-xs transition-colors ${
                      hasReacted
                        ? 'border-blue-500 bg-blue-50'
                        : 'border-gray-300 bg-white hover:border-gray-400'
                    }`}
                  >
                    {emoji} {data.count}
                  </button>
                );
              })}
            </div>
          )}
        </div>
      </div>

      {showActions && !isEditing && (
        <div className="absolute right-0 top-0 flex gap-1 rounded-lg border border-gray-200 bg-white p-1 shadow-sm">
          <button
            onClick={() => setShowReactionPicker(!showReactionPicker)}
            className="rounded p-1 hover:bg-gray-100"
            title="Add reaction"
          >
            <span className="text-sm">😊</span>
          </button>
          {isOwnMessage && (
            <>
              <button
                onClick={() => setIsEditing(true)}
                className="rounded p-1 hover:bg-gray-100"
                title="Edit message"
              >
                <svg
                  className="h-4 w-4 text-gray-600"
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
                className="rounded p-1 hover:bg-gray-100"
                title="Delete message"
              >
                <svg
                  className="h-4 w-4 text-red-600"
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

      {showReactionPicker && (
        <div className="absolute right-0 top-8 z-10 rounded-lg border border-gray-200 bg-white p-2 shadow-lg">
          <div className="flex gap-1">
            {commonEmojis.map((emoji) => (
              <button
                key={emoji}
                onClick={() => handleAddReaction(emoji)}
                className="rounded p-1 text-lg hover:bg-gray-100"
              >
                {emoji}
              </button>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
