'use client';

import { useState, useEffect } from 'react';
import Sidebar from './Sidebar';
import MessageArea from './MessageArea';
import KeyboardShortcutsHelp from './KeyboardShortcutsHelp';
import QuickSwitcher from './QuickSwitcher';
import { keyboardShortcutsManager, SHORTCUT_CATEGORIES } from '@/lib/keyboard-shortcuts';
import type { Channel, DirectMessage } from '@/lib/types';

export default function ChatLayout() {
  const [activeChannel, setActiveChannel] = useState<Channel | null>(null);
  const [activeDm, setActiveDm] = useState<DirectMessage | null>(null);
  const [showShortcutsHelp, setShowShortcutsHelp] = useState(false);
  const [showQuickSwitcher, setShowQuickSwitcher] = useState(false);

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
      unregisterHelp();
      unregisterEscape();
    };
  }, []);

  return (
    <div className="flex h-screen bg-gray-100">
      <Sidebar
        activeChannel={activeChannel}
        activeDm={activeDm}
        onSelectChannel={handleSelectChannel}
        onSelectDm={handleSelectDm}
      />
      <MessageArea
        channel={activeChannel}
        dm={activeDm}
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
