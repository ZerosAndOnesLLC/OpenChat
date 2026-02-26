'use client';

import { useEffect, useRef } from 'react';
import { createPortal } from 'react-dom';

interface ContextMenuItem {
  label: string;
  onClick: () => void;
  danger?: boolean;
}

interface ContextMenuProps {
  x: number;
  y: number;
  items: ContextMenuItem[];
  onClose: () => void;
}

export default function ContextMenu({ x, y, items, onClose }: ContextMenuProps) {
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClick = (e: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        onClose();
      }
    };
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
    };
    document.addEventListener('mousedown', handleClick);
    document.addEventListener('keydown', handleEscape);
    return () => {
      document.removeEventListener('mousedown', handleClick);
      document.removeEventListener('keydown', handleEscape);
    };
  }, [onClose]);

  // Adjust position if menu would overflow viewport
  const adjustedY = Math.min(y, window.innerHeight - items.length * 36 - 16);
  const adjustedX = Math.min(x, window.innerWidth - 180);

  return createPortal(
    <div
      ref={menuRef}
      className="fixed z-50 min-w-[160px] rounded-lg bg-gray-800 border border-gray-700 py-1 shadow-xl"
      style={{ top: adjustedY, left: adjustedX }}
    >
      {items.map((item, index) => (
        <button
          key={index}
          onClick={() => { item.onClick(); onClose(); }}
          className={`w-full px-3 py-1.5 text-left text-sm hover:bg-gray-700 ${
            item.danger ? 'text-red-400 hover:text-red-300' : 'text-gray-300 hover:text-white'
          }`}
        >
          {item.label}
        </button>
      ))}
    </div>,
    document.body
  );
}
