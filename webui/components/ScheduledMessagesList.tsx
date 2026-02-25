'use client';

import { useState } from 'react';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import { apiClient } from '@/lib/api';
import { toastManager } from '@/lib/toast';
import { format } from 'date-fns';

interface ScheduledMessagesListProps {
  isOpen: boolean;
  onClose: () => void;
}

export default function ScheduledMessagesList({ isOpen, onClose }: ScheduledMessagesListProps) {
  const queryClient = useQueryClient();
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editContent, setEditContent] = useState('');
  const [editDate, setEditDate] = useState('');

  const { data: messages = [], isLoading } = useQuery({
    queryKey: ['scheduled-messages'],
    queryFn: () => apiClient.listScheduledMessages(),
    enabled: isOpen,
  });

  const handleDelete = async (id: string) => {
    if (!confirm('Cancel this scheduled message?')) return;
    try {
      await apiClient.deleteScheduledMessage(id);
      queryClient.invalidateQueries({ queryKey: ['scheduled-messages'] });
      toastManager.success('Scheduled message cancelled');
    } catch (error) {
      toastManager.error('Failed to cancel scheduled message');
    }
  };

  const handleEdit = async (id: string) => {
    try {
      await apiClient.updateScheduledMessage(id, {
        content: editContent || undefined,
        scheduled_at: editDate ? new Date(editDate).toISOString() : undefined,
      });
      queryClient.invalidateQueries({ queryKey: ['scheduled-messages'] });
      setEditingId(null);
      toastManager.success('Scheduled message updated');
    } catch (error) {
      toastManager.error('Failed to update scheduled message');
    }
  };

  const startEdit = (msg: { id: string; content: string; scheduled_at: string }) => {
    setEditingId(msg.id);
    setEditContent(msg.content);
    setEditDate(new Date(msg.scheduled_at).toISOString().slice(0, 16));
  };

  const getMinDateTime = () => {
    const now = new Date();
    now.setMinutes(now.getMinutes() + 1);
    return now.toISOString().slice(0, 16);
  };

  if (!isOpen) return null;

  return (
    <>
      <div className="fixed inset-0 bg-black bg-opacity-50 z-40" onClick={onClose} />
      <div className="fixed inset-0 flex items-center justify-center z-50 pointer-events-none">
        <div className="bg-gray-900 border border-gray-700 rounded-lg shadow-xl w-[480px] max-h-[600px] flex flex-col pointer-events-auto">
          <div className="p-4 border-b border-gray-700">
            <div className="flex items-center justify-between">
              <h3 className="text-lg font-semibold text-white">Scheduled Messages</h3>
              <button onClick={onClose} className="text-gray-400 hover:text-white">
                <svg className="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>
          </div>
          <div className="flex-1 overflow-y-auto">
            {isLoading ? (
              <div className="flex items-center justify-center py-8 text-gray-500">Loading...</div>
            ) : messages.length === 0 ? (
              <div className="flex items-center justify-center py-8 text-gray-500">No scheduled messages</div>
            ) : (
              <div className="divide-y divide-gray-800">
                {messages.map((msg) => (
                  <div key={msg.id} className="p-4">
                    {editingId === msg.id ? (
                      <div className="space-y-2">
                        <textarea
                          value={editContent}
                          onChange={(e) => setEditContent(e.target.value)}
                          className="w-full rounded bg-gray-800 border border-gray-600 px-3 py-2 text-sm text-white focus:outline-none focus:border-blue-500 resize-none"
                          rows={2}
                        />
                        <input
                          type="datetime-local"
                          value={editDate}
                          onChange={(e) => setEditDate(e.target.value)}
                          min={getMinDateTime()}
                          className="w-full rounded bg-gray-800 border border-gray-600 px-3 py-2 text-sm text-white focus:outline-none focus:border-blue-500 [color-scheme:dark]"
                        />
                        <div className="flex gap-2">
                          <button
                            onClick={() => handleEdit(msg.id)}
                            className="rounded bg-blue-600 px-3 py-1 text-xs text-white hover:bg-blue-700"
                          >
                            Save
                          </button>
                          <button
                            onClick={() => setEditingId(null)}
                            className="rounded bg-gray-700 px-3 py-1 text-xs text-gray-300 hover:bg-gray-600"
                          >
                            Cancel
                          </button>
                        </div>
                      </div>
                    ) : (
                      <>
                        <p className="text-sm text-gray-200 line-clamp-2">{msg.content}</p>
                        <div className="mt-1 flex items-center justify-between">
                          <span className="text-xs text-gray-500">
                            {format(new Date(msg.scheduled_at), 'MMM d, yyyy h:mm a')}
                          </span>
                          <div className="flex gap-2">
                            <button
                              onClick={() => startEdit(msg)}
                              className="text-xs text-blue-400 hover:text-blue-300"
                            >
                              Edit
                            </button>
                            <button
                              onClick={() => handleDelete(msg.id)}
                              className="text-xs text-red-400 hover:text-red-300"
                            >
                              Cancel
                            </button>
                          </div>
                        </div>
                      </>
                    )}
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>
      </div>
    </>
  );
}
