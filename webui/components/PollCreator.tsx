'use client';

import { useState } from 'react';
import { apiClient } from '@/lib/api';
import { toastManager } from '@/lib/toast';

interface PollCreatorProps {
  isOpen: boolean;
  onClose: () => void;
  channelId?: string;
  dmId?: string;
}

export default function PollCreator({ isOpen, onClose, channelId, dmId }: PollCreatorProps) {
  const [question, setQuestion] = useState('');
  const [options, setOptions] = useState(['', '']);
  const [pollType, setPollType] = useState<'single' | 'multiple'>('single');
  const [anonymous, setAnonymous] = useState(false);
  const [hasExpiry, setHasExpiry] = useState(false);
  const [expiryHours, setExpiryHours] = useState(24);
  const [submitting, setSubmitting] = useState(false);

  if (!isOpen) return null;

  const addOption = () => {
    if (options.length < 10) {
      setOptions([...options, '']);
    }
  };

  const removeOption = (index: number) => {
    if (options.length > 2) {
      setOptions(options.filter((_, i) => i !== index));
    }
  };

  const updateOption = (index: number, value: string) => {
    const updated = [...options];
    updated[index] = value;
    setOptions(updated);
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!question.trim()) return;

    const validOptions = options.filter((o) => o.trim());
    if (validOptions.length < 2) {
      toastManager.error('At least 2 options are required');
      return;
    }

    try {
      setSubmitting(true);
      const expiresAt = hasExpiry
        ? new Date(Date.now() + expiryHours * 60 * 60 * 1000).toISOString()
        : undefined;

      await apiClient.createPoll({
        channel_id: channelId,
        dm_id: dmId,
        question: question.trim(),
        options: validOptions,
        poll_type: pollType,
        anonymous,
        expires_at: expiresAt,
      });

      setQuestion('');
      setOptions(['', '']);
      setPollType('single');
      setAnonymous(false);
      setHasExpiry(false);
      onClose();
    } catch (error) {
      toastManager.error('Failed to create poll');
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60">
      <div className="bg-gray-800 rounded-lg border border-gray-700 w-full max-w-md mx-4 max-h-[80vh] overflow-y-auto">
        <div className="flex items-center justify-between p-4 border-b border-gray-700">
          <h2 className="text-lg font-semibold text-white">Create Poll</h2>
          <button onClick={onClose} className="text-gray-400 hover:text-white">
            <svg className="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        <form onSubmit={handleSubmit} className="p-4 space-y-4">
          <div>
            <label className="block text-sm text-gray-400 mb-1">Question</label>
            <input
              type="text"
              value={question}
              onChange={(e) => setQuestion(e.target.value)}
              placeholder="What would you like to ask?"
              className="w-full px-3 py-2 bg-gray-900 border border-gray-700 rounded text-white text-sm focus:outline-none focus:border-blue-500"
              autoFocus
            />
          </div>

          <div>
            <label className="block text-sm text-gray-400 mb-1">Options</label>
            <div className="space-y-2">
              {options.map((opt, index) => (
                <div key={index} className="flex items-center gap-2">
                  <input
                    type="text"
                    value={opt}
                    onChange={(e) => updateOption(index, e.target.value)}
                    placeholder={`Option ${index + 1}`}
                    className="flex-1 px-3 py-2 bg-gray-900 border border-gray-700 rounded text-white text-sm focus:outline-none focus:border-blue-500"
                  />
                  {options.length > 2 && (
                    <button
                      type="button"
                      onClick={() => removeOption(index)}
                      className="text-gray-500 hover:text-red-400"
                    >
                      <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                      </svg>
                    </button>
                  )}
                </div>
              ))}
            </div>
            {options.length < 10 && (
              <button
                type="button"
                onClick={addOption}
                className="mt-2 text-sm text-blue-400 hover:text-blue-300"
              >
                + Add option
              </button>
            )}
          </div>

          <div className="flex items-center gap-4">
            <label className="flex items-center gap-2 text-sm text-gray-300 cursor-pointer">
              <input
                type="radio"
                checked={pollType === 'single'}
                onChange={() => setPollType('single')}
                className="text-blue-500"
              />
              Single choice
            </label>
            <label className="flex items-center gap-2 text-sm text-gray-300 cursor-pointer">
              <input
                type="radio"
                checked={pollType === 'multiple'}
                onChange={() => setPollType('multiple')}
                className="text-blue-500"
              />
              Multiple choice
            </label>
          </div>

          <label className="flex items-center gap-2 text-sm text-gray-300 cursor-pointer">
            <input
              type="checkbox"
              checked={anonymous}
              onChange={(e) => setAnonymous(e.target.checked)}
              className="rounded"
            />
            Anonymous voting
          </label>

          <div>
            <label className="flex items-center gap-2 text-sm text-gray-300 cursor-pointer">
              <input
                type="checkbox"
                checked={hasExpiry}
                onChange={(e) => setHasExpiry(e.target.checked)}
                className="rounded"
              />
              Set expiry
            </label>
            {hasExpiry && (
              <div className="mt-2 flex items-center gap-2">
                <input
                  type="number"
                  value={expiryHours}
                  onChange={(e) => setExpiryHours(Math.max(1, parseInt(e.target.value) || 1))}
                  min={1}
                  max={720}
                  className="w-20 px-2 py-1 bg-gray-900 border border-gray-700 rounded text-white text-sm"
                />
                <span className="text-sm text-gray-400">hours</span>
              </div>
            )}
          </div>

          <div className="flex justify-end gap-2 pt-2">
            <button
              type="button"
              onClick={onClose}
              className="px-4 py-2 text-gray-400 hover:text-white transition-colors"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={submitting || !question.trim() || options.filter((o) => o.trim()).length < 2}
              className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:bg-gray-700 disabled:text-gray-400 transition-colors"
            >
              {submitting ? 'Creating...' : 'Create Poll'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
