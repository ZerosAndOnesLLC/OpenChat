'use client';

interface MentionHighlightProps {
  content: string;
  currentUserId?: string;
}

export default function MentionHighlight({ content, currentUserId }: MentionHighlightProps) {
  const highlightMentions = (text: string) => {
    const mentionRegex = /(@\w+|@channel|@here)/g;
    const parts = text.split(mentionRegex);

    return parts.map((part, index) => {
      if (part.match(mentionRegex)) {
        const isCurrentUser = currentUserId && part === `@${currentUserId}`;
        const isChannel = part === '@channel' || part === '@here';

        return (
          <span
            key={index}
            className={`font-semibold rounded px-1 ${
              isCurrentUser
                ? 'bg-yellow-200 dark:bg-yellow-800 text-yellow-900 dark:text-yellow-100'
                : isChannel
                ? 'bg-blue-200 dark:bg-blue-800 text-blue-900 dark:text-blue-100'
                : 'bg-gray-200 dark:bg-gray-700 text-blue-600 dark:text-blue-400'
            }`}
          >
            {part}
          </span>
        );
      }
      return part;
    });
  };

  return <>{highlightMentions(content)}</>;
}
