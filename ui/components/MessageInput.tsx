'use client';

import { useState, useRef, useEffect } from 'react';
import { useWebSocketStore } from '@/lib/websocket';
import type { Message } from '@/lib/types';
import MarkdownToolbar from './MarkdownToolbar';
import MarkdownRenderer from './MarkdownRenderer';

interface MessageInputProps {
  channelId?: string;
  dmId?: string;
  replyTo?: Message;
  onClearReply?: () => void;
}

export default function MessageInput({ channelId, dmId, replyTo, onClearReply }: MessageInputProps) {
  const [message, setMessage] = useState('');
  const [showPreview, setShowPreview] = useState(false);
  const { sendMessage, sendTyping } = useWebSocketStore();
  const typingTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

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

  const handleChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
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

  const handleFormat = (before: string, after: string, placeholder?: string) => {
    const textarea = textareaRef.current;
    if (!textarea) return;

    const start = textarea.selectionStart;
    const end = textarea.selectionEnd;
    const selectedText = message.substring(start, end);
    const textToInsert = selectedText || placeholder || '';

    const newText =
      message.substring(0, start) +
      before +
      textToInsert +
      after +
      message.substring(end);

    setMessage(newText);

    // Set cursor position after formatting
    setTimeout(() => {
      const newCursorPos = start + before.length + textToInsert.length;
      textarea.focus();
      textarea.setSelectionRange(newCursorPos, newCursorPos);
    }, 0);
  };

  const handleTogglePreview = () => {
    setShowPreview(!showPreview);
  };

  useEffect(() => {
    return () => {
      if (typingTimeoutRef.current) {
        clearTimeout(typingTimeoutRef.current);
      }
    };
  }, []);

  return (
    <div className="border-t border-gray-800">
      {replyTo && (
        <div className="mx-6 mt-4 mb-2 flex items-center gap-2 rounded-lg bg-gray-800 px-3 py-2">
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
      <form onSubmit={handleSubmit} className="flex flex-col">
        <MarkdownToolbar
          onFormat={handleFormat}
          onTogglePreview={handleTogglePreview}
          showPreview={showPreview}
        />
        <div className="flex gap-2 px-6 py-4">
          {showPreview ? (
            <div className="flex-1 min-h-[100px] rounded-lg border border-gray-600 bg-gray-900 px-4 py-2 text-white">
              {message.trim() ? (
                <MarkdownRenderer content={message} />
              ) : (
                <div className="text-gray-400">Nothing to preview...</div>
              )}
            </div>
          ) : (
            <textarea
              ref={textareaRef}
              value={message}
              onChange={handleChange}
              placeholder={replyTo ? "Type your reply..." : "Type a message..."}
              className="flex-1 min-h-[100px] max-h-[300px] rounded-lg border border-gray-600 bg-gray-900 px-4 py-2 text-white placeholder-gray-400 focus:border-blue-500 focus:outline-none resize-y"
              rows={3}
            />
          )}
          <button
            type="submit"
            disabled={!message.trim()}
            className="rounded-lg bg-blue-600 px-6 py-2 font-medium text-white transition-colors hover:bg-blue-700 disabled:bg-gray-700 disabled:cursor-not-allowed h-fit"
          >
            Send
          </button>
        </div>
      </form>
    </div>
  );
}
