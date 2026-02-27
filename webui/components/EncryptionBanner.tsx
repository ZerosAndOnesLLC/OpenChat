'use client';

import { Shield } from 'lucide-react';

export default function EncryptionBanner() {
  return (
    <div className="flex items-center justify-center gap-2 border-b border-gray-800 bg-gray-900/50 px-4 py-2">
      <Shield className="h-3.5 w-3.5 text-green-400" />
      <span className="text-xs text-gray-400">
        Messages in this channel are end-to-end encrypted
      </span>
    </div>
  );
}
