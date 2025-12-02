'use client';

import { useState, useEffect } from 'react';
import { apiClient } from '@/lib/api';
import type { Attachment } from '@/lib/types';

interface AttachmentDisplayProps {
  attachments: Attachment[];
}

// Cache for blob URLs to avoid re-fetching
const blobUrlCache = new Map<string, string>();

export default function AttachmentDisplay({ attachments = [] }: AttachmentDisplayProps) {
  const [blobUrls, setBlobUrls] = useState<Record<string, string>>({});
  const [loadingIds, setLoadingIds] = useState<Set<string>>(new Set());

  // Load blob URLs for images and videos on mount
  useEffect(() => {
    const mediaAttachments = attachments.filter(
      a => a.file_type?.startsWith('image/') || a.file_type?.startsWith('video/')
    );

    mediaAttachments.forEach(async (attachment) => {
      // Check cache first
      if (blobUrlCache.has(attachment.id)) {
        setBlobUrls(prev => ({ ...prev, [attachment.id]: blobUrlCache.get(attachment.id)! }));
        return;
      }

      // Skip if already loading or already have URL
      if (loadingIds.has(attachment.id) || blobUrls[attachment.id]) return;

      setLoadingIds(prev => new Set(prev).add(attachment.id));

      try {
        const blob = await apiClient.downloadAttachment(attachment.id);
        const url = window.URL.createObjectURL(blob);
        blobUrlCache.set(attachment.id, url);
        setBlobUrls(prev => ({ ...prev, [attachment.id]: url }));
      } catch (err) {
        console.error('Failed to load media:', err);
      } finally {
        setLoadingIds(prev => {
          const next = new Set(prev);
          next.delete(attachment.id);
          return next;
        });
      }
    });
  }, [attachments, blobUrls, loadingIds]);

  const handleDownload = async (attachmentId: string, fileName: string) => {
    try {
      const blob = await apiClient.downloadAttachment(attachmentId);
      const url = window.URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = fileName;
      document.body.appendChild(a);
      a.click();
      window.URL.revokeObjectURL(url);
      document.body.removeChild(a);
    } catch (err) {
      console.error('Failed to download attachment:', err);
    }
  };

  const isImage = (fileType: string | null | undefined) => {
    return fileType?.startsWith('image/');
  };

  const isVideo = (fileType: string | null | undefined) => {
    return fileType?.startsWith('video/');
  };

  const isPDF = (fileType: string | null | undefined) => {
    return fileType?.includes('pdf');
  };

  const formatFileSize = (bytes: number): string => {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i];
  };

  const getFileIcon = (fileType: string | null | undefined) => {
    if (isVideo(fileType)) {
      return (
        <svg className="h-8 w-8" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 10l4.553-2.276A1 1 0 0121 8.618v6.764a1 1 0 01-1.447.894L15 14M5 18h8a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v8a2 2 0 002 2z" />
        </svg>
      );
    } else if (isPDF(fileType)) {
      return (
        <svg className="h-8 w-8" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M7 21h10a2 2 0 002-2V9.414a1 1 0 00-.293-.707l-5.414-5.414A1 1 0 0012.586 3H7a2 2 0 00-2 2v14a2 2 0 002 2z" />
        </svg>
      );
    } else {
      return (
        <svg className="h-8 w-8" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
        </svg>
      );
    }
  };

  if (!attachments || attachments.length === 0) {
    return null;
  }

  return (
    <div className="mt-2 space-y-2">
      {attachments.map((attachment) => {
        if (isImage(attachment.file_type)) {
          const imageSrc = blobUrls[attachment.id];
          const isLoading = loadingIds.has(attachment.id);

          // Inline image preview
          return (
            <div key={attachment.id} className="relative group max-w-md">
              {isLoading ? (
                <div className="flex items-center justify-center rounded-lg border border-gray-700 bg-gray-800 h-40 w-64">
                  <div className="flex flex-col items-center gap-2 text-gray-400">
                    <svg className="h-6 w-6 animate-spin" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                    </svg>
                    <span className="text-xs">Loading image...</span>
                  </div>
                </div>
              ) : imageSrc ? (
                <img
                  src={imageSrc}
                  alt={attachment.file_name}
                  className="rounded-lg border border-gray-700 max-h-80 cursor-pointer hover:opacity-90 transition-opacity"
                  onClick={() => handleDownload(attachment.id, attachment.file_name)}
                />
              ) : (
                <div className="flex items-center justify-center rounded-lg border border-gray-700 bg-gray-800 h-40 w-64">
                  <div className="flex flex-col items-center gap-2 text-gray-500">
                    <svg className="h-8 w-8" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
                    </svg>
                    <span className="text-xs">Image unavailable</span>
                  </div>
                </div>
              )}
              {imageSrc && (
                <div className="absolute top-2 right-2 opacity-0 group-hover:opacity-100 transition-opacity">
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      handleDownload(attachment.id, attachment.file_name);
                    }}
                    className="rounded-lg bg-gray-900 bg-opacity-80 p-2 text-white hover:bg-opacity-100"
                    title="Download"
                  >
                    <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
                    </svg>
                  </button>
                </div>
              )}
              <div className="mt-1 text-xs text-gray-400">
                {attachment.file_name} • {formatFileSize(attachment.file_size || 0)}
              </div>
            </div>
          );
        } else if (isVideo(attachment.file_type)) {
          const videoSrc = blobUrls[attachment.id];
          const isLoading = loadingIds.has(attachment.id);

          // Video player with thumbnail
          return (
            <div key={attachment.id} className="max-w-md">
              {isLoading ? (
                <div className="flex items-center justify-center rounded-lg border border-gray-700 bg-gray-800 h-40 w-full">
                  <div className="flex flex-col items-center gap-2 text-gray-400">
                    <svg className="h-6 w-6 animate-spin" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                    </svg>
                    <span className="text-xs">Loading video...</span>
                  </div>
                </div>
              ) : videoSrc ? (
                <video
                  controls
                  className="rounded-lg border border-gray-700 max-h-80 w-full"
                  preload="metadata"
                >
                  <source src={videoSrc} type={attachment.file_type || undefined} />
                  Your browser does not support the video tag.
                </video>
              ) : (
                <div className="flex items-center justify-center rounded-lg border border-gray-700 bg-gray-800 h-40 w-full">
                  <div className="flex flex-col items-center gap-2 text-gray-500">
                    <svg className="h-8 w-8" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 10l4.553-2.276A1 1 0 0121 8.618v6.764a1 1 0 01-1.447.894L15 14M5 18h8a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v8a2 2 0 002 2z" />
                    </svg>
                    <span className="text-xs">Video unavailable</span>
                  </div>
                </div>
              )}
              <div className="mt-1 flex items-center justify-between text-xs text-gray-400">
                <span>{attachment.file_name} • {formatFileSize(attachment.file_size || 0)}</span>
                <button
                  onClick={() => handleDownload(attachment.id, attachment.file_name)}
                  className="text-blue-400 hover:text-blue-300"
                >
                  Download
                </button>
              </div>
            </div>
          );
        } else {
          // Generic file with icon and download button
          return (
            <div
              key={attachment.id}
              className="flex items-center gap-3 rounded-lg border border-gray-700 bg-gray-800 p-3 max-w-md hover:bg-gray-750 transition-colors cursor-pointer"
              onClick={() => handleDownload(attachment.id, attachment.file_name)}
            >
              <div className="flex-shrink-0 text-gray-400">
                {getFileIcon(attachment.file_type)}
              </div>
              <div className="flex-1 min-w-0">
                <div className="text-sm text-white truncate">{attachment.file_name}</div>
                <div className="text-xs text-gray-400">{formatFileSize(attachment.file_size || 0)}</div>
              </div>
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  handleDownload(attachment.id, attachment.file_name);
                }}
                className="flex-shrink-0 rounded-lg bg-blue-600 p-2 text-white hover:bg-blue-700 transition-colors"
                title="Download"
              >
                <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
                </svg>
              </button>
            </div>
          );
        }
      })}
    </div>
  );
}
