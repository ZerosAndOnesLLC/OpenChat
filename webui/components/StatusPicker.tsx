'use client';

import { useState, useRef, useEffect } from 'react';
import dynamic from 'next/dynamic';
import { apiClient } from '@/lib/api';
import { useWebSocketStore } from '@/lib/websocket';
import type { UpdateUserStatusRequest } from '@/lib/types';

// Dynamically import EmojiPicker to avoid SSR issues
const EmojiPicker = dynamic(() => import('emoji-picker-react'), { ssr: false });
import { Theme } from 'emoji-picker-react';

interface StatusPickerProps {
  userId?: string;
  currentStatus?: 'online' | 'offline' | 'away' | 'dnd';
  currentCustomMessage?: string;
  currentEmoji?: string;
  onStatusUpdate?: () => void;
}

const STATUS_OPTIONS = [
  {
    value: 'online' as const,
    label: 'Online',
    icon: '🟢',
    description: 'Available to chat',
  },
  {
    value: 'away' as const,
    label: 'Away',
    icon: '🟡',
    description: 'Stepped away',
  },
  {
    value: 'dnd' as const,
    label: 'Do Not Disturb',
    icon: '🔴',
    description: 'Please do not disturb',
  },
  {
    value: 'offline' as const,
    label: 'Offline',
    icon: '⚫',
    description: 'Not available',
  },
];

const TIME_OPTIONS = [
  { value: 30, label: '30 minutes' },
  { value: 60, label: '1 hour' },
  { value: 240, label: '4 hours' },
  { value: 1440, label: 'Today' },
  { value: null, label: 'Don\'t clear' },
];

