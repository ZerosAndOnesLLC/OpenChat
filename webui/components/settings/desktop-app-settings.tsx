'use client';

import DesktopLogin from '@/components/desktop-login';

export default function DesktopAppSettings() {
  return (
    <div className="space-y-6">
      <p className="text-sm text-gray-400">
        Connect the OpenChat desktop application to your account for a native experience.
      </p>

      <div className="-mx-2">
        <DesktopLogin />
      </div>
    </div>
  );
}
