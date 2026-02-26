'use client';

import { useEffect, useState } from 'react';
import { apiClient } from '@/lib/api';
import type { UserGroup } from '@/lib/types';

interface MentionHighlightProps {
  content: string;
  currentUserId?: string;
}

let cachedGroupHandles: Set<string> | null = null;

export default function MentionHighlight({ content, currentUserId }: MentionHighlightProps) {
  const [groupHandles, setGroupHandles] = useState<Set<string>>(cachedGroupHandles || new Set());

  useEffect(() => {
    if (!cachedGroupHandles) {
      apiClient.listUserGroups().then((groups: UserGroup[]) => {
        const handles = new Set(groups.map(g => g.handle.toLowerCase()));
        cachedGroupHandles = handles;
        setGroupHandles(handles);
      }).catch(() => {});
    }
  }, []);

  const highlightMentions = (text: string) => {
    const mentionRegex = /(@[\w-]+|@channel|@here)/g;
    const parts = text.split(mentionRegex);

    return parts.map((part, index) => {
      if (part.match(mentionRegex)) {
        const handle = part.slice(1).toLowerCase();
        const isCurrentUser = currentUserId && part === `@${currentUserId}`;
        const isChannel = part === '@channel' || part === '@here';
        const isGroup = groupHandles.has(handle);

        return (
          <span
            key={index}
            className={`font-semibold rounded px-1 ${
              isCurrentUser
                ? 'bg-yellow-200 dark:bg-yellow-800 text-yellow-900 dark:text-yellow-100'
                : isChannel
                ? 'bg-blue-200 dark:bg-blue-800 text-blue-900 dark:text-blue-100'
                : isGroup
                ? 'bg-purple-200 dark:bg-purple-800 text-purple-900 dark:text-purple-100'
                : 'bg-gray-200 dark:bg-gray-700 text-blue-600 dark:text-blue-400'
            }`}
          >
            {part}
          </span>
        );
      }
      return part;
    });
  };

  return <>{highlightMentions(content)}</>;
}
