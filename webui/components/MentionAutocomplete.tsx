'use client';

import { useState, useEffect, useRef } from 'react';
import { apiClient } from '@/lib/api';
import type { User, UserGroup } from '@/lib/types';

// Helper to get username from email (before @)
const getUsernameFromEmail = (email: string): string => {
  return email.split('@')[0];
};

interface MentionAutocompleteProps {
  value: string;
  onChange: (value: string) => void;
  onSelectMention?: (username: string) => void;
  placeholder?: string;
  className?: string;
}

export default function MentionAutocomplete({
  value,
  onChange,
  onSelectMention,
  placeholder,
  className
}: MentionAutocompleteProps) {
  const [users, setUsers] = useState<User[]>([]);
  const [groups, setGroups] = useState<UserGroup[]>([]);
  const [showSuggestions, setShowSuggestions] = useState(false);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [mentionQuery, setMentionQuery] = useState('');
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    loadUsers();
    loadGroups();
  }, []);

  const loadUsers = async () => {
    try {
      const data = await apiClient.listUsers();
      setUsers(data);
    } catch (error) {
      console.error('Failed to load users:', error);
    }
  };

  const loadGroups = async () => {
    try {
      const data = await apiClient.listUserGroups();
      setGroups(data);
    } catch (error) {
      console.error('Failed to load groups:', error);
    }
  };

  useEffect(() => {
    const cursorPos = textareaRef.current?.selectionStart || 0;
    const textBeforeCursor = value.substring(0, cursorPos);
    const mentionMatch = textBeforeCursor.match(/@([\w-]*)$/);

    if (mentionMatch) {
      const query = mentionMatch[1];
      setMentionQuery(query);
      setShowSuggestions(true);
      setSelectedIndex(0);
    } else {
      setShowSuggestions(false);
    }
  }, [value]);

  const filteredUsers = users.filter(user => {
    const username = getUsernameFromEmail(user.email);
    return username.toLowerCase().includes(mentionQuery.toLowerCase()) ||
      user.display_name.toLowerCase().includes(mentionQuery.toLowerCase());
  }).slice(0, 5);

  const filteredGroups = groups.filter(group =>
    group.handle.toLowerCase().includes(mentionQuery.toLowerCase()) ||
    group.name.toLowerCase().includes(mentionQuery.toLowerCase())
  ).slice(0, 3);

  const specialMentions = ['@channel', '@here'].filter(m =>
    m.includes(mentionQuery.toLowerCase())
  );

  const allSuggestions = [
    ...specialMentions,
    ...filteredGroups.map(g => `@${g.handle}`),
    ...filteredUsers.map(u => `@${getUsernameFromEmail(u.email)}`),
  ];

  const insertMention = (mention: string) => {
    const cursorPos = textareaRef.current?.selectionStart || 0;
    const textBeforeCursor = value.substring(0, cursorPos);
    const textAfterCursor = value.substring(cursorPos);

    const mentionStart = textBeforeCursor.lastIndexOf('@');
    const newText = textBeforeCursor.substring(0, mentionStart) + mention + ' ' + textAfterCursor;

    onChange(newText);
    setShowSuggestions(false);

    if (onSelectMention) {
      onSelectMention(mention);
    }

    setTimeout(() => {
      const newCursorPos = mentionStart + mention.length + 1;
      textareaRef.current?.setSelectionRange(newCursorPos, newCursorPos);
      textareaRef.current?.focus();
    }, 0);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (!showSuggestions || allSuggestions.length === 0) return;

    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault();
        setSelectedIndex((prev) => (prev + 1) % allSuggestions.length);
        break;
      case 'ArrowUp':
        e.preventDefault();
        setSelectedIndex((prev) => (prev - 1 + allSuggestions.length) % allSuggestions.length);
        break;
      case 'Enter':
      case 'Tab':
        if (allSuggestions[selectedIndex]) {
          e.preventDefault();
          insertMention(allSuggestions[selectedIndex]);
        }
        break;
      case 'Escape':
        setShowSuggestions(false);
        break;
    }
  };

  return (
    <div className="relative">
      <textarea
        ref={textareaRef}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        onKeyDown={handleKeyDown}
        placeholder={placeholder}
        className={className}
        rows={3}
      />

      {showSuggestions && allSuggestions.length > 0 && (
        <div className="absolute bottom-full left-0 mb-2 w-64 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg shadow-lg max-h-48 overflow-y-auto z-10">
          {specialMentions.map((mention, index) => (
            <div
              key={mention}
              onClick={() => insertMention(mention)}
              className={`px-4 py-2 cursor-pointer flex items-center gap-2 ${
                selectedIndex === index
                  ? 'bg-blue-500 text-white'
                  : 'hover:bg-gray-100 dark:hover:bg-gray-700'
              }`}
            >
              <span className="text-xl">📢</span>
              <div>
                <div className="font-medium">{mention}</div>
                <div className="text-xs opacity-75">Notify all members</div>
              </div>
            </div>
          ))}
          {filteredGroups.map((group, index) => {
            const adjustedIndex = index + specialMentions.length;
            return (
              <div
                key={group.id}
                onClick={() => insertMention(`@${group.handle}`)}
                className={`px-4 py-2 cursor-pointer flex items-center gap-2 ${
                  selectedIndex === adjustedIndex
                    ? 'bg-blue-500 text-white'
                    : 'hover:bg-gray-100 dark:hover:bg-gray-700'
                }`}
              >
                <div className="w-8 h-8 rounded-lg bg-purple-600/20 flex items-center justify-center">
                  <svg className="w-4 h-4 text-purple-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0z" />
                  </svg>
                </div>
                <div>
                  <div className="font-medium">{group.name}</div>
                  <div className="text-xs opacity-75">@{group.handle}</div>
                </div>
              </div>
            );
          })}
          {filteredUsers.map((user, index) => {
            const adjustedIndex = index + specialMentions.length + filteredGroups.length;
            const username = getUsernameFromEmail(user.email);
            return (
              <div
                key={user.id}
                onClick={() => insertMention(`@${username}`)}
                className={`px-4 py-2 cursor-pointer flex items-center gap-2 ${
                  selectedIndex === adjustedIndex
                    ? 'bg-blue-500 text-white'
                    : 'hover:bg-gray-100 dark:hover:bg-gray-700'
                }`}
              >
                <div className="w-8 h-8 rounded-full bg-gray-300 dark:bg-gray-600 flex items-center justify-center text-sm font-semibold">
                  {user.display_name.charAt(0).toUpperCase()}
                </div>
                <div>
                  <div className="font-medium">{user.display_name}</div>
                  <div className="text-xs opacity-75">@{username}</div>
                </div>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
