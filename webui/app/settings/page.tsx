'use client';

import { Suspense } from 'react';
import { useRouter } from 'next/navigation';
import { useAuth } from '@/lib/auth';
import SettingsLayout, { SettingsCategory, SettingsItem } from '@/components/settings/settings-layout';
import ProfileSettings from '@/components/settings/profile-settings';
import PrivacySettings from '@/components/settings/privacy-settings';
import DeviceManagement from '@/components/settings/devices';
import DesktopAppSettings from '@/components/settings/desktop-app-settings';
import WebhookManagement from '@/components/settings/webhooks';
import NotificationSoundSettings from '@/components/settings/notification-sound-settings';
import AboutSettings from '@/components/settings/about-settings';
import packageJson from '../../package.json';
import {
  User,
  Shield,
  Smartphone,
  Monitor,
  Webhook,
  Bell,
  Info,
  ChevronLeft,
  RefreshCw,
} from 'lucide-react';

function SettingsContent() {
  const router = useRouter();
  const { user } = useAuth();

  if (!user) {
    return (
      <div className="flex h-screen items-center justify-center bg-gray-950">
        <RefreshCw className="w-8 h-8 text-blue-400 animate-spin" />
      </div>
    );
  }

  const categories: SettingsCategory[] = [
    {
      id: 'account',
      label: 'Account',
      icon: <User className="w-4 h-4" />,
      items: [
        {
          id: 'profile',
          label: 'Profile',
          icon: <User className="w-4 h-4" />,
          component: <ProfileSettings user={user} />,
        },
        {
          id: 'privacy',
          label: 'Privacy',
          icon: <Shield className="w-4 h-4" />,
          component: <PrivacySettings user={user} />,
        },
      ],
    },
    {
      id: 'devices',
      label: 'Devices',
      icon: <Smartphone className="w-4 h-4" />,
      items: [
        {
          id: 'active-sessions',
          label: 'Active Sessions',
          icon: <Smartphone className="w-4 h-4" />,
          component: <DeviceManagement />,
        },
        {
          id: 'desktop-app',
          label: 'Desktop App',
          icon: <Monitor className="w-4 h-4" />,
          component: <DesktopAppSettings />,
        },
      ],
    },
    {
      id: 'notifications',
      label: 'Notifications',
      icon: <Bell className="w-4 h-4" />,
      items: [
        {
          id: 'notification-sounds',
          label: 'Sounds & Alerts',
          icon: <Bell className="w-4 h-4" />,
          component: <NotificationSoundSettings />,
        },
      ],
    },
    {
      id: 'integrations',
      label: 'Integrations',
      icon: <Webhook className="w-4 h-4" />,
      items: [
        {
          id: 'webhooks',
          label: 'Webhooks',
          icon: <Webhook className="w-4 h-4" />,
          component: <WebhookManagement />,
        },
      ],
    },
  ];

  const singleItems: SettingsItem[] = [
    {
      id: 'about',
      label: 'About',
      icon: <Info className="w-4 h-4" />,
      component: <AboutSettings version={packageJson.version} />,
    },
  ];

  return (
    <div className="flex h-screen flex-col bg-gray-950">
      {/* Header */}
      <header className="flex-shrink-0 border-b border-gray-800 bg-gray-900 px-4 py-3">
        <div className="flex items-center gap-4">
          <button
            onClick={() => router.back()}
            className="flex items-center gap-2 text-gray-400 hover:text-white transition-colors"
            title="Go back"
          >
            <ChevronLeft className="w-5 h-5" />
          </button>
          <h1 className="text-lg font-semibold text-white">Settings</h1>
        </div>
      </header>

      {/* Content */}
      <div className="flex-1 overflow-hidden">
        <SettingsLayout
          categories={categories}
          singleItems={singleItems}
          defaultSection="profile"
        />
      </div>
    </div>
  );
}

export default function SettingsPage() {
  return (
    <Suspense
      fallback={
        <div className="flex h-screen items-center justify-center bg-gray-950">
          <RefreshCw className="w-8 h-8 text-blue-400 animate-spin" />
        </div>
      }
    >
      <SettingsContent />
    </Suspense>
  );
}
