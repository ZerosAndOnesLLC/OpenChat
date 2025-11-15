'use client';

interface TypingIndicatorProps {
  users: string[];
}

export default function TypingIndicator({ users }: TypingIndicatorProps) {
  if (users.length === 0) return null;

  const getText = () => {
    if (users.length === 1) {
      return `${users[0]} is typing...`;
    } else if (users.length === 2) {
      return `${users[0]} and ${users[1]} are typing...`;
    } else {
      return `${users[0]}, ${users[1]}, and ${users.length - 2} other${
        users.length - 2 > 1 ? 's' : ''
      } are typing...`;
    }
  };

  return (
    <div className="px-6 py-2">
      <div className="flex items-center gap-2 text-sm text-gray-500">
        <div className="flex gap-1">
          <span className="animate-bounce">•</span>
          <span className="animate-bounce animation-delay-200">•</span>
          <span className="animate-bounce animation-delay-400">•</span>
        </div>
        <span>{getText()}</span>
      </div>
    </div>
  );
}
