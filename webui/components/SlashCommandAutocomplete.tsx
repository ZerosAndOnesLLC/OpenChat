'use client';

import { useState, useEffect } from 'react';
import { apiClient } from '@/lib/api';
import type { SlashCommand } from '@/lib/types';

interface SlashCommandAutocompleteProps {
  cursorPosition: number;
  textValue: string;
  onSelect: (commandText: string) => void;
}

export default function SlashCommandAutocomplete({
  cursorPosition,
  textValue,
  onSelect,
}: SlashCommandAutocompleteProps) {
  const [commands, setCommands] = useState<SlashCommand[]>([]);
  const [showSuggestions, setShowSuggestions] = useState(false);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [query, setQuery] = useState('');

  useEffect(() => {
    loadCommands();
  }, []);

  const loadCommands = async () => {
    try {
      const cmds = await apiClient.listCommands();
      setCommands(cmds);
    } catch (error) {
      console.error('Failed to load commands:', error);
    }
  };

  useEffect(() => {
    // Only trigger at the start of input
    const match = textValue.match(/^\/(\w*)$/);
    if (match) {
      setQuery(match[1]);
      setShowSuggestions(true);
      setSelectedIndex(0);
    } else {
      setShowSuggestions(false);
    }
  }, [textValue, cursorPosition]);

  const filtered = commands
    .filter((cmd) => cmd.name.toLowerCase().startsWith(query.toLowerCase()))
    .slice(0, 10);

  useEffect(() => {
    if (!showSuggestions || filtered.length === 0) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      switch (e.key) {
        case 'ArrowDown':
          e.preventDefault();
          e.stopPropagation();
          setSelectedIndex((prev) => (prev + 1) % filtered.length);
          break;
        case 'ArrowUp':
          e.preventDefault();
          e.stopPropagation();
          setSelectedIndex((prev) => (prev - 1 + filtered.length) % filtered.length);
          break;
        case 'Enter':
        case 'Tab':
          if (filtered[selectedIndex]) {
            e.preventDefault();
            e.stopPropagation();
            onSelect(`/${filtered[selectedIndex].name} `);
            setShowSuggestions(false);
          }
          break;
        case 'Escape':
          setShowSuggestions(false);
          break;
      }
    };

    document.addEventListener('keydown', handleKeyDown, true);
    return () => document.removeEventListener('keydown', handleKeyDown, true);
  }, [showSuggestions, selectedIndex, filtered, onSelect]);

  if (!showSuggestions || filtered.length === 0) {
    return null;
  }

  return (
    <div className="absolute bottom-full mb-2 left-0 bg-gray-800 border border-gray-600 rounded-lg shadow-lg w-80 z-50">
      <div className="p-2 border-b border-gray-700">
        <p className="text-xs text-gray-400">Slash Commands</p>
      </div>
      <div className="max-h-48 overflow-y-auto">
        {filtered.map((cmd, index) => (
          <button
            key={cmd.name}
            onClick={() => {
              onSelect(`/${cmd.name} `);
              setShowSuggestions(false);
            }}
            className={`w-full flex items-start px-3 py-2 hover:bg-gray-700 transition-colors ${
              index === selectedIndex ? 'bg-blue-900/30' : ''
            }`}
          >
            <div className="flex-1 text-left">
              <p className="text-sm font-medium text-white">/{cmd.name}</p>
              <p className="text-xs text-gray-400">{cmd.description}</p>
            </div>
          </button>
        ))}
      </div>
    </div>
  );
}
