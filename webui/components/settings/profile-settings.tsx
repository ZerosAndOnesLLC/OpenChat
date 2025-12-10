'use client';

import { User } from '@/lib/types';
import { User as UserIcon, Mail } from 'lucide-react';

interface ProfileSettingsProps {
  user: User;
}

export default function ProfileSettings({ user }: ProfileSettingsProps) {
  return (
    <div className="space-y-6">
      <p className="text-sm text-gray-400">
        View your account information. Contact your administrator to make changes.
      </p>

      <div className="bg-gray-900 rounded-xl border border-gray-800 divide-y divide-gray-800">
        {/* Display Name */}
        <div className="flex items-center gap-4 p-4">
          <div className="p-2.5 bg-blue-500/10 rounded-lg">
            <UserIcon className="w-5 h-5 text-blue-400" />
          </div>
          <div className="flex-1 min-w-0">
            <p className="text-sm text-gray-400">Display Name</p>
            <p className="text-white font-medium truncate">{user.display_name}</p>
          </div>
        </div>

        {/* Email */}
        <div className="flex items-center gap-4 p-4">
          <div className="p-2.5 bg-purple-500/10 rounded-lg">
            <Mail className="w-5 h-5 text-purple-400" />
          </div>
          <div className="flex-1 min-w-0">
            <p className="text-sm text-gray-400">Email Address</p>
            <p className="text-white font-medium truncate">{user.email}</p>
          </div>
        </div>
      </div>
    </div>
  );
}
