'use client';

interface MarkdownToolbarProps {
  onFormat: (before: string, after: string, placeholder?: string) => void;
  onTogglePreview: () => void;
  showPreview: boolean;
}

export default function MarkdownToolbar({ onFormat, onTogglePreview, showPreview }: MarkdownToolbarProps) {
  const buttons = [
    {
      title: 'Bold',
      icon: 'B',
      action: () => onFormat('**', '**', 'bold text'),
      className: 'font-bold',
    },
    {
      title: 'Italic',
      icon: 'I',
      action: () => onFormat('*', '*', 'italic text'),
      className: 'italic',
    },
    {
      title: 'Code',
      icon: '</>',
      action: () => onFormat('`', '`', 'code'),
      className: 'font-mono text-xs',
    },
    {
      title: 'Code Block',
      icon: '{}',
      action: () => onFormat('```\n', '\n```', 'code block'),
      className: 'font-mono text-xs',
    },
    {
      title: 'Link',
      icon: '🔗',
      action: () => onFormat('[', '](url)', 'link text'),
      className: '',
    },
    {
      title: 'Quote',
      icon: '❝',
      action: () => onFormat('> ', '', 'quote'),
      className: 'text-lg',
    },
    {
      title: 'Bulleted List',
      icon: '•',
      action: () => onFormat('- ', '', 'list item'),
      className: 'text-lg',
    },
    {
      title: 'Numbered List',
      icon: '1.',
      action: () => onFormat('1. ', '', 'list item'),
      className: 'text-xs',
    },
  ];

  return (
    <div className="flex items-center gap-1 border-b border-gray-700 bg-gray-900 px-2 py-1">
      {buttons.map((btn, idx) => (
        <button
          key={idx}
          type="button"
          onClick={btn.action}
          className={`rounded px-2 py-1 text-gray-300 hover:bg-gray-800 hover:text-white ${btn.className}`}
          title={btn.title}
        >
          {btn.icon}
        </button>
      ))}

      <div className="ml-auto flex items-center gap-2">
        <button
          type="button"
          onClick={onTogglePreview}
          className={`rounded px-3 py-1 text-xs transition-colors ${
            showPreview
              ? 'bg-blue-600 text-white'
              : 'bg-gray-800 text-gray-300 hover:bg-gray-700'
          }`}
          title="Toggle preview"
        >
          {showPreview ? 'Edit' : 'Preview'}
        </button>
      </div>
    </div>
  );
}
