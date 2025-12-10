'use client';

import { Package, Globe, Heart } from 'lucide-react';

interface AboutSettingsProps {
  version: string;
}

export default function AboutSettings({ version }: AboutSettingsProps) {
  return (
    <div className="space-y-6">
      <p className="text-sm text-gray-400">
        Application information and version details.
      </p>

      <div className="bg-gray-900 rounded-xl border border-gray-800 divide-y divide-gray-800">
        {/* Version */}
        <div className="flex items-center gap-4 p-4">
          <div className="p-2.5 bg-green-500/10 rounded-lg">
            <Package className="w-5 h-5 text-green-400" />
          </div>
          <div className="flex-1">
            <p className="text-sm text-gray-400">Version</p>
            <p className="text-white font-mono font-medium">{version}</p>
          </div>
        </div>

        {/* Build Info */}
        <div className="flex items-center gap-4 p-4">
          <div className="p-2.5 bg-purple-500/10 rounded-lg">
            <Globe className="w-5 h-5 text-purple-400" />
          </div>
          <div className="flex-1">
            <p className="text-sm text-gray-400">Platform</p>
            <p className="text-white font-medium">Web Application</p>
          </div>
        </div>
      </div>

      {/* Footer */}
      <div className="bg-gray-900/50 rounded-xl border border-gray-800 p-5 text-center">
        <div className="flex items-center justify-center gap-2 text-gray-400">
          <span className="text-sm">Made with</span>
          <Heart className="w-4 h-4 text-red-400 fill-red-400" />
          <span className="text-sm">by the OpenChat Team</span>
        </div>
      </div>
    </div>
  );
}
