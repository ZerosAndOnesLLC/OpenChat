'use client';

import React, { useEffect, useState } from 'react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import rehypeSanitize from 'rehype-sanitize';
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { vscDarkPlus } from 'react-syntax-highlighter/dist/esm/styles/prism';
import { CustomEmoji } from '@/lib/types';
import { apiClient } from '@/lib/api';

interface MarkdownRendererProps {
  content: string;
  className?: string;
}

export default function MarkdownRenderer({ content, className = '' }: MarkdownRendererProps) {
  const [customEmojis, setCustomEmojis] = useState<CustomEmoji[]>([]);
  const [processedContent, setProcessedContent] = useState(content);

  useEffect(() => {
    loadCustomEmojis();
  }, []);

  useEffect(() => {
    if (customEmojis.length > 0) {
      setProcessedContent(replaceCustomEmojis(content));
    } else {
      setProcessedContent(content);
    }
  }, [content, customEmojis]);

  const loadCustomEmojis = async () => {
    try {
      const emojis = await apiClient.getCustomEmojis();
      setCustomEmojis(emojis);
    } catch (error) {
      console.error('Failed to load custom emojis:', error);
    }
  };

  const replaceCustomEmojis = (text: string): string => {
    let result = text;
    const emojiRegex = /:([a-zA-Z0-9_-]+):/g;

    result = result.replace(emojiRegex, (match, emojiName) => {
      const customEmoji = customEmojis.find((e) => e.name === emojiName);
      if (customEmoji) {
        // Use a special marker that we'll replace in the text component
        return `__CUSTOM_EMOJI__${customEmoji.id}__${emojiName}__`;
      }
      return match;
    });

    return result;
  };

  return (
    <div className={`prose prose-invert prose-sm max-w-none ${className}`}>
      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        rehypePlugins={[rehypeSanitize]}
        components={{
        // Custom text rendering to handle custom emojis
        text({ node, children, ...props }: any) {
          const text = String(children);
          const emojiMarkerRegex = /__CUSTOM_EMOJI__([^_]+)__([^_]+)__/g;
          const parts: (string | React.ReactElement)[] = [];
          let lastIndex = 0;
          let match;

          while ((match = emojiMarkerRegex.exec(text)) !== null) {
            const [fullMatch, emojiId, emojiName] = match;
            const matchIndex = match.index;

            // Add text before the emoji
            if (matchIndex > lastIndex) {
              parts.push(text.substring(lastIndex, matchIndex));
            }

            // Render custom emoji image
            parts.push(
              <img
                key={`emoji-${emojiId}-${matchIndex}`}
                src={apiClient.getEmojiImage(emojiId)}
                alt={`:${emojiName}:`}
                title={`:${emojiName}:`}
                className="inline-block w-5 h-5 align-text-bottom mx-0.5"
              />
            );

            lastIndex = emojiMarkerRegex.lastIndex;
          }

          // Add remaining text
          if (lastIndex < text.length) {
            parts.push(text.substring(lastIndex));
          }

          return parts.length > 0 ? <>{parts}</> : <>{children}</>;
        },
        // Custom code block rendering with syntax highlighting
        code({ node, inline, className, children, ...props }: any) {
          const match = /language-(\w+)/.exec(className || '');
          return !inline && match ? (
            <SyntaxHighlighter
              style={vscDarkPlus}
              language={match[1]}
              PreTag="div"
              customStyle={{
                margin: 0,
                borderRadius: '0.375rem',
                fontSize: '0.875rem',
              }}
              {...props}
            >
              {String(children).replace(/\n$/, '')}
            </SyntaxHighlighter>
          ) : (
            <code
              className="rounded bg-gray-800 px-1.5 py-0.5 text-sm text-pink-400"
              {...props}
            >
              {children}
            </code>
          );
        },
        // Custom link rendering with target blank
        a({ node, children, ...props }: any) {
          return (
            <a
              className="text-blue-400 hover:underline"
              target="_blank"
              rel="noopener noreferrer"
              {...props}
            >
              {children}
            </a>
          );
        },
        // Custom blockquote styling
        blockquote({ node, children, ...props }: any) {
          return (
            <blockquote
              className="border-l-4 border-gray-600 pl-4 italic text-gray-300"
              {...props}
            >
              {children}
            </blockquote>
          );
        },
        // Custom heading styling
        h1({ node, children, ...props }: any) {
          return (
            <h1 className="text-xl font-bold text-white" {...props}>
              {children}
            </h1>
          );
        },
        h2({ node, children, ...props }: any) {
          return (
            <h2 className="text-lg font-bold text-white" {...props}>
              {children}
            </h2>
          );
        },
        h3({ node, children, ...props }: any) {
          return (
            <h3 className="text-base font-bold text-white" {...props}>
              {children}
            </h3>
          );
        },
        // Custom list styling
        ul({ node, children, ...props }: any) {
          return (
            <ul className="list-disc pl-6 text-gray-200" {...props}>
              {children}
            </ul>
          );
        },
        ol({ node, children, ...props }: any) {
          return (
            <ol className="list-decimal pl-6 text-gray-200" {...props}>
              {children}
            </ol>
          );
        },
        // Custom paragraph styling
        p({ node, children, ...props }: any) {
          return (
            <p className="text-gray-200" {...props}>
              {children}
            </p>
          );
        },
        }}
      >
        {processedContent}
      </ReactMarkdown>
    </div>
  );
}
