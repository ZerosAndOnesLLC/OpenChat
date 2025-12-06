'use client';

import { useState, useRef, useEffect } from 'react';
import { useQueryClient } from '@tanstack/react-query';
import { useWebSocketStore } from '@/lib/websocket';
import { draftsManager } from '@/lib/drafts';
import { keyboardShortcutsManager, SHORTCUT_CATEGORIES } from '@/lib/keyboard-shortcuts';
import { apiClient } from '@/lib/api';
import type { Message } from '@/lib/types';
import MarkdownToolbar from './MarkdownToolbar';
import MarkdownRenderer from './MarkdownRenderer';

interface MessageInputProps {
  channelId?: string;
  dmId?: string;
  replyTo?: Message;
  onClearReply?: () => void;
}

interface SelectedFile {
  file: File;
  preview?: string;
  progress: number;
  uploading: boolean;
  error?: string;
}

export default function MessageInput({ channelId, dmId, replyTo, onClearReply }: MessageInputProps) {
  const [message, setMessage] = useState('');
  const [showPreview, setShowPreview] = useState(false);
  const [selectedFiles, setSelectedFiles] = useState<SelectedFile[]>([]);
  const [isDragging, setIsDragging] = useState(false);
  const { sendMessage, sendTyping } = useWebSocketStore();
  const queryClient = useQueryClient();
  const typingTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const currentDraftKey = useRef<string | null>(null);
  const dragCounterRef = useRef(0);
  const lastSavedDraft = useRef<string>('');

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!message.trim() && selectedFiles.length === 0) return;

    try {
      if (selectedFiles.length > 0) {
        // If we have files, create message via API first to get message ID
        const messageData = {
          content: message.trim() || '(attached files)',
          channel_id: channelId || undefined,
          dm_id: dmId || undefined,
          parent_message_id: replyTo?.id || undefined,
        };

        // Create message via API to get the message ID
        const response = await fetch(`${process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080'}/api/messages`, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${apiClient.getToken()}`,
          },
          body: JSON.stringify(messageData),
        });

        if (!response.ok) {
          throw new Error('Failed to create message');
        }

        const createdMessage = await response.json();
        const messageId = createdMessage.id;

        // Upload files with progress tracking
        for (let i = 0; i < selectedFiles.length; i++) {
          setSelectedFiles((prev) =>
            prev.map((f, idx) => (idx === i ? { ...f, uploading: true, progress: 0 } : f))
          );

          try {
            // Note: The API client's uploadAttachment doesn't support progress yet
            // We'll use fetch directly to track progress
            const formData = new FormData();
            formData.append('file', selectedFiles[i].file);
            formData.append('message_id', messageId);

            await fetch(`${process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080'}/api/attachments/upload`, {
              method: 'POST',
              headers: {
                'Authorization': `Bearer ${apiClient.getToken()}`,
              },
              body: formData,
            });

            setSelectedFiles((prev) =>
              prev.map((f, idx) => (idx === i ? { ...f, progress: 100 } : f))
            );
          } catch (error) {
            console.error('Failed to upload file:', error);
            setSelectedFiles((prev) =>
              prev.map((f, idx) => (idx === i ? { ...f, error: 'Upload failed' } : f))
            );
          }
        }

        // Invalidate messages query to refetch with attachments
        const messageKey = channelId || dmId;
        if (messageKey) {
          queryClient.invalidateQueries({ queryKey: ['messages', messageKey] });
        }
      } else {
        // No files, send via WebSocket as usual
        sendMessage(channelId, dmId, message.trim(), replyTo?.id);
      }

      setMessage('');
      setSelectedFiles([]);

      // Clear draft after sending
      const draftKey = channelId || dmId;
      if (draftKey) {
        try {
          await draftsManager.deleteDraft(draftKey);
        } catch (error) {
          console.error('Failed to clear draft:', error);
        }
      }

      // Clear reply after sending
      if (onClearReply) {
        onClearReply();
      }

      if (typingTimeoutRef.current) {
        clearTimeout(typingTimeoutRef.current);
        typingTimeoutRef.current = null;
      }
    } catch (error) {
      console.error('Failed to send message:', error);
      // Don't clear the message/files on error so user can retry
    }
  };

  const handleFileSelect = (files: FileList | null) => {
    if (!files || files.length === 0) return;

    const newFiles: SelectedFile[] = Array.from(files).map((file) => {
      const selectedFile: SelectedFile = {
        file,
        progress: 0,
        uploading: false,
      };

      // Generate preview for images
      if (file.type.startsWith('image/')) {
        const reader = new FileReader();
        reader.onload = (e) => {
          setSelectedFiles((prev) =>
            prev.map((f) =>
              f.file === file ? { ...f, preview: e.target?.result as string } : f
            )
          );
        };
        reader.readAsDataURL(file);
      }

      return selectedFile;
    });

    setSelectedFiles((prev) => [...prev, ...newFiles]);
  };

  const handleFileInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    handleFileSelect(e.target.files);
    // Reset input so the same file can be selected again
    if (fileInputRef.current) {
      fileInputRef.current.value = '';
    }
  };

  const handleRemoveFile = (index: number) => {
    setSelectedFiles((prev) => prev.filter((_, i) => i !== index));
  };

  const handleDragEnter = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    dragCounterRef.current++;
    if (e.dataTransfer.items && e.dataTransfer.items.length > 0) {
      setIsDragging(true);
    }
  };

  const handleDragLeave = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    dragCounterRef.current--;
    if (dragCounterRef.current === 0) {
      setIsDragging(false);
    }
  };

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
  };

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(false);
    dragCounterRef.current = 0;

    const files = e.dataTransfer.files;
    handleFileSelect(files);
  };

  const getFileIcon = (fileType: string) => {
    if (fileType.startsWith('image/')) {
      return (
        <svg className="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
        </svg>
      );
    } else if (fileType.startsWith('video/')) {
      return (
        <svg className="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 10l4.553-2.276A1 1 0 0121 8.618v6.764a1 1 0 01-1.447.894L15 14M5 18h8a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v8a2 2 0 002 2z" />
        </svg>
      );
    } else if (fileType.includes('pdf')) {
      return (
        <svg className="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M7 21h10a2 2 0 002-2V9.414a1 1 0 00-.293-.707l-5.414-5.414A1 1 0 0012.586 3H7a2 2 0 00-2 2v14a2 2 0 002 2z" />
        </svg>
      );
    } else {
      return (
        <svg className="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
        </svg>
      );
    }
  };

  const formatFileSize = (bytes: number): string => {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i];
  };

  const handleChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    const newValue = e.target.value;
    setMessage(newValue);

    // Send typing indicator
    if (typingTimeoutRef.current) {
      clearTimeout(typingTimeoutRef.current);
    }

    if (newValue.trim()) {
      sendTyping(channelId, dmId);
      typingTimeoutRef.current = setTimeout(() => {
        typingTimeoutRef.current = null;
      }, 3000);
    }
    // Draft saving is handled only on blur (handleBlur) for better performance
  };

  // Save draft when input loses focus
  const handleBlur = async () => {
    const draftKey = channelId || dmId;
    if (!draftKey) return;

    // Only save if content has changed since last save
    if (message !== lastSavedDraft.current) {
      try {
        if (message.trim()) {
          await draftsManager.saveDraft(draftKey, message);
          lastSavedDraft.current = message;
        } else {
          await draftsManager.deleteDraft(draftKey);
          lastSavedDraft.current = '';
        }
      } catch (error) {
        console.error('Failed to save draft on blur:', error);
      }
    }
  };

  const handleFormat = (before: string, after: string, placeholder?: string) => {
    const textarea = textareaRef.current;
    if (!textarea) return;

    const start = textarea.selectionStart;
    const end = textarea.selectionEnd;
    const selectedText = message.substring(start, end);
    const textToInsert = selectedText || placeholder || '';

    const newText =
      message.substring(0, start) +
      before +
      textToInsert +
      after +
      message.substring(end);

    setMessage(newText);

    // Set cursor position after formatting
    setTimeout(() => {
      const newCursorPos = start + before.length + textToInsert.length;
      textarea.focus();
      textarea.setSelectionRange(newCursorPos, newCursorPos);
    }, 0);
  };

  const handleTogglePreview = () => {
    setShowPreview(!showPreview);
  };

  // Load draft when channel/DM changes
  useEffect(() => {
    const loadDraft = async () => {
      const draftKey = channelId || dmId;
      if (!draftKey) return;

      // Save current draft before switching (only if content changed)
      if (currentDraftKey.current && currentDraftKey.current !== draftKey) {
        if (message !== lastSavedDraft.current) {
          try {
            if (message.trim()) {
              await draftsManager.saveDraft(currentDraftKey.current, message);
            } else {
              await draftsManager.deleteDraft(currentDraftKey.current);
            }
          } catch (error) {
            console.error('Failed to save previous draft:', error);
          }
        }
      }

      // Load new draft
      try {
        const draft = await draftsManager.getDraft(draftKey);
        setMessage(draft || '');
        lastSavedDraft.current = draft || '';
        currentDraftKey.current = draftKey;
      } catch (error) {
        console.error('Failed to load draft:', error);
        setMessage('');
        lastSavedDraft.current = '';
      }
    };

    loadDraft();
  }, [channelId, dmId]);

  // Cleanup timeouts on unmount
  useEffect(() => {
    return () => {
      if (typingTimeoutRef.current) {
        clearTimeout(typingTimeoutRef.current);
      }
    };
  }, []);

  // Register keyboard shortcuts for message input
  useEffect(() => {
    // Cmd/Ctrl+Enter: Send message
    const unregisterSend = keyboardShortcutsManager.register({
      key: 'enter',
      ctrl: true,
      meta: true,
      description: 'Send message',
      category: SHORTCUT_CATEGORIES.MESSAGING,
      handler: () => {
        if (message.trim()) {
          handleSubmit(new Event('submit') as any);
        }
      },
    });

    return () => {
      unregisterSend();
    };
  }, [message]);

  // Handle keyboard events in textarea
  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    // Enter: Send message (without shift)
    // Shift+Enter: Insert newline
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      if (message.trim() || selectedFiles.length > 0) {
        handleSubmit(new Event('submit') as any);
      }
    }

    // Up arrow: Edit last message (when input is empty)
    // Note: Full implementation would require access to user's messages and edit mode
    if (e.key === 'ArrowUp' && !e.shiftKey && !e.ctrlKey && !e.metaKey && message.trim() === '') {
      e.preventDefault();
      // This is a placeholder for the edit last message functionality
      // The full implementation would require MessageArea to pass a callback
    }
  };

  return (
    <div className="border-t border-gray-800 relative"
      onDragEnter={handleDragEnter}
      onDragLeave={handleDragLeave}
      onDragOver={handleDragOver}
      onDrop={handleDrop}
    >
      {/* Drag and drop overlay */}
      {isDragging && (
        <div className="absolute inset-0 z-50 flex items-center justify-center bg-blue-900 bg-opacity-90 border-2 border-dashed border-blue-400">
          <div className="text-center">
            <svg className="mx-auto h-16 w-16 text-blue-300" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
            </svg>
            <p className="mt-2 text-lg font-semibold text-blue-200">Drop files here to upload</p>
          </div>
        </div>
      )}

      {replyTo && (
        <div className="mx-6 mt-4 mb-2 flex items-center gap-2 rounded-lg bg-gray-800 px-3 py-2">
          <div className="flex-1">
            <div className="text-xs text-gray-400">
              Replying to <span className="font-semibold text-white">{replyTo.user?.display_name || 'Unknown User'}</span>
            </div>
            <div className="truncate text-sm text-gray-300">{replyTo.content}</div>
          </div>
          <button
            onClick={onClearReply}
            className="text-gray-400 hover:text-white"
            title="Cancel reply"
          >
            <svg className="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
      )}

      {/* Selected files preview */}
      {selectedFiles.length > 0 && (
        <div className="mx-3 mt-3 mb-2 flex flex-wrap gap-2">
          {selectedFiles.map((selectedFile, index) => (
            <div key={index} className="relative group">
              {selectedFile.preview ? (
                // Image preview
                <div className="relative h-24 w-24 rounded-lg overflow-hidden border border-gray-700 bg-gray-800">
                  <img
                    src={selectedFile.preview}
                    alt={selectedFile.file.name}
                    className="h-full w-full object-cover"
                  />
                  {selectedFile.uploading && (
                    <div className="absolute inset-0 flex items-center justify-center bg-black bg-opacity-50">
                      <div className="text-xs text-white">{selectedFile.progress}%</div>
                    </div>
                  )}
                  <button
                    onClick={() => handleRemoveFile(index)}
                    className="absolute top-1 right-1 rounded-full bg-red-600 p-1 opacity-0 group-hover:opacity-100 transition-opacity"
                    title="Remove file"
                  >
                    <svg className="h-3 w-3 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                    </svg>
                  </button>
                </div>
              ) : (
                // File icon for non-images
                <div className="relative h-24 w-32 rounded-lg border border-gray-700 bg-gray-800 p-2 flex flex-col items-center justify-center">
                  <div className="text-gray-400">{getFileIcon(selectedFile.file.type)}</div>
                  <div className="mt-1 text-xs text-gray-300 truncate w-full text-center">
                    {selectedFile.file.name}
                  </div>
                  <div className="text-xs text-gray-500">{formatFileSize(selectedFile.file.size)}</div>
                  {selectedFile.uploading && (
                    <div className="mt-1 w-full bg-gray-700 rounded-full h-1">
                      <div
                        className="bg-blue-600 h-1 rounded-full transition-all"
                        style={{ width: `${selectedFile.progress}%` }}
                      />
                    </div>
                  )}
                  <button
                    onClick={() => handleRemoveFile(index)}
                    className="absolute top-1 right-1 rounded-full bg-red-600 p-1 opacity-0 group-hover:opacity-100 transition-opacity"
                    title="Remove file"
                  >
                    <svg className="h-3 w-3 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                    </svg>
                  </button>
                </div>
              )}
            </div>
          ))}
        </div>
      )}

      <form onSubmit={handleSubmit} className="flex flex-col">
        <input
          ref={fileInputRef}
          type="file"
          multiple
          onChange={handleFileInputChange}
          className="hidden"
          accept="image/*,video/*,audio/*,.pdf,.doc,.docx,.xls,.xlsx,.txt"
        />
        <div className="relative mx-3 my-2 rounded-lg border border-gray-700 bg-gray-900">
          <div className="flex flex-col">
            {showPreview ? (
              <div className="min-h-[48px] max-h-[200px] overflow-y-auto pl-12 pr-14 py-3 text-sm text-white">
                {message.trim() ? (
                  <MarkdownRenderer content={message} />
                ) : (
                  <div className="text-gray-400">Nothing to preview...</div>
                )}
              </div>
            ) : (
              <textarea
                ref={textareaRef}
                value={message}
                onChange={handleChange}
                onBlur={handleBlur}
                onKeyDown={handleKeyDown}
                placeholder={replyTo ? "Type your reply..." : "Type a message..."}
                className="w-full min-h-[48px] max-h-[200px] bg-transparent pl-12 pr-14 py-3 text-sm text-white placeholder-gray-500 focus:outline-none resize-none overflow-y-auto leading-relaxed"
                rows={2}
              />
            )}
            <MarkdownToolbar
              onFormat={handleFormat}
              onTogglePreview={handleTogglePreview}
              showPreview={showPreview}
            />
          </div>
          <button
            type="button"
            onClick={() => fileInputRef.current?.click()}
            className="absolute left-2 top-2 p-2 text-gray-400 transition-colors hover:text-white z-10"
            title="Attach files"
          >
            <svg className="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15.172 7l-6.586 6.586a2 2 0 102.828 2.828l6.414-6.586a4 4 0 00-5.656-5.656l-6.415 6.585a6 6 0 108.486 8.486L20.5 13" />
            </svg>
          </button>
          <button
            type="submit"
            disabled={!message.trim() && selectedFiles.length === 0}
            className="absolute right-3 top-3 rounded-md bg-blue-600 p-2 text-white transition-colors hover:bg-blue-700 disabled:bg-gray-700 disabled:text-gray-400 disabled:cursor-not-allowed z-20"
          >
            <svg className="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8" />
            </svg>
          </button>
        </div>
      </form>
    </div>
  );
}
