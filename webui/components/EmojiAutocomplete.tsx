'use client';

import { useState, useEffect, useMemo, useCallback } from 'react';
import Image from 'next/image';
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
  const [selectedIndex, setSelectedIndex] = useState(0);
  // Set when the user dismisses (Escape) or picks a suggestion; cleared as
  // soon as the text or cursor moves again.
  const [dismissed, setDismissed] = useState(false);

  useEffect(() => {
    let cancelled = false;
    apiClient.getCustomEmojis()
      .then((emojis) => {
        if (!cancelled) setCustomEmojis(emojis);
      })
      .catch((error) => console.error('Failed to load custom emojis:', error));
    return () => {
      cancelled = true;
    };
  }, []);

  // Re-open the list and reset the highlight whenever the text or cursor
  // changes (adjusted during render rather than in an effect).
  const [prevInput, setPrevInput] = useState({ textValue, cursorPosition });
  if (prevInput.textValue !== textValue || prevInput.cursorPosition !== cursorPosition) {
    setPrevInput({ textValue, cursorPosition });
    setSelectedIndex(0);
    setDismissed(false);
  }

  const emojiMatch = textValue.substring(0, cursorPosition).match(/:([a-zA-Z0-9_-]*)$/);
  const emojiQuery = emojiMatch ? emojiMatch[1] : null;
  const showSuggestions = emojiQuery !== null && !dismissed;

  const filteredEmojis = useMemo(() => {
    if (emojiQuery === null) return [];
    const query = emojiQuery.toLowerCase();
    return customEmojis.filter((emoji) => emoji.name.toLowerCase().includes(query)).slice(0, 10);
  }, [customEmojis, emojiQuery]);

  const visible = showSuggestions && filteredEmojis.length > 0;

  const handleSelectEmoji = useCallback((emojiName: string) => {
    onSelectEmoji(`:${emojiName}:`);
    setDismissed(true);
  }, [onSelectEmoji]);

  useEffect(() => {
    if (!visible) return;

    const handleKeyDown = (e: KeyboardEvent) => {
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
          setDismissed(true);
          break;
      }
    };

    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [visible, selectedIndex, filteredEmojis, handleSelectEmoji]);

  if (!visible) {
    return null;
  }

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
            <Image
              src={apiClient.getEmojiImage(emoji.id)}
              alt={emoji.name}
              width={24}
              height={24}
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
