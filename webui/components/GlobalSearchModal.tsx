'use client';

import { useRouter } from 'next/navigation';
import SearchBar from './SearchBar';

interface GlobalSearchModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export default function GlobalSearchModal({ isOpen, onClose }: GlobalSearchModalProps) {
  const router = useRouter();

  if (!isOpen) return null;

  const handleSearch = (query: string) => {
    router.push(`/search?q=${encodeURIComponent(query)}`);
    onClose();
  };

  return (
    <>
      <div
        className="fixed inset-0 bg-black bg-opacity-50 z-50 flex items-start justify-center pt-20 animate-fade-in"
        onClick={onClose}
      >
        <div
          className="bg-white dark:bg-gray-800 rounded-lg p-4 w-full max-w-2xl mx-4 shadow-xl animate-modal-in"
          onClick={(e) => e.stopPropagation()}
        >
          <SearchBar onSearch={handleSearch} autoFocus onClose={onClose} />
          <div className="mt-4 text-xs text-gray-500 dark:text-gray-400">
            <div className="flex items-center justify-between">
              <span>Press <kbd className="px-2 py-1 bg-gray-100 dark:bg-gray-700 rounded">Esc</kbd> to close</span>
              <span>Press <kbd className="px-2 py-1 bg-gray-100 dark:bg-gray-700 rounded">Enter</kbd> to search</span>
            </div>
          </div>
        </div>
      </div>
    </>
  );
}
