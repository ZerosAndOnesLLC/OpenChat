'use client';

import React, { useEffect, useMemo, useState } from 'react';
import ReactMarkdown, { type Components, type ExtraProps, type Options } from 'react-markdown';
import remarkGfm from 'remark-gfm';
import remarkBreaks from 'remark-breaks';
import rehypeSanitize from 'rehype-sanitize';
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { vscDarkPlus } from 'react-syntax-highlighter/dist/esm/styles/prism';
import { CustomEmoji } from '@/lib/types';
import { apiClient } from '@/lib/api';

interface MarkdownRendererProps {
  content: string;
  className?: string;
}

/** Minimal shape of the hast nodes the custom-emoji pass touches. */
interface HastNode {
  type: string;
  value?: string;
  tagName?: string;
  properties?: Record<string, unknown>;
  children?: HastNode[];
}

const EMOJI_SHORTCODE = /:([a-zA-Z0-9_-]+):/g;

/** Split one text node into text + <img> nodes for known `:shortcode:` emojis. */
function splitEmojiText(value: string, emojis: Map<string, CustomEmoji>): HastNode[] | null {
  const parts: HastNode[] = [];
  let lastIndex = 0;
  let replaced = false;
  for (const match of value.matchAll(EMOJI_SHORTCODE)) {
    const emoji = emojis.get(match[1]);
    if (!emoji) continue;
    replaced = true;
    if (match.index > lastIndex) {
      parts.push({ type: 'text', value: value.slice(lastIndex, match.index) });
    }
    parts.push({
      type: 'element',
      tagName: 'img',
      properties: {
        src: apiClient.getEmojiImage(emoji.id),
        alt: match[0],
        title: match[0],
        width: 20,
        height: 20,
        className: ['inline-block', 'w-5', 'h-5', 'align-text-bottom', 'mx-0.5'],
      },
      children: [],
    });
    lastIndex = match.index + match[0].length;
  }
  if (!replaced) return null;
  if (lastIndex < value.length) {
    parts.push({ type: 'text', value: value.slice(lastIndex) });
  }
  return parts;
}

/** Replace custom-emoji shortcodes in text nodes (outside code) with images. */
function replaceEmojiNodes(node: HastNode, emojis: Map<string, CustomEmoji>) {
  if (!node.children || node.tagName === 'code' || node.tagName === 'pre') return;
  node.children = node.children.flatMap((child) => {
    if (child.type === 'text' && child.value) {
      return splitEmojiText(child.value, emojis) ?? [child];
    }
    replaceEmojiNodes(child, emojis);
    return [child];
  });
}

/** Rehype plugin: render `:name:` as the org's custom emoji image. */
function rehypeCustomEmojis(emojis: CustomEmoji[]) {
  const byName = new Map(emojis.map((e) => [e.name, e]));
  return () => (tree: HastNode) => {
    replaceEmojiNodes(tree, byName);
  };
}

/** Drop react-markdown's `node` prop so it is not forwarded to the DOM. */
function domProps<P extends ExtraProps>(props: P): Omit<P, 'node'> {
  const rest = { ...props };
  delete rest.node;
  return rest;
}

const markdownComponents: Components = {
  // Custom code block rendering with syntax highlighting
  code(props) {
    const { className, children, ...rest } = domProps(props);
    const match = /language-(\w+)/.exec(className || '');
    return match ? (
      <SyntaxHighlighter
        style={vscDarkPlus}
        language={match[1]}
        PreTag="div"
        customStyle={{
          margin: 0,
          borderRadius: '0.375rem',
          fontSize: '0.875rem',
        }}
      >
        {String(children).replace(/\n$/, '')}
      </SyntaxHighlighter>
    ) : (
      <code
        className="rounded bg-gray-800 px-1.5 py-0.5 text-sm text-pink-400"
        {...rest}
      >
        {children}
      </code>
    );
  },
  // Custom link rendering with target blank
  a(props) {
    const { children, ...rest } = domProps(props);
    return (
      <a
        className="text-blue-400 hover:underline"
        target="_blank"
        rel="noopener noreferrer"
        {...rest}
      >
        {children}
      </a>
    );
  },
  // Custom blockquote styling
  blockquote(props) {
    const { children, ...rest } = domProps(props);
    return (
      <blockquote
        className="border-l-4 border-gray-600 pl-4 italic text-gray-300"
        {...rest}
      >
        {children}
      </blockquote>
    );
  },
  // Custom heading styling
  h1(props) {
    const { children, ...rest } = domProps(props);
    return (
      <h1 className="text-xl font-bold text-white" {...rest}>
        {children}
      </h1>
    );
  },
  h2(props) {
    const { children, ...rest } = domProps(props);
    return (
      <h2 className="text-lg font-bold text-white" {...rest}>
        {children}
      </h2>
    );
  },
  h3(props) {
    const { children, ...rest } = domProps(props);
    return (
      <h3 className="text-base font-bold text-white" {...rest}>
        {children}
      </h3>
    );
  },
  // Custom list styling
  ul(props) {
    const { children, ...rest } = domProps(props);
    return (
      <ul className="list-disc pl-6 text-gray-200" {...rest}>
        {children}
      </ul>
    );
  },
  ol(props) {
    const { children, ...rest } = domProps(props);
    return (
      <ol className="list-decimal pl-6 text-gray-200" {...rest}>
        {children}
      </ol>
    );
  },
  // Custom paragraph styling
  p(props) {
    const { children, ...rest } = domProps(props);
    return (
      <p className="text-gray-200" {...rest}>
        {children}
      </p>
    );
  },
};

export default function MarkdownRenderer({ content, className = '' }: MarkdownRendererProps) {
  const [customEmojis, setCustomEmojis] = useState<CustomEmoji[]>([]);

  useEffect(() => {
    let cancelled = false;
    apiClient.getCustomEmojis()
      .then((emojis) => {
        if (!cancelled) setCustomEmojis(emojis);
      })
      .catch((error) => console.error('Failed to load custom emojis:', error));
    return () => {
      cancelled = true;
    };
  }, []);

  // Sanitize first, then inject the (trusted, app-generated) emoji images.
  const rehypePlugins = useMemo<Options['rehypePlugins']>(
    () => [rehypeSanitize, rehypeCustomEmojis(customEmojis)],
    [customEmojis],
  );

  return (
    <div className={`prose prose-invert prose-sm max-w-none ${className}`}>
      <ReactMarkdown
        remarkPlugins={[remarkGfm, remarkBreaks]}
        rehypePlugins={rehypePlugins}
        components={markdownComponents}
      >
        {content}
      </ReactMarkdown>
    </div>
  );
}
