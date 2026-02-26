'use client';

import { useState } from 'react';
import { useFocusTrap } from '@/hooks/useFocusTrap';

interface ScheduleSendModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSchedule: (scheduledAt: string) => void;
}

export default function ScheduleSendModal({ isOpen, onClose, onSchedule }: ScheduleSendModalProps) {
  const [customDate, setCustomDate] = useState('');
  const [showCustom, setShowCustom] = useState(false);
  const trapRef = useFocusTrap(isOpen);

  if (!isOpen) return null;

  const getQuickOptions = () => {
    const now = new Date();
    const options = [
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
        label: 'Next Monday at 9am',
        getDate: () => {
          const d = new Date(now);
          const daysUntilMonday = (8 - d.getDay()) % 7 || 7;
          d.setDate(d.getDate() + daysUntilMonday);
          d.setHours(9, 0, 0, 0);
          return d;
        },
      },
    ];
    return options;
  };

  const handleQuickOption = (getDate: () => Date) => {
    onSchedule(getDate().toISOString());
    onClose();
  };

  const handleCustomSchedule = () => {
    if (!customDate) return;
    const date = new Date(customDate);
    if (date <= new Date()) return;
    onSchedule(date.toISOString());
    setCustomDate('');
    setShowCustom(false);
    onClose();
  };

  const getMinDateTime = () => {
    const now = new Date();
    now.setMinutes(now.getMinutes() + 1);
    return now.toISOString().slice(0, 16);
  };

  return (
    <>
      <div className="fixed inset-0 bg-black bg-opacity-50 z-40 animate-fade-in" onClick={onClose} />
      <div className="fixed inset-0 flex items-center justify-center z-50 pointer-events-none">
        <div ref={trapRef} role="dialog" aria-modal="true" aria-labelledby="schedule-send-modal-title" className="bg-gray-900 border border-gray-700 rounded-lg shadow-xl w-80 pointer-events-auto animate-modal-in">
          <div className="p-4 border-b border-gray-700">
            <div className="flex items-center justify-between">
              <h3 id="schedule-send-modal-title" className="text-lg font-semibold text-white">Schedule Message</h3>
              <button onClick={onClose} className="text-gray-400 hover:text-white">
                <svg className="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>
          </div>
          <div className="p-2">
            {getQuickOptions().map((option) => (
              <button
                key={option.label}
                onClick={() => handleQuickOption(option.getDate)}
                className="w-full text-left px-4 py-2.5 text-sm text-gray-200 hover:bg-gray-800 rounded transition-colors"
              >
                {option.label}
              </button>
            ))}
            <div className="border-t border-gray-700 my-1" />
            {showCustom ? (
              <div className="px-4 py-2">
                <input
                  type="datetime-local"
                  value={customDate}
                  onChange={(e) => setCustomDate(e.target.value)}
                  min={getMinDateTime()}
                  className="w-full rounded bg-gray-800 border border-gray-600 px-3 py-2 text-sm text-white focus:outline-none focus:border-blue-500 [color-scheme:dark]"
                />
                <button
                  onClick={handleCustomSchedule}
                  disabled={!customDate}
                  className="mt-2 w-full rounded bg-blue-600 px-3 py-2 text-sm text-white hover:bg-blue-700 disabled:bg-gray-700 disabled:text-gray-400 disabled:cursor-not-allowed"
                >
                  Schedule
                </button>
              </div>
            ) : (
              <button
                onClick={() => setShowCustom(true)}
                className="w-full text-left px-4 py-2.5 text-sm text-blue-400 hover:bg-gray-800 rounded transition-colors"
              >
                Custom date & time...
              </button>
            )}
          </div>
        </div>
      </div>
    </>
  );
}
