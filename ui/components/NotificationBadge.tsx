'use client';

import { useState, useEffect } from 'react';
import { apiClient } from '@/lib/api';
import NotificationsPanel from './NotificationsPanel';

export default function NotificationBadge() {
  const [count, setCount] = useState(0);
  const [isOpen, setIsOpen] = useState(false);

  useEffect(() => {
    loadCount();
    const interval = setInterval(loadCount, 30000); // Poll every 30 seconds
    return () => clearInterval(interval);
  }, []);

  const loadCount = async () => {
    try {
      const data = await apiClient.getUnreadNotificationCount();
      setCount(data.count);
    } catch (error) {
      console.error('Failed to load notification count:', error);
    }
  };

  const handleClick = () => {
    setIsOpen(!isOpen);
    if (!isOpen && count > 0) {
      setTimeout(loadCount, 1000);
    }
  };

  return (
    <>
      <button
        onClick={handleClick}
        className="relative p-2 text-gray-600 hover:text-gray-900 dark:text-gray-400 dark:hover:text-gray-100 transition-colors"
        aria-label="Notifications"
      >
        <svg
          className="w-6 h-6"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9"
          />
        </svg>
        {count > 0 && (
          <span className="absolute top-0 right-0 inline-flex items-center justify-center px-2 py-1 text-xs font-bold leading-none text-white transform translate-x-1/2 -translate-y-1/2 bg-red-500 rounded-full min-w-[20px]">
            {count > 99 ? '99+' : count}
          </span>
        )}
      </button>

      <NotificationsPanel isOpen={isOpen} onClose={() => setIsOpen(false)} />
    </>
  );
}
