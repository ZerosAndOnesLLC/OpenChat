'use client';

import { useState, useEffect, useCallback, useRef } from 'react';
import { useRouter } from 'next/navigation';
import Sidebar from './Sidebar';
import MessageArea from './MessageArea';
import KeyboardShortcutsHelp from './KeyboardShortcutsHelp';
import QuickSwitcher from './QuickSwitcher';
import { keyboardShortcutsManager, SHORTCUT_CATEGORIES } from '@/lib/keyboard-shortcuts';
import type { Channel, DirectMessage } from '@/lib/types';

const SIDEBAR_MIN_WIDTH = 180;
const SIDEBAR_MAX_WIDTH = 400;
const SIDEBAR_DEFAULT_WIDTH = 256;
const SIDEBAR_WIDTH_KEY = 'openchat-sidebar-width';

export default function ChatLayout() {
  const router = useRouter();
  const [activeChannel, setActiveChannel] = useState<Channel | null>(null);
  const [activeDm, setActiveDm] = useState<DirectMessage | null>(null);
  const [showShortcutsHelp, setShowShortcutsHelp] = useState(false);
  const [showQuickSwitcher, setShowQuickSwitcher] = useState(false);
  const [sidebarWidth, setSidebarWidth] = useState(SIDEBAR_DEFAULT_WIDTH);
  const [isResizing, setIsResizing] = useState(false);
  const resizeRef = useRef<{ startX: number; startWidth: number } | null>(null);

  const handleSelectChannel = (channel: Channel) => {
    setActiveChannel(channel);
    setActiveDm(null);
    setShowQuickSwitcher(false);
  };

  const handleSelectDm = (dm: DirectMessage) => {
    setActiveDm(dm);
    setActiveChannel(null);
    setShowQuickSwitcher(false);
  };

  const handleLeaveChannel = () => {
    setActiveChannel(null);
  };

  // Load sidebar width from localStorage
  useEffect(() => {
    const savedWidth = localStorage.getItem(SIDEBAR_WIDTH_KEY);
    if (savedWidth) {
      const width = parseInt(savedWidth, 10);
      if (width >= SIDEBAR_MIN_WIDTH && width <= SIDEBAR_MAX_WIDTH) {
        setSidebarWidth(width);
      }
    }
  }, []);

  // Handle resize start
  const handleResizeStart = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    setIsResizing(true);
    resizeRef.current = { startX: e.clientX, startWidth: sidebarWidth };
  }, [sidebarWidth]);

  // Handle resize move and end
  useEffect(() => {
    if (!isResizing) return;

    const handleMouseMove = (e: MouseEvent) => {
      if (!resizeRef.current) return;
      const delta = e.clientX - resizeRef.current.startX;
      const newWidth = Math.min(
        SIDEBAR_MAX_WIDTH,
        Math.max(SIDEBAR_MIN_WIDTH, resizeRef.current.startWidth + delta)
      );
      setSidebarWidth(newWidth);
    };

    const handleMouseUp = () => {
      setIsResizing(false);
      localStorage.setItem(SIDEBAR_WIDTH_KEY, sidebarWidth.toString());
      resizeRef.current = null;
    };

    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);

    return () => {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    };
  }, [isResizing, sidebarWidth]);

  // Register global keyboard shortcuts
  useEffect(() => {
    // Cmd/Ctrl+K: Quick switcher
    const unregisterQuickSwitcher = keyboardShortcutsManager.register({
      key: 'k',
      ctrl: true,
      meta: true,
      description: 'Quick switcher (channels/DMs)',
      category: SHORTCUT_CATEGORIES.NAVIGATION,
      handler: () => setShowQuickSwitcher(true),
    });

    // Cmd/Ctrl+/: Show keyboard shortcuts help
    const unregisterHelp = keyboardShortcutsManager.register({
      key: '/',
      ctrl: true,
      meta: true,
      description: 'Show keyboard shortcuts',
      category: SHORTCUT_CATEGORIES.GENERAL,
      handler: () => setShowShortcutsHelp(true),
    });

    // Cmd/Ctrl+F: Search messages
    const unregisterSearch = keyboardShortcutsManager.register({
      key: 'f',
      ctrl: true,
      meta: true,
      description: 'Search messages',
      category: SHORTCUT_CATEGORIES.NAVIGATION,
      handler: () => router.push('/search'),
    });

    // Escape: Close modals/panels
    const unregisterEscape = keyboardShortcutsManager.register({
      key: 'escape',
      description: 'Close modals and panels',
      category: SHORTCUT_CATEGORIES.GENERAL,
      handler: () => {
        setShowShortcutsHelp(false);
        setShowQuickSwitcher(false);
      },
    });

    // Global keyboard event listener
    const handleKeyDown = (e: KeyboardEvent) => {
      keyboardShortcutsManager.handleKeyDown(e);
    };

    window.addEventListener('keydown', handleKeyDown);

    return () => {
      window.removeEventListener('keydown', handleKeyDown);
      unregisterQuickSwitcher();
      unregisterSearch();
      unregisterHelp();
      unregisterEscape();
    };
  }, [router]);

  return (
    <div className={`flex h-screen bg-gray-100 ${isResizing ? 'select-none' : ''}`}>
      <Sidebar
        activeChannel={activeChannel}
        activeDm={activeDm}
        onSelectChannel={handleSelectChannel}
        onSelectDm={handleSelectDm}
        width={sidebarWidth}
      />
      {/* Resize handle */}
      <div
        className={`w-1 cursor-col-resize bg-gray-700 hover:bg-blue-500 transition-colors ${
          isResizing ? 'bg-blue-500' : ''
        }`}
        onMouseDown={handleResizeStart}
      />
      <MessageArea
        channel={activeChannel}
        dm={activeDm}
        onLeaveChannel={handleLeaveChannel}
      />

      {/* Modals */}
      <KeyboardShortcutsHelp
        isOpen={showShortcutsHelp}
        onClose={() => setShowShortcutsHelp(false)}
      />
      <QuickSwitcher
        isOpen={showQuickSwitcher}
        onClose={() => setShowQuickSwitcher(false)}
        onSelectChannel={handleSelectChannel}
        onSelectDm={handleSelectDm}
      />
    </div>
  );
}
