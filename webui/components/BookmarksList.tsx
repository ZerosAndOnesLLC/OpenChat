'use client';

import { useQuery } from '@tanstack/react-query';
import { apiClient } from '@/lib/api';
import type { Bookmark } from '@/lib/types';

interface BookmarksListProps {
  onSelectBookmark?: (bookmark: Bookmark) => void;
}

export default function BookmarksList({ onSelectBookmark }: BookmarksListProps) {
  const { data: bookmarks = [], isLoading } = useQuery({
    queryKey: ['bookmarks'],
    queryFn: () => apiClient.getUserBookmarks(),
  });

  const formatTime = (timestamp: string) => {
    const date = new Date(timestamp);
    return date.toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
    });
  };

  if (isLoading) {
    return (
      <div className="px-2 py-2 text-xs text-gray-500">
        Loading bookmarks...
      </div>
    );
  }

  if (bookmarks.length === 0) {
    return (
      <div className="px-2 py-2 text-xs text-gray-500">
        No bookmarks yet
      </div>
    );
  }

  return (
    <div className="space-y-1">
      {bookmarks.map((bookmark) => (
        <button
          key={bookmark.id}
          onClick={() => onSelectBookmark?.(bookmark)}
          className="w-full rounded px-2 py-1.5 text-left text-sm hover:bg-gray-800"
        >
          <div className="flex items-start gap-2">
            <svg
              className="h-4 w-4 flex-shrink-0 text-blue-400"
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
            <div className="flex-1 overflow-hidden">
              <p className="truncate text-gray-300">
                {bookmark.message?.content || 'Message'}
              </p>
              <p className="text-xs text-gray-500">
                {bookmark.message?.created_at && formatTime(bookmark.message.created_at)}
              </p>
            </div>
          </div>
        </button>
      ))}
    </div>
  );
}