export default function StatusPicker({
  userId,
  currentStatus = 'online',
  currentCustomMessage = '',
  currentEmoji = '',
  onStatusUpdate,
}: StatusPickerProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [selectedStatus, setSelectedStatus] = useState(currentStatus);
  const [customMessage, setCustomMessage] = useState(currentCustomMessage);
  const [emoji, setEmoji] = useState(currentEmoji);
  const [clearAfter, setClearAfter] = useState<number | null>(null);
  const [backAt, setBackAt] = useState<string>('');
  const [showEmojiPicker, setShowEmojiPicker] = useState(false);
  const [saving, setSaving] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);
  const emojiPickerRef = useRef<HTMLDivElement>(null);

  // Subscribe to WebSocket status updates for this user
  const wsStatusDetails = useWebSocketStore((state) =>
    userId ? state.userStatusDetails[userId] : undefined
  );

  // Update local state when WebSocket status changes
  useEffect(() => {
    if (wsStatusDetails) {
      setSelectedStatus(wsStatusDetails.status);
      setCustomMessage(wsStatusDetails.custom_message || '');
      setEmoji(wsStatusDetails.emoji || '');
    }
  }, [wsStatusDetails]);

  // Sync with props when they change (e.g., initial load)
  useEffect(() => {
    setSelectedStatus(currentStatus);
    setCustomMessage(currentCustomMessage);
    setEmoji(currentEmoji);
  }, [currentStatus, currentCustomMessage, currentEmoji]);

  // Close dropdown when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setIsOpen(false);
      }
      if (emojiPickerRef.current && !emojiPickerRef.current.contains(event.target as Node)) {
        setShowEmojiPicker(false);
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  // Get the display status from WebSocket store or props
  const displayStatus = wsStatusDetails?.status || currentStatus;
  const displayCustomMessage = wsStatusDetails?.custom_message || currentCustomMessage;
  const displayEmoji = wsStatusDetails?.emoji || currentEmoji;

  const getCurrentStatusOption = () => {
    return STATUS_OPTIONS.find(opt => opt.value === displayStatus) || STATUS_OPTIONS[0];
  };

  const handleSave = async () => {
    try {
      setSaving(true);
      const data: UpdateUserStatusRequest = {
        status: selectedStatus,
        custom_message: customMessage || undefined,
        emoji: emoji || undefined,
        clear_after_minutes: clearAfter || undefined,
        back_at: backAt || undefined,
      };
      await apiClient.updateMyStatus(data);
      setIsOpen(false);
      // Reset back_at after save
      setBackAt('');
      onStatusUpdate?.();
    } catch (error) {
      console.error('Failed to update status:', error);
    } finally {
      setSaving(false);
    }
  };

  const handleEmojiClick = (emojiData: { emoji: string }) => {
    setEmoji(emojiData.emoji);
    setShowEmojiPicker(false);
  };

  return (
    <div className="relative" ref={dropdownRef}>
      {/* Status button */}
      <button
        onClick={() => setIsOpen(!isOpen)}
        className="flex items-center gap-2 rounded-md px-3 py-2 hover:bg-gray-800"
      >
        <span className="text-lg">{getCurrentStatusOption().icon}</span>
        <span className="text-sm text-gray-300">{getCurrentStatusOption().label}</span>
        {displayCustomMessage && (
          <>
            {displayEmoji && <span>{displayEmoji}</span>}
            <span className="text-sm text-gray-400 truncate max-w-32">{displayCustomMessage}</span>
          </>
        )}
        <svg className="h-4 w-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
        </svg>
      </button>

      {/* Dropdown */}
      {isOpen && (
        <div className="absolute bottom-full left-0 z-50 mb-2 w-96 rounded-lg border border-gray-700 bg-gray-900 shadow-lg">
          <div className="p-4">
            <h3 className="mb-3 font-semibold text-white">Set your status</h3>

            {/* Status options */}
            <div className="mb-4 space-y-1">
              {STATUS_OPTIONS.map((option) => (
                <button
                  key={option.value}
                  onClick={() => setSelectedStatus(option.value)}
                  className={`w-full rounded-md px-3 py-2 text-left transition-colors ${
                    selectedStatus === option.value
                      ? 'bg-blue-900 text-white'
                      : 'hover:bg-gray-800 text-gray-300'
                  }`}
                >
                  <div className="flex items-center gap-3">
                    <span className="text-lg">{option.icon}</span>
                    <div className="flex-1">
                      <div className="font-medium">{option.label}</div>
                      <div className="text-xs text-gray-400">{option.description}</div>
                    </div>
                    {selectedStatus === option.value && (
                      <svg className="h-5 w-5 text-blue-400" fill="currentColor" viewBox="0 0 20 20">
                        <path
                          fillRule="evenodd"
                          d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
                          clipRule="evenodd"
                        />
                      </svg>
                    )}
                  </div>
                </button>
              ))}
            </div>

            {/* Custom status message */}
            <div className="mb-4">
              <label className="mb-2 block text-sm font-medium text-gray-300">
                Custom message (optional)
              </label>
              <div className="flex gap-2">
                <div className="relative">
                  <button
                    onClick={() => setShowEmojiPicker(!showEmojiPicker)}
                    className="flex h-10 w-10 items-center justify-center rounded border border-gray-600 bg-gray-800 text-xl hover:border-gray-500"
                  >
                    {emoji || '😀'}
                  </button>
                  {showEmojiPicker && (
                    <div ref={emojiPickerRef} className="absolute left-0 top-12 z-50">
                      <EmojiPicker
                        onEmojiClick={handleEmojiClick}
                        theme={Theme.DARK}
                        width={300}
                        height={350}
                      />
                    </div>
                  )}
                </div>
                <input
                  type="text"
                  value={customMessage}
                  onChange={(e) => setCustomMessage(e.target.value)}
                  placeholder="What's your status?"
                  className="flex-1 rounded border border-gray-600 bg-gray-800 px-3 py-2 text-sm text-white placeholder-gray-500 focus:border-blue-500 focus:outline-none"
                  maxLength={80}
                />
              </div>
            </div>

            {/* Clear after */}
            <div className="mb-4">
              <label className="mb-2 block text-sm font-medium text-gray-300">
                Clear status after
              </label>
              <select
                value={clearAfter === null ? 'null' : clearAfter}
                onChange={(e) => setClearAfter(e.target.value === 'null' ? null : parseInt(e.target.value))}
                className="w-full rounded border border-gray-600 bg-gray-800 px-3 py-2 text-sm text-white focus:border-blue-500 focus:outline-none"
              >
                {TIME_OPTIONS.map((option) => (
                  <option key={option.value === null ? 'null' : option.value} value={option.value === null ? 'null' : option.value}>
                    {option.label}
                  </option>
                ))}
              </select>
            </div>

            {/* Back at - only show for Away or DND status */}
            {(selectedStatus === 'away' || selectedStatus === 'dnd') && (
              <div className="mb-4">
                <label className="mb-2 block text-sm font-medium text-gray-300">
                  Back at (optional)
                </label>
                <input
                  type="datetime-local"
                  value={backAt}
                  onChange={(e) => setBackAt(e.target.value)}
                  min={new Date().toISOString().slice(0, 16)}
                  className="w-full rounded border border-gray-600 bg-gray-800 px-3 py-2 text-sm text-white focus:border-blue-500 focus:outline-none [color-scheme:dark]"
                />
                {backAt && (
                  <button
                    onClick={() => setBackAt('')}
                    className="mt-1 text-xs text-gray-400 hover:text-gray-300"
                  >
                    Clear
                  </button>
                )}
              </div>
            )}

            {/* Actions */}
            <div className="flex gap-2">
              <button
                onClick={handleSave}
                disabled={saving}
                className="flex-1 rounded-md bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700 disabled:opacity-50"
              >
                {saving ? 'Saving...' : 'Save'}
              </button>
              <button
                onClick={() => setIsOpen(false)}
                className="rounded-md border border-gray-600 px-4 py-2 text-sm font-medium text-gray-300 hover:bg-gray-800"
              >
                Cancel
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
