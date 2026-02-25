'use client';

import { useState } from 'react';
import { apiClient } from '@/lib/api';
import { toastManager } from '@/lib/toast';

interface ReminderPickerModalProps {
  isOpen: boolean;
  onClose: () => void;
  messageId: string;
}

export default function ReminderPickerModal({ isOpen, onClose, messageId }: ReminderPickerModalProps) {
  const [customDate, setCustomDate] = useState('');
  const [showCustom, setShowCustom] = useState(false);
  const [loading, setLoading] = useState(false);

  if (!isOpen) return null;

  const getQuickOptions = () => {
    const now = new Date();
    return [
      {
        label: 'In 30 minutes',
        getDate: () => new Date(now.getTime() + 30 * 60 * 1000),
      },
      {
        label: 'In 1 hour',
        getDate: () => new Date(now.getTime() + 60 * 60 * 1000),
      },
      {
        label: 'In 3 hours',
        getDate: () => new Date(now.getTime() + 3 * 60 * 60 * 1000),
      },
      {
        label: 'Tomorrow at 9am',
        getDate: () => {
          const d = new Date(now);
          d.setDate(d.getDate() + 1);
          d.setHours(9, 0, 0, 0);
          return d;
        },
      },
      {
        label: 'Next week',
        getDate: () => {
          const d = new Date(now);
          const daysUntilMonday = (8 - d.getDay()) % 7 || 7;
          d.setDate(d.getDate() + daysUntilMonday);
          d.setHours(9, 0, 0, 0);
          return d;
        },
      },
    ];
  };

  const createReminder = async (remindAt: Date) => {
    setLoading(true);
    try {
      await apiClient.createReminder({
        message_id: messageId,
        remind_at: remindAt.toISOString(),
      });
      toastManager.success('Reminder set');
      onClose();
    } catch (error) {
      toastManager.error('Failed to set reminder');
    } finally {
      setLoading(false);
    }
  };

  const handleCustomSchedule = () => {
    if (!customDate) return;
    const date = new Date(customDate);
    if (date <= new Date()) return;
    createReminder(date);
  };

  const getMinDateTime = () => {
    const now = new Date();
    now.setMinutes(now.getMinutes() + 1);
    return now.toISOString().slice(0, 16);
  };

  return (
    <>
      <div className="fixed inset-0 z-40" onClick={onClose} />
      <div className="absolute right-0 top-8 z-50 bg-gray-900 border border-gray-700 rounded-lg shadow-xl w-64">
        <div className="p-2">
          <div className="px-3 py-2 text-xs font-semibold text-gray-400 uppercase">Remind me</div>
          {getQuickOptions().map((option) => (
            <button
              key={option.label}
              onClick={() => createReminder(option.getDate())}
              disabled={loading}
              className="w-full text-left px-3 py-2 text-sm text-gray-200 hover:bg-gray-800 rounded transition-colors disabled:opacity-50"
            >
              {option.label}
            </button>
          ))}
          <div className="border-t border-gray-700 my-1" />
          {showCustom ? (
            <div className="px-3 py-2">
              <input
                type="datetime-local"
                value={customDate}
                onChange={(e) => setCustomDate(e.target.value)}
                min={getMinDateTime()}
                className="w-full rounded bg-gray-800 border border-gray-600 px-2 py-1.5 text-sm text-white focus:outline-none focus:border-blue-500 [color-scheme:dark]"
              />
              <button
                onClick={handleCustomSchedule}
                disabled={!customDate || loading}
                className="mt-2 w-full rounded bg-blue-600 px-3 py-1.5 text-sm text-white hover:bg-blue-700 disabled:bg-gray-700 disabled:text-gray-400 disabled:cursor-not-allowed"
              >
                Set Reminder
              </button>
            </div>
          ) : (
            <button
              onClick={() => setShowCustom(true)}
              className="w-full text-left px-3 py-2 text-sm text-blue-400 hover:bg-gray-800 rounded transition-colors"
            >
              Custom...
            </button>
          )}
        </div>
      </div>
    </>
  );
}
