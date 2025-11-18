'use client';

import { useState, useEffect } from 'react';
import dynamic from 'next/dynamic';
import { CustomEmoji } from '@/lib/types';
import { apiClient } from '@/lib/api';
import { Theme } from 'emoji-picker-react';

// Dynamically import EmojiPicker to avoid SSR issues
const EmojiPicker = dynamic(() => import('emoji-picker-react'), { ssr: false });

interface CustomEmojiPickerProps {
  onEmojiClick: (emoji: string) => void;
  onClose: () => void;
}

export default function CustomEmojiPicker({ onEmojiClick, onClose }: CustomEmojiPickerProps) {
  const [customEmojis, setCustomEmojis] = useState<CustomEmoji[]>([]);
  const [selectedTab, setSelectedTab] = useState<'standard' | 'custom'>('standard');
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadCustomEmojis();
  }, []);

  const loadCustomEmojis = async () => {
    try {
      const emojis = await apiClient.getCustomEmojis();
      setCustomEmojis(emojis);
    } catch (error) {
      console.error('Failed to load custom emojis:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleStandardEmojiClick = (emojiData: any) => {
    onEmojiClick(emojiData.emoji);
    onClose();
  };

  const handleCustomEmojiClick = (emojiName: string) => {
    onEmojiClick(`:${emojiName}:`);
    onClose();
  };

  return (
    <div className="absolute bottom-full mb-2 right-0 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-lg shadow-lg z-50">
      {/* Tabs */}
      <div className="flex border-b border-gray-300 dark:border-gray-600">
        <button
          onClick={() => setSelectedTab('standard')}
          className={`flex-1 px-4 py-2 text-sm font-medium ${
            selectedTab === 'standard'
              ? 'text-blue-600 border-b-2 border-blue-600'
              : 'text-gray-600 dark:text-gray-400 hover:text-gray-800 dark:hover:text-gray-200'
          }`}
        >
          Standard Emojis
        </button>
        <button
          onClick={() => setSelectedTab('custom')}
          className={`flex-1 px-4 py-2 text-sm font-medium ${
            selectedTab === 'custom'
              ? 'text-blue-600 border-b-2 border-blue-600'
              : 'text-gray-600 dark:text-gray-400 hover:text-gray-800 dark:hover:text-gray-200'
          }`}
        >
          Custom Emojis {customEmojis.length > 0 && `(${customEmojis.length})`}
        </button>
      </div>

      {/* Content */}
      <div>
        {selectedTab === 'standard' ? (
          <EmojiPicker
            onEmojiClick={handleStandardEmojiClick}
            theme={Theme.AUTO}
            height={350}
            width={320}
          />
        ) : (
          <div className="p-4 w-80 h-96 overflow-y-auto">
            {loading ? (
              <div className="flex items-center justify-center h-full">
                <div className="text-gray-500 dark:text-gray-400">Loading custom emojis...</div>
              </div>
            ) : customEmojis.length === 0 ? (
              <div className="flex items-center justify-center h-full">
                <div className="text-center text-gray-500 dark:text-gray-400">
                  <p className="mb-2">No custom emojis yet</p>
                  <p className="text-sm">Ask your admin to upload some!</p>
                </div>
              </div>
            ) : (
              <div className="grid grid-cols-6 gap-2">
                {customEmojis.map((emoji) => (
                  <button
                    key={emoji.id}
                    onClick={() => handleCustomEmojiClick(emoji.name)}
                    className="w-12 h-12 flex items-center justify-center hover:bg-gray-100 dark:hover:bg-gray-700 rounded transition-colors"
                    title={`:${emoji.name}:`}
                  >
                    <img
                      src={apiClient.getEmojiImage(emoji.id)}
                      alt={emoji.name}
                      className="w-8 h-8 object-contain"
                    />
                  </button>
                ))}
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
