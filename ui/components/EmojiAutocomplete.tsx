'use client';

import { useState, useEffect } from 'react';
import { apiClient } from '@/lib/api';
import type { CustomEmoji } from '@/lib/types';

interface EmojiAutocompleteProps {
  cursorPosition: number;
  textValue: string;
  onSelectEmoji: (emoji: string) => void;
}

export default function EmojiAutocomplete({
  cursorPosition,
  textValue,
  onSelectEmoji,
}: EmojiAutocompleteProps) {
  const [customEmojis, setCustomEmojis] = useState<CustomEmoji[]>([]);
  const [showSuggestions, setShowSuggestions] = useState(false);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [emojiQuery, setEmojiQuery] = useState('');
  const [position, setPosition] = useState({ top: 0, left: 0 });

  useEffect(() => {
    loadCustomEmojis();
  }, []);

  const loadCustomEmojis = async () => {
    try {
      const emojis = await apiClient.getCustomEmojis();
      setCustomEmojis(emojis);
    } catch (error) {
      console.error('Failed to load custom emojis:', error);
    }
  };

  useEffect(() => {
    const textBeforeCursor = textValue.substring(0, cursorPosition);
    const emojiMatch = textBeforeCursor.match(/:([a-zA-Z0-9_-]*)$/);

    if (emojiMatch) {
      const query = emojiMatch[1];
      setEmojiQuery(query);
      setShowSuggestions(true);
      setSelectedIndex(0);
    } else {
      setShowSuggestions(false);
    }
  }, [textValue, cursorPosition]);

  const filteredEmojis = customEmojis.filter((emoji) =>
    emoji.name.toLowerCase().includes(emojiQuery.toLowerCase())
  ).slice(0, 10);

  if (!showSuggestions || filteredEmojis.length === 0) {
    return null;
  }

  const handleSelectEmoji = (emojiName: string) => {
    onSelectEmoji(`:${emojiName}:`);
    setShowSuggestions(false);
  };

  const handleKeyDown = (e: KeyboardEvent) => {
    if (!showSuggestions || filteredEmojis.length === 0) return;

    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault();
        setSelectedIndex((prev) => (prev + 1) % filteredEmojis.length);
        break;
      case 'ArrowUp':
        e.preventDefault();
        setSelectedIndex((prev) => (prev - 1 + filteredEmojis.length) % filteredEmojis.length);
        break;
      case 'Enter':
      case 'Tab':
        if (filteredEmojis[selectedIndex]) {
          e.preventDefault();
          handleSelectEmoji(filteredEmojis[selectedIndex].name);
        }
        break;
      case 'Escape':
        setShowSuggestions(false);
        break;
    }
  };

  useEffect(() => {
    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [showSuggestions, selectedIndex, filteredEmojis]);

  return (
    <div className="absolute bottom-full mb-2 left-0 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-lg shadow-lg max-w-xs z-50">
      <div className="p-2 border-b border-gray-200 dark:border-gray-700">
        <p className="text-xs text-gray-500 dark:text-gray-400">Custom Emojis</p>
      </div>
      <div className="max-h-48 overflow-y-auto">
        {filteredEmojis.map((emoji, index) => (
          <button
            key={emoji.id}
            onClick={() => handleSelectEmoji(emoji.name)}
            className={`w-full flex items-center px-3 py-2 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors ${
              index === selectedIndex ? 'bg-blue-50 dark:bg-blue-900/30' : ''
            }`}
          >
            <img
              src={apiClient.getEmojiImage(emoji.id)}
              alt={emoji.name}
              className="w-6 h-6 mr-2 object-contain"
            />
            <div className="flex-1 text-left">
              <p className="text-sm font-medium text-gray-900 dark:text-white">
                :{emoji.name}:
              </p>
            </div>
          </button>
        ))}
      </div>
    </div>
  );
}
