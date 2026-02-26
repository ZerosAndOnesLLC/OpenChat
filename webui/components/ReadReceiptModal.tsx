'use client';

import { useEffect, useState } from 'react';
import { apiClient } from '@/lib/api';
import { useFocusTrap } from '@/hooks/useFocusTrap';
import type { ReadReceiptWithUser } from '@/lib/types';

interface ReadReceiptModalProps {
  messageId: string;
  isOpen: boolean;
  onClose: () => void;
}

export default function ReadReceiptModal({ messageId, isOpen, onClose }: ReadReceiptModalProps) {
  const [receipts, setReceipts] = useState<ReadReceiptWithUser[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const trapRef = useFocusTrap(isOpen);

  useEffect(() => {
    if (isOpen && messageId) {
      loadReceipts();
    }
  }, [isOpen, messageId]);

  const loadReceipts = async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await apiClient.getMessageReceipts(messageId);
      setReceipts(data);
    } catch (err) {
      console.error('Failed to load read receipts:', err);
      setError('Failed to load read receipts');
    } finally {
      setLoading(false);
    }
  };

  const formatTime = (timestamp: string) => {
    const date = new Date(timestamp);
    return date.toLocaleString('en-US', {
      month: 'short',
      day: 'numeric',
      hour: 'numeric',
      minute: '2-digit',
      hour12: true,
    });
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-50 animate-fade-in" onClick={onClose}>
      <div
        ref={trapRef}
        role="dialog"
        aria-modal="true"
        aria-labelledby="read-receipt-modal-title"
        className="w-full max-w-md rounded-lg bg-gray-900 p-6 shadow-xl animate-modal-in"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="mb-4 flex items-center justify-between">
          <h2 id="read-receipt-modal-title" className="text-xl font-semibold text-white">Read by</h2>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-white"
          >
            <svg className="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        {loading ? (
          <div className="flex items-center justify-center py-8">
            <div className="h-8 w-8 animate-spin rounded-full border-b-2 border-blue-500"></div>
          </div>
        ) : error ? (
          <div className="py-4 text-center text-red-400">{error}</div>
        ) : receipts.length === 0 ? (
          <div className="py-4 text-center text-gray-400">No one has read this message yet</div>
        ) : (
          <div className="max-h-96 space-y-3 overflow-y-auto">
            {receipts.map((receipt) => (
              <div key={receipt.id} className="flex items-center gap-3 rounded-lg bg-gray-800 p-3">
                <div className="flex h-10 w-10 flex-shrink-0 items-center justify-center rounded-full bg-blue-600 text-sm font-semibold text-white">
                  {receipt.display_name.charAt(0).toUpperCase()}
                </div>
                <div className="flex-1">
                  <div className="font-medium text-white">{receipt.display_name}</div>
                  <div className="text-xs text-gray-400">{formatTime(receipt.read_at)}</div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
