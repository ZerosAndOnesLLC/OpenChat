/**
 * Keyboard Shortcuts Manager
 * Centralized keyboard shortcut handling for the application
 */

export interface KeyboardShortcut {
  key: string;
  ctrl?: boolean;
  meta?: boolean;
  shift?: boolean;
  alt?: boolean;
  description: string;
  category: string;
  handler: (e: KeyboardEvent) => void;
}

export const SHORTCUT_CATEGORIES = {
  NAVIGATION: 'Navigation',
  MESSAGING: 'Messaging',
  EDITING: 'Editing',
  GENERAL: 'General',
} as const;

class KeyboardShortcutsManager {
  private shortcuts: Map<string, KeyboardShortcut> = new Map();
  private enabled = true;

  /**
   * Generate a unique key for a shortcut
   */
  private getShortcutKey(shortcut: Omit<KeyboardShortcut, 'description' | 'category' | 'handler'>): string {
    const parts: string[] = [];
    if (shortcut.ctrl) parts.push('ctrl');
    if (shortcut.meta) parts.push('meta');
    if (shortcut.shift) parts.push('shift');
    if (shortcut.alt) parts.push('alt');
    parts.push(shortcut.key.toLowerCase());
    return parts.join('+');
  }

  /**
   * Register a keyboard shortcut
   */
  register(shortcut: KeyboardShortcut): () => void {
    const key = this.getShortcutKey(shortcut);
    this.shortcuts.set(key, shortcut);

    // Return unregister function
    return () => this.unregister(shortcut);
  }

  /**
   * Unregister a keyboard shortcut
   */
  unregister(shortcut: Omit<KeyboardShortcut, 'description' | 'category' | 'handler'>): void {
    const key = this.getShortcutKey(shortcut);
    this.shortcuts.delete(key);
  }

  /**
   * Handle keyboard event
   */
  handleKeyDown(e: KeyboardEvent): boolean {
    if (!this.enabled) return false;

    // Don't trigger shortcuts when typing in input fields (except for specific cases)
    const target = e.target as HTMLElement;
    const isInput = target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable;

    // Build the key combination
    const parts: string[] = [];
    if (e.ctrlKey) parts.push('ctrl');
    if (e.metaKey) parts.push('meta');
    if (e.shiftKey) parts.push('shift');
    if (e.altKey) parts.push('alt');
    parts.push(e.key.toLowerCase());
    const key = parts.join('+');

    const shortcut = this.shortcuts.get(key);
    if (shortcut) {
      // Allow some shortcuts even in input fields
      const allowInInput = ['meta+k', 'ctrl+k', 'meta+/', 'ctrl+/', 'escape'];

      if (isInput && !allowInInput.includes(key)) {
        return false;
      }

      e.preventDefault();
      e.stopPropagation();
      shortcut.handler(e);
      return true;
    }

    return false;
  }

  /**
   * Get all registered shortcuts
   */
  getAllShortcuts(): KeyboardShortcut[] {
    return Array.from(this.shortcuts.values());
  }

  /**
   * Get shortcuts by category
   */
  getShortcutsByCategory(): Record<string, KeyboardShortcut[]> {
    const shortcuts = this.getAllShortcuts();
    const byCategory: Record<string, KeyboardShortcut[]> = {};

    shortcuts.forEach((shortcut) => {
      if (!byCategory[shortcut.category]) {
        byCategory[shortcut.category] = [];
      }
      byCategory[shortcut.category].push(shortcut);
    });

    return byCategory;
  }

  /**
   * Enable/disable all shortcuts
   */
  setEnabled(enabled: boolean): void {
    this.enabled = enabled;
  }

  /**
   * Format shortcut for display
   */
  formatShortcut(shortcut: KeyboardShortcut): string {
    const isMac = typeof navigator !== 'undefined' && navigator.platform.toUpperCase().indexOf('MAC') >= 0;
    const parts: string[] = [];

    if (shortcut.ctrl && !isMac) parts.push('Ctrl');
    if (shortcut.meta || (shortcut.ctrl && isMac)) parts.push('⌘');
    if (shortcut.shift) parts.push('⇧');
    if (shortcut.alt) parts.push(isMac ? '⌥' : 'Alt');

    // Format special keys
    const keyName = shortcut.key === ' ' ? 'Space' :
                    shortcut.key === 'arrowup' ? '↑' :
                    shortcut.key === 'arrowdown' ? '↓' :
                    shortcut.key === 'arrowleft' ? '←' :
                    shortcut.key === 'arrowright' ? '→' :
                    shortcut.key === 'escape' ? 'Esc' :
                    shortcut.key === 'enter' ? 'Enter' :
                    shortcut.key.length === 1 ? shortcut.key.toUpperCase() : shortcut.key;

    parts.push(keyName);
    return parts.join(' + ');
  }
}

// Export singleton instance
export const keyboardShortcutsManager = new KeyboardShortcutsManager();

/**
 * Hook to use keyboard shortcuts in React components
 * Note: This is exported for potential future use, but is not currently used in the codebase.
 * Global shortcuts are registered directly in components using keyboardShortcutsManager.register().
 */
export function useKeyboardShortcut(
  shortcut: Omit<KeyboardShortcut, 'handler'>,
  handler: (e: KeyboardEvent) => void,
  deps: any[] = []
): void {
  // This is a placeholder implementation
  // Actual usage should directly use keyboardShortcutsManager.register() in useEffect
  if (typeof window !== 'undefined') {
    console.warn('useKeyboardShortcut is not implemented. Use keyboardShortcutsManager.register() directly in useEffect.');
  }
}
