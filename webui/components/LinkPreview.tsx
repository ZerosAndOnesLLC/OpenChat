'use client';

import { useState, useEffect, useRef } from 'react';
import { apiClient } from '@/lib/api';
import type { LinkPreview as LinkPreviewType } from '@/lib/types';

interface LinkPreviewProps {
  url: string;
}

// Global in-memory cache for link previews
const linkPreviewCache = new Map<string, LinkPreviewType | null>();

export default function LinkPreview({ url }: LinkPreviewProps) {
  const [preview, setPreview] = useState<LinkPreviewType | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(false);
  const [isVisible, setIsVisible] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);
  const linkRef = useRef<HTMLAnchorElement>(null);

  // Intersection observer for lazy loading
  useEffect(() => {
    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            setIsVisible(true);
            observer.disconnect();
          }
        });
      },
      { rootMargin: '50px' }
    );

    if (containerRef.current) {
      observer.observe(containerRef.current);
    }

    return () => {
      observer.disconnect();
    };
  }, []);

  useEffect(() => {
    if (!isVisible) return;

    // Check cache first
    if (linkPreviewCache.has(url)) {
      const cached = linkPreviewCache.get(url);
      setPreview(cached || null);
      return;
    }

    const fetchPreview = async () => {
      try {
        setLoading(true);
        setError(false);
        const data = await apiClient.getLinkPreview(url);
        linkPreviewCache.set(url, data);
        setPreview(data);
      } catch (err) {
        console.error('Failed to fetch link preview:', err);
        linkPreviewCache.set(url, null);
        setError(true);
      } finally {
        setLoading(false);
      }
    };

    fetchPreview();
  }, [url, isVisible]);

  if (!isVisible) {
    return <div ref={containerRef} className="h-4" />;
  }

  if (loading) {
    return (
      <div ref={containerRef} className="mt-2 animate-pulse rounded-lg border border-gray-700 bg-gray-900 p-3">
        <div className="h-4 w-3/4 rounded bg-gray-700"></div>
        <div className="mt-2 h-3 w-full rounded bg-gray-700"></div>
        <div className="mt-1 h-3 w-2/3 rounded bg-gray-700"></div>
      </div>
    );
  }

  if (error || !preview || (!preview.title && !preview.description && !preview.image)) {
    return null;
  }

  return (
    <a
      ref={linkRef}
      href={url}
      target="_blank"
      rel="noopener noreferrer"
      className="mt-2 block rounded-lg border border-gray-700 bg-gray-900 transition-colors hover:border-gray-600 hover:bg-gray-800"
    >
      <div className="flex gap-3 p-3">
        {preview.image && (
          <div className="flex-shrink-0">
            <img
              src={preview.image}
              alt={preview.title || 'Link preview'}
              className="h-20 w-20 rounded object-cover"
              onError={(e) => {
                e.currentTarget.style.display = 'none';
              }}
            />
          </div>
        )}
        <div className="flex-1 overflow-hidden">
          {preview.title && (
            <div className="font-semibold text-blue-400 line-clamp-2">
              {preview.title}
            </div>
          )}
          {preview.description && (
            <div className="mt-1 text-sm text-gray-400 line-clamp-2">
              {preview.description}
            </div>
          )}
          {preview.site_name && (
            <div className="mt-1 text-xs text-gray-500">
              {preview.site_name}
            </div>
          )}
          <div className="mt-1 text-xs text-gray-600">
            {new URL(url).hostname}
          </div>
        </div>
      </div>
    </a>
  );
}
