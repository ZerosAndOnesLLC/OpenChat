'use client';

interface ChannelMentionDialogProps {
  isOpen: boolean;
  memberCount: number;
  onConfirm: () => void;
  onCancel: () => void;
}

export default function ChannelMentionDialog({
  isOpen,
  memberCount,
  onConfirm,
  onCancel
}: ChannelMentionDialogProps) {
  if (!isOpen) return null;

  return (
    <>
      <div
        className="fixed inset-0 bg-black bg-opacity-50 z-50 flex items-center justify-center animate-fade-in"
        onClick={onCancel}
      >
        <div
          className="bg-white dark:bg-gray-800 rounded-lg p-6 max-w-md w-full mx-4 shadow-xl animate-modal-in"
          onClick={(e) => e.stopPropagation()}
        >
          <div className="mb-4">
            <h2 className="text-xl font-bold text-gray-900 dark:text-gray-100 mb-2">
              Notify Channel Members?
            </h2>
            <p className="text-gray-600 dark:text-gray-400">
              Using @channel will notify all <strong>{memberCount}</strong> members in this channel.
              They will receive a notification even if they have muted this channel.
            </p>
          </div>

          <div className="bg-yellow-50 dark:bg-yellow-900 dark:bg-opacity-20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-3 mb-4">
            <div className="flex gap-2">
              <span className="text-yellow-600 dark:text-yellow-500">⚠️</span>
              <p className="text-sm text-yellow-800 dark:text-yellow-300">
                Use @channel sparingly for important announcements that require everyone's attention.
              </p>
            </div>
          </div>

          <div className="flex gap-3 justify-end">
            <button
              onClick={onCancel}
              className="px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-600 transition-colors"
            >
              Cancel
            </button>
            <button
              onClick={onConfirm}
              className="px-4 py-2 text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 rounded-lg transition-colors"
            >
              Notify Everyone
            </button>
          </div>
        </div>
      </div>
    </>
  );
}
