'use client';

import { useEffect, useState } from 'react';
import { apiClient } from '@/lib/api';
import { useFocusTrap } from '@/hooks/useFocusTrap';
import type { MessageEditWithUser } from '@/lib/types';
import MarkdownRenderer from './MarkdownRenderer';

interface EditHistoryModalProps {
  messageId: string;
  currentContent: string;
  isOpen: boolean;
  onClose: () => void;
}

export default function EditHistoryModal({ messageId, currentContent, isOpen, onClose }: EditHistoryModalProps) {
  const [history, setHistory] = useState<MessageEditWithUser[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [selectedIndex, setSelectedIndex] = useState<number | null>(null);
  const trapRef = useFocusTrap(isOpen);

  useEffect(() => {
    if (isOpen && messageId) {
      loadHistory();
    }
  }, [isOpen, messageId]);

  const loadHistory = async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await apiClient.getMessageHistory(messageId);
      setHistory(data);
    } catch (err) {
      console.error('Failed to load edit history:', err);
      setError('Failed to load edit history');
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

  // Generate a simple word-level diff
  const generateDiff = (oldText: string, newText: string) => {
    const oldWords = oldText.split(/(\s+)/);
    const newWords = newText.split(/(\s+)/);

    const result: Array<{ type: 'added' | 'removed' | 'unchanged'; text: string }> = [];
    let oldIdx = 0;
    let newIdx = 0;

    while (oldIdx < oldWords.length || newIdx < newWords.length) {
      if (oldIdx >= oldWords.length) {
        // Remaining words are all added
        result.push({ type: 'added', text: newWords[newIdx] });
        newIdx++;
      } else if (newIdx >= newWords.length) {
        // Remaining words are all removed
        result.push({ type: 'removed', text: oldWords[oldIdx] });
        oldIdx++;
      } else if (oldWords[oldIdx] === newWords[newIdx]) {
        // Words match
        result.push({ type: 'unchanged', text: oldWords[oldIdx] });
        oldIdx++;
        newIdx++;
      } else {
        // Words differ - try to find the word in the other array
        const oldInNew = newWords.indexOf(oldWords[oldIdx], newIdx);
        const newInOld = oldWords.indexOf(newWords[newIdx], oldIdx);

        if (oldInNew !== -1 && (newInOld === -1 || oldInNew - newIdx < newInOld - oldIdx)) {
          // Old word appears later in new, so these are additions
          result.push({ type: 'added', text: newWords[newIdx] });
          newIdx++;
        } else if (newInOld !== -1) {
          // New word appears later in old, so these are removals
          result.push({ type: 'removed', text: oldWords[oldIdx] });
          oldIdx++;
        } else {
          // No match found, treat as removal + addition
          result.push({ type: 'removed', text: oldWords[oldIdx] });
          result.push({ type: 'added', text: newWords[newIdx] });
          oldIdx++;
          newIdx++;
        }
      }
    }

    return result;
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-50 animate-fade-in" onClick={onClose}>
      <div
        ref={trapRef}
        role="dialog"
        aria-modal="true"
        aria-labelledby="edit-history-modal-title"
        className="w-full max-w-3xl rounded-lg bg-gray-900 p-6 shadow-xl animate-modal-in"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="mb-4 flex items-center justify-between">
          <h2 id="edit-history-modal-title" className="text-xl font-semibold text-white">Edit History</h2>
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
        ) : history.length === 0 ? (
          <div className="py-4 text-center text-gray-400">No edit history available</div>
        ) : (
          <div className="max-h-96 space-y-4 overflow-y-auto">
            {/* Current version */}
            <div
              className={`cursor-pointer rounded-lg border p-4 transition-colors ${
                selectedIndex === null
                  ? 'border-blue-500 bg-gray-800'
                  : 'border-gray-700 bg-gray-800 hover:border-gray-600'
              }`}
              onClick={() => setSelectedIndex(null)}
            >
              <div className="mb-2 flex items-center justify-between">
                <span className="text-sm font-semibold text-green-400">Current Version</span>
                <span className="text-xs text-gray-400">Latest</span>
              </div>
              {selectedIndex === null ? (
                <div className="text-sm text-white">
                  <MarkdownRenderer content={currentContent} />
                </div>
              ) : (
                <div className="text-sm text-gray-400">Click to view</div>
              )}
            </div>

            {/* Edit history */}
            {history.map((edit, index) => {
              const previousContent = index < history.length - 1 ? history[index + 1].old_content : currentContent;
              const diff = generateDiff(edit.old_content, previousContent);

              return (
                <div
                  key={edit.id}
                  className={`cursor-pointer rounded-lg border p-4 transition-colors ${
                    selectedIndex === index
                      ? 'border-blue-500 bg-gray-800'
                      : 'border-gray-700 bg-gray-800 hover:border-gray-600'
                  }`}
                  onClick={() => setSelectedIndex(index)}
                >
                  <div className="mb-2 flex items-center justify-between">
                    <span className="text-sm font-medium text-white">
                      Edited by {edit.editor_name}
                    </span>
                    <span className="text-xs text-gray-400">{formatTime(edit.edited_at)}</span>
                  </div>
                  {selectedIndex === index ? (
                    <div className="space-y-3">
                      <div>
                        <div className="mb-1 text-xs font-semibold text-gray-400">Previous Content:</div>
                        <div className="rounded bg-gray-900 p-3 text-sm text-white">
                          <MarkdownRenderer content={edit.old_content} />
                        </div>
                      </div>
                      <div>
                        <div className="mb-1 text-xs font-semibold text-gray-400">Changes:</div>
                        <div className="rounded bg-gray-900 p-3 text-sm">
                          {diff.map((part, idx) => {
                            if (part.type === 'added') {
                              return (
                                <span key={idx} className="bg-green-900 text-green-200">
                                  {part.text}
                                </span>
                              );
                            } else if (part.type === 'removed') {
                              return (
                                <span key={idx} className="bg-red-900 text-red-200 line-through">
                                  {part.text}
                                </span>
                              );
                            } else {
                              return (
                                <span key={idx} className="text-gray-300">
                                  {part.text}
                                </span>
                              );
                            }
                          })}
                        </div>
                      </div>
                    </div>
                  ) : (
                    <div className="text-sm text-gray-400">Click to view details</div>
                  )}
                </div>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
}
