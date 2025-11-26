'use client';

import React, { useEffect, useState } from 'react';
import { CustomEmoji } from '@/lib/types';
import { apiClient } from '@/lib/api';

interface CustomEmojiRendererProps {
  content: string;
}

export default function CustomEmojiRenderer({ content }: CustomEmojiRendererProps) {
  const [customEmojis, setCustomEmojis] = useState<CustomEmoji[]>([]);
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

  const renderContentWithCustomEmojis = (text: string) => {
    if (loading || customEmojis.length === 0) {
      return text;
    }

    // Replace :emoji_name: with custom emoji images
    const parts: (string | React.ReactElement)[] = [];
    let lastIndex = 0;
    const emojiRegex = /:([a-zA-Z0-9_-]+):/g;
    let match;

    while ((match = emojiRegex.exec(text)) !== null) {
      const [fullMatch, emojiName] = match;
      const matchIndex = match.index;

      // Add text before the emoji
      if (matchIndex > lastIndex) {
        parts.push(text.substring(lastIndex, matchIndex));
      }

      // Find matching custom emoji
      const customEmoji = customEmojis.find((e) => e.name === emojiName);

      if (customEmoji) {
        // Render custom emoji image
        parts.push(
          <img
            key={`emoji-${matchIndex}`}
            src={apiClient.getEmojiImage(customEmoji.id)}
            alt={`:${emojiName}:`}
            title={`:${emojiName}:`}
            className="inline-block w-5 h-5 align-text-bottom mx-0.5"
          />
        );
      } else {
        // If no matching custom emoji, keep the original text
        parts.push(fullMatch);
      }

      lastIndex = emojiRegex.lastIndex;
    }

    // Add remaining text
    if (lastIndex < text.length) {
      parts.push(text.substring(lastIndex));
    }

    return parts;
  };

  const renderedContent = renderContentWithCustomEmojis(content);

  return <>{renderedContent}</>;
}
