'use client';

interface StatusIndicatorProps {
  status: 'online' | 'offline' | 'away' | 'dnd';
  size?: 'sm' | 'md' | 'lg';
  showLabel?: boolean;
}

const STATUS_COLORS = {
  online: 'bg-green-500',
  away: 'bg-yellow-500',
  dnd: 'bg-red-500',
  offline: 'bg-gray-500',
};

const STATUS_LABELS = {
  online: 'Online',
  away: 'Away',
  dnd: 'Do Not Disturb',
  offline: 'Offline',
};

const SIZE_CLASSES = {
  sm: 'h-2 w-2',
  md: 'h-3 w-3',
  lg: 'h-4 w-4',
};

export default function StatusIndicator({ status, size = 'md', showLabel = false }: StatusIndicatorProps) {
  const colorClass = STATUS_COLORS[status];
  const sizeClass = SIZE_CLASSES[size];

  if (showLabel) {
    return (
      <div className="flex items-center gap-2">
        <div className={`${sizeClass} ${colorClass} rounded-full border-2 border-gray-900`} />
        <span className="text-sm text-gray-300">{STATUS_LABELS[status]}</span>
      </div>
    );
  }

  return (
    <div
      className={`${sizeClass} ${colorClass} rounded-full border-2 border-gray-900`}
      title={STATUS_LABELS[status]}
    />
  );
}
