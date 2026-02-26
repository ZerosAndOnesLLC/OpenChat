'use client';

import { useState, useMemo } from 'react';
import { apiClient } from '@/lib/api';
import { useWebSocketStore } from '@/lib/websocket';
import { useFocusTrap } from '@/hooks/useFocusTrap';
import type { Message } from '@/lib/types';

interface ForwardMessageModalProps {
  message: Message;
  isOpen: boolean;
  onClose: () => void;
  onSuccess: () => void;
}

export default function ForwardMessageModal({ message, isOpen, onClose, onSuccess }: ForwardMessageModalProps) {
  const { channels, dms } = useWebSocketStore();
  const [search, setSearch] = useState('');
  const [comment, setComment] = useState('');
  const [sending, setSending] = useState(false);
  const [error, setError] = useState('');
  const trapRef = useFocusTrap(isOpen);

  const filteredChannels = useMemo(() => {
    if (!search.trim()) return channels;
    const q = search.toLowerCase();
    return channels.filter(ch => ch.name.toLowerCase().includes(q));
  }, [channels, search]);

  const filteredDms = useMemo(() => {
    if (!search.trim()) return dms;
    const q = search.toLowerCase();
    return dms.filter(dm => dm.other_user_name.toLowerCase().includes(q));
  }, [dms, search]);

  const handleForward = async (targetChannelId?: string, targetDmId?: string) => {
    setSending(true);
    setError('');
    try {
      await apiClient.forwardMessage(message.id, {
        channel_id: targetChannelId,
        dm_id: targetDmId,
        comment: comment.trim() || undefined,
      });
      setComment('');
      setSearch('');
      onSuccess();
      onClose();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to forward message');
    } finally {
      setSending(false);
    }
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 animate-fade-in" onClick={onClose}>
      <div ref={trapRef} role="dialog" aria-modal="true" aria-labelledby="forward-modal-title" className="w-full max-w-md rounded-lg bg-gray-900 shadow-xl animate-modal-in" onClick={e => e.stopPropagation()}>
        <div className="flex items-center justify-between border-b border-gray-700 px-4 py-3">
          <h3 id="forward-modal-title" className="text-lg font-semibold text-white">Forward Message</h3>
          <button onClick={onClose} className="text-gray-400 hover:text-white">
            <svg className="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        <div className="px-4 py-3">
          {/* Message preview */}
          <div className="mb-3 rounded border border-gray-700 bg-gray-800 p-3">
            <div className="mb-1 text-xs text-gray-400">
              {message.user?.display_name || 'Unknown User'}
            </div>
            <div className="line-clamp-3 text-sm text-gray-300">{message.content}</div>
          </div>

          {/* Optional comment */}
          <textarea
            value={comment}
            onChange={e => setComment(e.target.value)}
            placeholder="Add a comment (optional)"
            className="mb-3 w-full rounded border border-gray-700 bg-gray-800 px-3 py-2 text-sm text-white placeholder-gray-500 focus:border-blue-500 focus:outline-none"
            rows={2}
          />

          {/* Search */}
          <input
            type="text"
            value={search}
            onChange={e => setSearch(e.target.value)}
            placeholder="Search channels and DMs..."
            className="mb-3 w-full rounded border border-gray-700 bg-gray-800 px-3 py-2 text-sm text-white placeholder-gray-500 focus:border-blue-500 focus:outline-none"
            autoFocus
          />

          {error && (
            <div className="mb-3 rounded bg-red-900/50 px-3 py-2 text-sm text-red-300">{error}</div>
          )}

          {/* Target list */}
          <div className="max-h-60 overflow-y-auto">
            {filteredChannels.length > 0 && (
              <>
                <div className="mb-1 text-xs font-semibold uppercase text-gray-500">Channels</div>
                {filteredChannels.map(ch => (
                  <button
                    key={ch.id}
                    onClick={() => handleForward(ch.id, undefined)}
                    disabled={sending}
                    className="flex w-full items-center gap-2 rounded px-3 py-2 text-left text-sm text-gray-300 hover:bg-gray-800 disabled:opacity-50"
                  >
                    <span className="text-gray-500">{ch.channel_type === 'private' ? '🔒' : '#'}</span>
                    <span>{ch.name}</span>
                  </button>
                ))}
              </>
            )}

            {filteredDms.length > 0 && (
              <>
                <div className="mb-1 mt-2 text-xs font-semibold uppercase text-gray-500">Direct Messages</div>
                {filteredDms.map(dm => (
                  <button
                    key={dm.id}
                    onClick={() => handleForward(undefined, dm.id)}
                    disabled={sending}
                    className="flex w-full items-center gap-2 rounded px-3 py-2 text-left text-sm text-gray-300 hover:bg-gray-800 disabled:opacity-50"
                  >
                    <span className="text-gray-500">💬</span>
                    <span>{dm.other_user_name}</span>
                  </button>
                ))}
              </>
            )}

            {filteredChannels.length === 0 && filteredDms.length === 0 && (
              <div className="py-4 text-center text-sm text-gray-500">No results found</div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
