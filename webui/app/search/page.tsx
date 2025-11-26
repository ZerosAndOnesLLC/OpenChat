'use client';

import { useState, useEffect, Suspense } from 'react';
import { useSearchParams, useRouter } from 'next/navigation';
import { apiClient } from '@/lib/api';
import SearchBar from '@/components/SearchBar';

interface Message {
  id: string;
  user_id: string;
  channel_id?: string;
  dm_id?: string;
  content: string;
  created_at: string;
  parent_message_id?: string;
}

function SearchContent() {
  const searchParams = useSearchParams();
  const router = useRouter();
  const [messages, setMessages] = useState<Message[]>([]);
  const [totalCount, setTotalCount] = useState(0);
  const [loading, setLoading] = useState(false);
  const [scope, setScope] = useState<'all' | 'channel' | 'dm'>('all');

  const query = searchParams.get('q') || '';

  useEffect(() => {
    if (query) {
      performSearch(query);
    }
  }, [query, scope]);

  const performSearch = async (searchQuery: string) => {
    setLoading(true);
    try {
      const data = await apiClient.searchMessages(searchQuery, scope, undefined, undefined, 50);
      setMessages(data.messages);
      setTotalCount(data.total_count);
    } catch (error) {
      console.error('Search failed:', error);
      setMessages([]);
      setTotalCount(0);
    } finally {
      setLoading(false);
    }
  };

  const handleSearch = (newQuery: string) => {
    router.push(`/search?q=${encodeURIComponent(newQuery)}`);
  };

  const formatDate = (timestamp: string) => {
    const date = new Date(timestamp);
    return date.toLocaleString();
  };

  const highlightQuery = (text: string, query: string) => {
    if (!query) return text;

    const queryTerms = query.split(' ').filter(term =>
      !term.startsWith('from:') &&
      !term.startsWith('in:') &&
      !term.startsWith('before:') &&
      !term.startsWith('after:')
    );

    let highlightedText = text;
    queryTerms.forEach(term => {
      const regex = new RegExp(`(${term})`, 'gi');
      highlightedText = highlightedText.replace(regex, '<mark class="bg-yellow-200 dark:bg-yellow-800">$1</mark>');
    });

    return highlightedText;
  };

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900">
      <div className="max-w-4xl mx-auto p-6">
        <div className="mb-6">
          <h1 className="text-2xl font-bold mb-4 text-gray-900 dark:text-gray-100">Search Messages</h1>
          <SearchBar onSearch={handleSearch} placeholder="Search messages..." autoFocus />
        </div>

        {query && (
          <div className="mb-4 flex items-center gap-4">
            <div className="flex gap-2">
              <button
                onClick={() => setScope('all')}
                className={`px-3 py-1 rounded text-sm ${
                  scope === 'all'
                    ? 'bg-blue-500 text-white'
                    : 'bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300'
                }`}
              >
                All
              </button>
              <button
                onClick={() => setScope('channel')}
                className={`px-3 py-1 rounded text-sm ${
                  scope === 'channel'
                    ? 'bg-blue-500 text-white'
                    : 'bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300'
                }`}
              >
                Channels
              </button>
              <button
                onClick={() => setScope('dm')}
                className={`px-3 py-1 rounded text-sm ${
                  scope === 'dm'
                    ? 'bg-blue-500 text-white'
                    : 'bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300'
                }`}
              >
                Direct Messages
              </button>
            </div>
            <div className="text-sm text-gray-600 dark:text-gray-400">
              {loading ? 'Searching...' : `${totalCount} result${totalCount !== 1 ? 's' : ''}`}
            </div>
          </div>
        )}

        <div className="space-y-4">
          {loading ? (
            <div className="flex items-center justify-center py-12">
              <div className="text-gray-500">Searching...</div>
            </div>
          ) : messages.length === 0 ? (
            <div className="flex items-center justify-center py-12">
              <div className="text-center">
                <p className="text-gray-500 mb-2">
                  {query ? 'No results found' : 'Enter a search query to find messages'}
                </p>
                {query && (
                  <p className="text-sm text-gray-400">
                    Try different keywords or use filters like from:@user or in:#channel
                  </p>
                )}
              </div>
            </div>
          ) : (
            messages.map(message => (
              <div
                key={message.id}
                className="bg-white dark:bg-gray-800 rounded-lg p-4 shadow hover:shadow-md transition-shadow cursor-pointer"
                onClick={() => {
                  if (message.channel_id) {
                    router.push(`/?channel=${message.channel_id}#${message.id}`);
                  } else if (message.dm_id) {
                    router.push(`/?dm=${message.dm_id}#${message.id}`);
                  }
                }}
              >
                <div className="flex items-start justify-between mb-2">
                  <div className="font-medium text-gray-900 dark:text-gray-100">
                    User {message.user_id.substring(0, 8)}
                  </div>
                  <div className="text-xs text-gray-500 dark:text-gray-400">
                    {formatDate(message.created_at)}
                  </div>
                </div>
                <div
                  className="text-gray-700 dark:text-gray-300 text-sm"
                  dangerouslySetInnerHTML={{ __html: highlightQuery(message.content, query) }}
                />
                <div className="mt-2 text-xs text-gray-500 dark:text-gray-400">
                  {message.channel_id ? '📢 Channel' : '💬 Direct Message'}
                </div>
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
}

export default function SearchPage() {
  return (
    <Suspense fallback={
      <div className="min-h-screen bg-gray-50 dark:bg-gray-900 flex items-center justify-center">
        <div className="text-gray-500">Loading search...</div>
      </div>
    }>
      <SearchContent />
    </Suspense>
  );
}
