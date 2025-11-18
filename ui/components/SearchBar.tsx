'use client';

import { useState, useRef, useEffect } from 'react';
import { useRouter } from 'next/navigation';

interface SearchBarProps {
  onSearch?: (query: string) => void;
  placeholder?: string;
  autoFocus?: boolean;
  onClose?: () => void;
}

export default function SearchBar({ onSearch, placeholder = 'Search messages...', autoFocus = false, onClose }: SearchBarProps) {
  const [query, setQuery] = useState('');
  const [showHelp, setShowHelp] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);
  const router = useRouter();

  useEffect(() => {
    if (autoFocus && inputRef.current) {
      inputRef.current.focus();
    }
  }, [autoFocus]);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (query.trim()) {
      if (onSearch) {
        onSearch(query);
      } else {
        router.push(`/search?q=${encodeURIComponent(query)}`);
      }
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Escape' && onClose) {
      onClose();
    }
  };

  return (
    <div className="relative">
      <form onSubmit={handleSubmit} className="relative">
        <div className="relative">
          <input
            ref={inputRef}
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={handleKeyDown}
            onFocus={() => setShowHelp(true)}
            onBlur={() => setTimeout(() => setShowHelp(false), 200)}
            placeholder={placeholder}
            className="w-full px-4 py-2 pl-10 pr-10 text-sm border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
          />
          <div className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400">
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
            </svg>
          </div>
          {query && (
            <button
              type="button"
              onClick={() => setQuery('')}
              className="absolute right-3 top-1/2 transform -translate-y-1/2 text-gray-400 hover:text-gray-600"
            >
              <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          )}
        </div>
      </form>

      {showHelp && (
        <div className="absolute z-50 w-full mt-2 p-4 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg shadow-lg">
          <h3 className="text-sm font-semibold mb-2 text-gray-900 dark:text-gray-100">Search Filters</h3>
          <div className="space-y-2 text-xs text-gray-600 dark:text-gray-400">
            <div><code className="bg-gray-100 dark:bg-gray-700 px-1 py-0.5 rounded">from:@username</code> - Filter by user</div>
            <div><code className="bg-gray-100 dark:bg-gray-700 px-1 py-0.5 rounded">in:#channel</code> - Filter by channel</div>
            <div><code className="bg-gray-100 dark:bg-gray-700 px-1 py-0.5 rounded">before:2025-01-15</code> - Before date</div>
            <div><code className="bg-gray-100 dark:bg-gray-700 px-1 py-0.5 rounded">after:2025-01-01</code> - After date</div>
            <div className="pt-2 border-t border-gray-200 dark:border-gray-700">
              Combine filters: <code className="bg-gray-100 dark:bg-gray-700 px-1 py-0.5 rounded">from:@john in:#general hello</code>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
