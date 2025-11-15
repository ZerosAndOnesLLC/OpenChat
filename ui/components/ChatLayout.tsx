'use client';

import { useState } from 'react';
import Sidebar from './Sidebar';
import MessageArea from './MessageArea';
import type { Channel, DirectMessage } from '@/lib/types';

export default function ChatLayout() {
  const [activeChannel, setActiveChannel] = useState<Channel | null>(null);
  const [activeDm, setActiveDm] = useState<DirectMessage | null>(null);

  const handleSelectChannel = (channel: Channel) => {
    setActiveChannel(channel);
    setActiveDm(null);
  };

  const handleSelectDm = (dm: DirectMessage) => {
    setActiveDm(dm);
    setActiveChannel(null);
  };

  return (
    <div className="flex h-screen bg-gray-100">
      <Sidebar
        activeChannel={activeChannel}
        activeDm={activeDm}
        onSelectChannel={handleSelectChannel}
        onSelectDm={handleSelectDm}
      />
      <MessageArea
        channel={activeChannel}
        dm={activeDm}
      />
    </div>
  );
}
