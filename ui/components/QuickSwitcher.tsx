'use client';

import { useState, useEffect, useRef } from 'react';
import { useQuery } from '@tanstack/react-query';
import { apiClient } from '@/lib/api';
import type { Channel, DirectMessage } from '@/lib/types';

interface QuickSwitcherProps {
  isOpen: boolean;
  onClose: () => void;
  onSelectChannel: (channel: Channel) => void;
  onSelectDm: (dm: DirectMessage) => void;
}

export default function QuickSwitcher({ isOpen, onClose, onSelectChannel, onSelectDm }: QuickSwitcherProps) {
  const [search, setSearch] = useState('');
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  // Fetch channels and DMs
  const { data: channels = [] } = useQuery({
    queryKey: ['channels'],
    queryFn: apiClient.listChannels,
  });

  const { data: dms = [] } = useQuery({
    queryKey: ['dms'],
    queryFn: apiClient.listDms,
  });

  // Filter channels and DMs based on search
  const filteredChannels = channels.filter((channel) =>
    channel.name.toLowerCase().includes(search.toLowerCase())
  );

  const filteredDms = dms.filter((dm) => {
    const otherUser = dm.participants?.find((p) => p.id !== dm.id);
    const userName = otherUser?.display_name || '';
    return userName.toLowerCase().includes(search.toLowerCase());
  });

  const allItems = [
    ...filteredChannels.map((c) => ({ type: 'channel' as const, item: c })),
    ...filteredDms.map((d) => ({ type: 'dm' as const, item: d })),
  ];

  // Reset search and selection when opened
  useEffect(() => {
    if (isOpen) {
      setSearch('');
      setSelectedIndex(0);
      setTimeout(() => inputRef.current?.focus(), 0);
    }
  }, [isOpen]);

  // Handle keyboard navigation
  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      setSelectedIndex((prev) => (prev + 1) % allItems.length);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      setSelectedIndex((prev) => (prev - 1 + allItems.length) % allItems.length);
    } else if (e.key === 'Enter' && allItems[selectedIndex]) {
      e.preventDefault();
      handleSelect(allItems[selectedIndex]);
    } else if (e.key === 'Escape') {
      e.preventDefault();
      onClose();
    }
  };

  const handleSelect = (item: typeof allItems[0]) => {
    if (item.type === 'channel') {
      onSelectChannel(item.item);
    } else {
      onSelectDm(item.item);
    }
  };

  if (!isOpen) return null;

  return (
    <div
      className="fixed inset-0 z-50 flex items-start justify-center bg-black bg-opacity-50 pt-20"
      onClick={onClose}
    >
      <div
        className="w-full max-w-2xl overflow-hidden rounded-lg bg-gray-900 shadow-xl"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Search Input */}
        <div className="border-b border-gray-700 p-4">
          <input
            ref={inputRef}
            type="text"
            value={search}
            onChange={(e) => {
              setSearch(e.target.value);
              setSelectedIndex(0);
            }}
            onKeyDown={handleKeyDown}
            placeholder="Search channels and direct messages..."
            className="w-full bg-transparent text-lg text-white placeholder-gray-400 focus:outline-none"
          />
        </div>

        {/* Results */}
        <div className="max-h-[400px] overflow-y-auto">
          {allItems.length === 0 ? (
            <div className="px-4 py-8 text-center text-gray-400">
              No channels or DMs found
            </div>
          ) : (
            <div className="py-2">
              {allItems.map((item, index) => {
                const isChannel = item.type === 'channel';
                const name = isChannel
                  ? item.item.name
                  : item.item.participants?.find((p) => p.id !== item.item.id)?.display_name || 'Unknown';

                const isSelected = index === selectedIndex;

                return (
                  <button
                    key={`${item.type}-${item.item.id}`}
                    onClick={() => handleSelect(item)}
                    className={`flex w-full items-center gap-3 px-4 py-3 text-left transition-colors ${
                      isSelected ? 'bg-blue-600 text-white' : 'text-gray-300 hover:bg-gray-800'
                    }`}
                  >
                    {/* Icon */}
                    <div
                      className={`flex h-8 w-8 flex-shrink-0 items-center justify-center rounded ${
                        isSelected ? 'bg-blue-700' : 'bg-gray-700'
                      }`}
                    >
                      {isChannel ? (
                        <svg className="h-4 w-4" fill="currentColor" viewBox="0 0 20 20">
                          <path d="M2 5a2 2 0 012-2h7a2 2 0 012 2v4a2 2 0 01-2 2H9l-3 3v-3H4a2 2 0 01-2-2V5z" />
                          <path d="M15 7v2a4 4 0 01-4 4H9.828l-1.766 1.767c.28.149.599.233.938.233h2l3 3v-3h2a2 2 0 002-2V9a2 2 0 00-2-2h-1z" />
                        </svg>
                      ) : (
                        <svg className="h-4 w-4" fill="currentColor" viewBox="0 0 20 20">
                          <path fillRule="evenodd" d="M10 9a3 3 0 100-6 3 3 0 000 6zm-7 9a7 7 0 1114 0H3z" clipRule="evenodd" />
                        </svg>
                      )}
                    </div>

                    {/* Name and Type */}
                    <div className="flex-1">
                      <div className="font-medium">
                        {isChannel ? '#' : ''}{name}
                      </div>
                      <div className={`text-xs ${isSelected ? 'text-blue-200' : 'text-gray-500'}`}>
                        {isChannel ? 'Channel' : 'Direct Message'}
                      </div>
                    </div>
                  </button>
                );
              })}
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="border-t border-gray-700 px-4 py-2">
          <div className="flex items-center gap-4 text-xs text-gray-400">
            <span>
              <kbd className="rounded bg-gray-800 px-1.5 py-0.5 font-mono">↑↓</kbd> Navigate
            </span>
            <span>
              <kbd className="rounded bg-gray-800 px-1.5 py-0.5 font-mono">Enter</kbd> Select
            </span>
            <span>
              <kbd className="rounded bg-gray-800 px-1.5 py-0.5 font-mono">Esc</kbd> Close
            </span>
          </div>
        </div>
      </div>
    </div>
  );
}
