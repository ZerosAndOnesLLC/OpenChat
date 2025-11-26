'use client';

import { useState, useRef } from 'react';
import { apiClient } from '@/lib/api';
import { CustomEmoji } from '@/lib/types';

interface EmojiUploadDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onUploadSuccess: (emoji: CustomEmoji) => void;
}

export default function EmojiUploadDialog({ isOpen, onClose, onUploadSuccess }: EmojiUploadDialogProps) {
  const [emojiName, setEmojiName] = useState('');
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [previewUrl, setPreviewUrl] = useState<string | null>(null);
  const [uploading, setUploading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);

  if (!isOpen) return null;

  const handleFileSelect = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    // Validate file type
    if (!['image/jpeg', 'image/png', 'image/gif', 'image/webp'].includes(file.type)) {
      setError('Only JPEG, PNG, GIF, and WebP images are allowed');
      return;
    }

    // Validate file size (512KB max)
    if (file.size > 512 * 1024) {
      setError('File size must be less than 512KB');
      return;
    }

    setSelectedFile(file);
    setError(null);

    // Create preview
    const reader = new FileReader();
    reader.onloadend = () => {
      setPreviewUrl(reader.result as string);
    };
    reader.readAsDataURL(file);
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!emojiName || !selectedFile) {
      setError('Please provide both name and image');
      return;
    }

    // Validate emoji name format
    if (!/^[a-zA-Z0-9_-]+$/.test(emojiName)) {
      setError('Emoji name can only contain letters, numbers, underscores, and hyphens');
      return;
    }

    setUploading(true);
    setError(null);

    try {
      const result = await apiClient.uploadCustomEmoji(emojiName, selectedFile);

      // Create a CustomEmoji object from the response
      const newEmoji: CustomEmoji = {
        id: result.id,
        org_id: '', // Will be filled by backend
        name: result.name,
        image_url: result.image_url,
        storage_type: result.storage_type,
        storage_path: '',
        created_by: '',
        created_at: result.created_at,
      };

      onUploadSuccess(newEmoji);
      handleClose();
    } catch (err: any) {
      setError(err.message || 'Failed to upload emoji');
    } finally {
      setUploading(false);
    }
  };

  const handleClose = () => {
    setEmojiName('');
    setSelectedFile(null);
    setPreviewUrl(null);
    setError(null);
    onClose();
  };

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white dark:bg-gray-800 rounded-lg p-6 max-w-md w-full mx-4">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-xl font-semibold text-gray-900 dark:text-white">
            Upload Custom Emoji
          </h2>
          <button
            onClick={handleClose}
            className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
          >
            <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        <form onSubmit={handleSubmit} className="space-y-4">
          {/* Emoji Name */}
          <div>
            <label htmlFor="emojiName" className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
              Emoji Name
            </label>
            <input
              type="text"
              id="emojiName"
              value={emojiName}
              onChange={(e) => setEmojiName(e.target.value)}
              placeholder="e.g., company_logo"
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
              disabled={uploading}
              required
            />
            <p className="mt-1 text-xs text-gray-500 dark:text-gray-400">
              Letters, numbers, underscores, and hyphens only
            </p>
          </div>

          {/* File Upload */}
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
              Emoji Image
            </label>
            <div className="flex items-center space-x-4">
              <button
                type="button"
                onClick={() => fileInputRef.current?.click()}
                className="px-4 py-2 bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 rounded-md hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors"
                disabled={uploading}
              >
                Choose File
              </button>
              <span className="text-sm text-gray-600 dark:text-gray-400">
                {selectedFile ? selectedFile.name : 'No file chosen'}
              </span>
            </div>
            <input
              ref={fileInputRef}
              type="file"
              accept="image/jpeg,image/png,image/gif,image/webp"
              onChange={handleFileSelect}
              className="hidden"
              disabled={uploading}
            />
            <p className="mt-1 text-xs text-gray-500 dark:text-gray-400">
              PNG, JPEG, GIF, or WebP • Max 512KB • Will be resized to 128x128px
            </p>
          </div>

          {/* Preview */}
          {previewUrl && (
            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                Preview
              </label>
              <div className="flex items-center space-x-2">
                <div className="w-16 h-16 bg-gray-100 dark:bg-gray-700 rounded flex items-center justify-center">
                  <img
                    src={previewUrl}
                    alt="Preview"
                    className="max-w-full max-h-full object-contain"
                  />
                </div>
                <div className="text-sm text-gray-600 dark:text-gray-400">
                  Usage: <code className="bg-gray-100 dark:bg-gray-700 px-1 rounded">:{emojiName}:</code>
                </div>
              </div>
            </div>
          )}

          {/* Error Message */}
          {error && (
            <div className="p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-md">
              <p className="text-sm text-red-600 dark:text-red-400">{error}</p>
            </div>
          )}

          {/* Actions */}
          <div className="flex justify-end space-x-3 pt-4">
            <button
              type="button"
              onClick={handleClose}
              className="px-4 py-2 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-md transition-colors"
              disabled={uploading}
            >
              Cancel
            </button>
            <button
              type="submit"
              className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
              disabled={uploading || !emojiName || !selectedFile}
            >
              {uploading ? 'Uploading...' : 'Upload Emoji'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
