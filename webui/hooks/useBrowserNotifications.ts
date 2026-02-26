'use client';

import { useEffect, useRef, useCallback, useSyncExternalStore } from 'react';
import { useWebSocketStore } from '@/lib/websocket';

const STORAGE_KEY = 'browserNotificationsEnabled';

let cachedEnabled: boolean | null = null;
const listeners = new Set<() => void>();

function getEnabled(): boolean {
  if (cachedEnabled !== null) return cachedEnabled;
  if (typeof window === 'undefined') return false;
  try {
    cachedEnabled = localStorage.getItem(STORAGE_KEY) === 'true';
  } catch {
    cachedEnabled = false;
  }
  return cachedEnabled;
}

function setEnabled(val: boolean) {
  cachedEnabled = val;
  if (typeof window !== 'undefined') {
    localStorage.setItem(STORAGE_KEY, String(val));
  }
  listeners.forEach((l) => l());
}

function subscribe(listener: () => void) {
  listeners.add(listener);
  return () => { listeners.delete(listener); };
}

function getSnapshot() {
  return getEnabled();
}

function getServerSnapshot() {
  return false;
}

// --- Tab title badge ---
let originalTitle = '';
let badgeInterval: ReturnType<typeof setInterval> | null = null;

function updateTitleBadge(count: number) {
  if (typeof document === 'undefined') return;
  if (!originalTitle) {
    originalTitle = document.title.replace(/^\(\d+\)\s*/, '');
  }
  if (count > 0) {
    document.title = `(${count}) ${originalTitle}`;
  } else {
    document.title = originalTitle;
  }
}

// --- Dynamic favicon badge ---
let originalFavicon: string | null = null;

function setFaviconBadge(count: number) {
  if (typeof document === 'undefined') return;

  const link: HTMLLinkElement =
    document.querySelector('link[rel="icon"]') ||
    (() => {
      const el = document.createElement('link');
      el.rel = 'icon';
      document.head.appendChild(el);
      return el;
    })();

  // Store original
  if (!originalFavicon && link.href) {
    originalFavicon = link.href;
  }

  if (count <= 0) {
    if (originalFavicon) link.href = originalFavicon;
    return;
  }

  // Draw badge on canvas
  const canvas = document.createElement('canvas');
  canvas.width = 32;
  canvas.height = 32;
  const ctx = canvas.getContext('2d');
  if (!ctx) return;

  // If there's an existing favicon, draw it first
  if (originalFavicon) {
    const img = new Image();
    img.crossOrigin = 'anonymous';
    img.onload = () => {
      ctx.drawImage(img, 0, 0, 32, 32);
      drawBadge(ctx, count);
      link.href = canvas.toDataURL('image/png');
    };
    img.onerror = () => {
      // Draw just the badge on blank canvas
      ctx.fillStyle = '#1a1a2e';
      ctx.fillRect(0, 0, 32, 32);
      drawBadge(ctx, count);
      link.href = canvas.toDataURL('image/png');
    };
    img.src = originalFavicon;
  } else {
    ctx.fillStyle = '#1a1a2e';
    ctx.fillRect(0, 0, 32, 32);
    drawBadge(ctx, count);
    link.href = canvas.toDataURL('image/png');
  }
}

function drawBadge(ctx: CanvasRenderingContext2D, count: number) {
  const text = count > 99 ? '99+' : String(count);
  const radius = text.length > 2 ? 14 : 10;

  ctx.beginPath();
  ctx.arc(32 - radius, radius, radius, 0, 2 * Math.PI);
  ctx.fillStyle = '#ef4444';
  ctx.fill();

  ctx.fillStyle = '#ffffff';
  ctx.font = `bold ${text.length > 2 ? 10 : 12}px sans-serif`;
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  ctx.fillText(text, 32 - radius, radius + 1);
}

export function useBrowserNotifications() {
  const enabled = useSyncExternalStore(subscribe, getSnapshot, getServerSnapshot);
  const prevCountRef = useRef(0);

  // Compute total unread across all channels and DMs
  const totalUnread = useWebSocketStore((state) => {
    const counts = state.unreadCounts;
    let total = 0;
    for (const key of Object.keys(counts)) {
      total += counts[key];
    }
    return total;
  });

  // Update tab title and favicon on unread count changes
  useEffect(() => {
    updateTitleBadge(totalUnread);
    setFaviconBadge(totalUnread);
  }, [totalUnread]);

  // Clean up on unmount
  useEffect(() => {
    return () => {
      if (badgeInterval) {
        clearInterval(badgeInterval);
        badgeInterval = null;
      }
    };
  }, []);

  const requestPermission = useCallback(async () => {
    if (typeof Notification === 'undefined') return false;
    if (Notification.permission === 'granted') {
      setEnabled(true);
      return true;
    }
    if (Notification.permission === 'denied') return false;
    const result = await Notification.requestPermission();
    const granted = result === 'granted';
    setEnabled(granted);
    return granted;
  }, []);

  const toggle = useCallback(async (enable: boolean) => {
    if (enable) {
      return requestPermission();
    }
    setEnabled(false);
    return false;
  }, [requestPermission]);

  const showNotification = useCallback(
    (title: string, body: string) => {
      if (!enabled) return;
      if (typeof Notification === 'undefined') return;
      if (Notification.permission !== 'granted') return;
      // Only show when tab is hidden
      if (typeof document !== 'undefined' && !document.hidden) return;

      try {
        new Notification(title, {
          body,
          icon: '/favicon.ico',
          tag: 'openchat-notification',
        });
      } catch (err) {
        console.error('Failed to show browser notification:', err);
      }
    },
    [enabled]
  );

  return {
    enabled,
    permissionState: typeof Notification !== 'undefined' ? Notification.permission : 'default',
    toggle,
    requestPermission,
    showNotification,
    totalUnread,
  };
}
