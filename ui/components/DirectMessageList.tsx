'use client';

import type { DirectMessage } from '@/lib/types';

interface DirectMessageListProps {
  dms: DirectMessage[];
  activeDm: DirectMessage | null;
  onSelectDm: (dm: DirectMessage) => void;
}

export default function DirectMessageList({
  dms,
  activeDm,
  onSelectDm,
}: DirectMessageListProps) {
  const getDmName = (dm: DirectMessage) => {
    if (!dm.participants || dm.participants.length === 0) {
      return 'Unknown';
    }
    return dm.participants.map((p) => p.display_name).join(', ');
  };

  return (
    <div className="space-y-1">
      {dms.map((dm) => (
        <button
          key={dm.id}
          onClick={() => onSelectDm(dm)}
          className={`w-full rounded px-2 py-1.5 text-left text-sm transition-colors ${
            activeDm?.id === dm.id
              ? 'bg-blue-600 text-white'
              : 'text-gray-300 hover:bg-gray-800'
          }`}
        >
          <div className="flex items-center">
            <span className="mr-1.5">💬</span>
            <span className="truncate">{getDmName(dm)}</span>
          </div>
        </button>
      ))}
      {dms.length === 0 && (
        <p className="px-2 py-2 text-xs text-gray-500">No direct messages yet</p>
      )}
    </div>
  );
}
