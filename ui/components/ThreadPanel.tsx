'use client';

import { useEffect, useState, useRef } from 'react';
import { useQuery } from '@tanstack/react-query';
import { apiClient } from '@/lib/api';
import type { Message, ThreadResponse } from '@/lib/types';
import MessageItem from './MessageItem';
import MessageInput from './MessageInput';

interface ThreadPanelProps {
  messageId: string;
  onClose: () => void;
}

export default function ThreadPanel({ messageId, onClose }: ThreadPanelProps) {
  const [replyTo, setReplyTo] = useState<Message | undefined>(undefined);
  const scrollRef = useRef<HTMLDivElement>(null);

  // Fetch thread messages
  const { data: threadData, isLoading, error } = useQuery<ThreadResponse>({
    queryKey: ['thread', messageId],
    queryFn: () => apiClient.getMessageThread(messageId),
    // No polling - rely on WebSocket updates for real-time thread updates
  });

  // Auto-scroll to bottom when new replies arrive
  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [threadData?.replies]);

  const handleReply = (message: Message) => {
    setReplyTo(message);
  };

  const handleClearReply = () => {
    setReplyTo(undefined);
  };

  if (isLoading) {
    return (
      <div className="flex h-full w-96 flex-col border-l border-gray-800 bg-black">
        <div className="flex h-14 items-center justify-between border-b border-gray-800 px-4">
          <h2 className="text-lg font-semibold text-white">Thread</h2>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-white"
            title="Close thread"
          >
            <svg className="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
        <div className="flex flex-1 items-center justify-center">
          <p className="text-gray-400">Loading thread...</p>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex h-full w-96 flex-col border-l border-gray-800 bg-black">
        <div className="flex h-14 items-center justify-between border-b border-gray-800 px-4">
          <h2 className="text-lg font-semibold text-white">Thread</h2>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-white"
            title="Close thread"
          >
            <svg className="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
        <div className="flex flex-1 items-center justify-center">
          <p className="text-red-400">Error loading thread</p>
        </div>
      </div>
    );
  }

  if (!threadData) {
    return null;
  }

  return (
    <div className="flex h-full w-96 flex-col border-l border-gray-800 bg-black">
      {/* Header */}
      <div className="flex h-14 flex-col justify-center border-b border-gray-800 px-4">
        <div className="flex items-center justify-between">
          <h2 className="text-lg font-semibold text-white">Thread</h2>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-white"
            title="Close thread"
          >
            <svg className="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
        {/* Breadcrumb */}
        <div className="mt-1 flex items-center gap-1 text-xs text-gray-400">
          <span className="truncate">
            {threadData.parent.user?.display_name || 'Unknown'}
          </span>
          <span>•</span>
          <span className="truncate">
            {threadData.replies.length} {threadData.replies.length === 1 ? 'reply' : 'replies'}
          </span>
        </div>
      </div>

      {/* Thread messages */}
      <div ref={scrollRef} className="flex-1 overflow-y-auto px-6 py-4">
        <div className="space-y-4">
          {/* Parent message */}
          <div className="border-b border-gray-800 pb-4">
            <MessageItem
              message={threadData.parent}
              onReply={handleReply}
            />
          </div>

          {/* Replies */}
          {threadData.replies.length === 0 ? (
            <p className="text-center text-gray-400">No replies yet. Start the conversation!</p>
          ) : (
            threadData.replies.map((reply) => (
              <MessageItem
                key={reply.id}
                message={reply}
                onReply={handleReply}
              />
            ))
          )}
        </div>
      </div>

      {/* Input for replying */}
      <MessageInput
        channelId={threadData.parent.channel_id}
        dmId={threadData.parent.dm_id}
        replyTo={replyTo || threadData.parent}
        onClearReply={handleClearReply}
      />
    </div>
  );
}
