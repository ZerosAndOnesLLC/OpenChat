'use client';

import { useState, useCallback } from 'react';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import { apiClient } from '@/lib/api';
import { useWebSocketStore } from '@/lib/websocket';
import type { Channel, ChannelSection } from '@/lib/types';
import ContextMenu from './ContextMenu';
import ChannelList from './ChannelList';

interface ChannelSectionListProps {
  channels: Channel[];
  activeChannel: Channel | null;
  onSelectChannel: (channel: Channel) => void;
  onLeaveChannel?: (channelId: string) => void;
}

export default function ChannelSectionList({
  channels,
  activeChannel,
  onSelectChannel,
  onLeaveChannel,
}: ChannelSectionListProps) {
  const queryClient = useQueryClient();
  const [contextMenu, setContextMenu] = useState<{
    x: number;
    y: number;
    sectionId: string;
    sectionName: string;
  } | null>(null);
  const [renamingId, setRenamingId] = useState<string | null>(null);
  const [renameValue, setRenameValue] = useState('');

  const { data: sections = [], isLoading } = useQuery({
    queryKey: ['channel-sections'],
    queryFn: () => apiClient.listChannelSections(),
  });

  const handleContextMenu = useCallback((e: React.MouseEvent, sectionId: string, sectionName: string) => {
    e.preventDefault();
    setContextMenu({ x: e.clientX, y: e.clientY, sectionId, sectionName });
  }, []);

  const handleToggleCollapse = async (section: ChannelSection) => {
    // Optimistic update
    queryClient.setQueryData<ChannelSection[]>(['channel-sections'], (old) =>
      old?.map(s => s.id === section.id ? { ...s, collapsed: !s.collapsed } : s)
    );
    try {
      await apiClient.updateChannelSection(section.id, { collapsed: !section.collapsed });
    } catch {
      queryClient.invalidateQueries({ queryKey: ['channel-sections'] });
    }
  };

  const handleRename = async (sectionId: string) => {
    if (!renameValue.trim()) {
      setRenamingId(null);
      return;
    }
    try {
      await apiClient.updateChannelSection(sectionId, { name: renameValue });
      queryClient.invalidateQueries({ queryKey: ['channel-sections'] });
    } catch (err) {
      console.error('Failed to rename section:', err);
    }
    setRenamingId(null);
  };

  const handleDelete = async (sectionId: string) => {
    try {
      await apiClient.deleteChannelSection(sectionId);
      queryClient.invalidateQueries({ queryKey: ['channel-sections'] });
    } catch (err) {
      console.error('Failed to delete section:', err);
    }
  };

  // Build a set of all channel IDs assigned to any section
  const assignedChannelIds = new Set(sections.flatMap(s => s.channel_ids));

  // Channels not in any section
  const unsectionedChannels = channels.filter(ch => !assignedChannelIds.has(ch.id));

  if (isLoading) {
    return <div className="px-2 py-2 text-xs text-gray-500">Loading sections...</div>;
  }

  return (
    <div className="space-y-1">
      {sections.map((section) => {
        const sectionChannels = section.channel_ids
          .map(id => channels.find(ch => ch.id === id))
          .filter((ch): ch is Channel => ch !== undefined);

        return (
          <div key={section.id}>
            {/* Section Header */}
            <div
              className="group flex items-center justify-between px-2 py-1 cursor-pointer hover:bg-gray-800 rounded"
              onClick={() => handleToggleCollapse(section)}
              onContextMenu={(e) => handleContextMenu(e, section.id, section.name)}
            >
              <div className="flex items-center gap-1 min-w-0">
                <svg
                  className={`h-3 w-3 flex-shrink-0 text-gray-500 transition-transform ${
                    section.collapsed ? '' : 'rotate-90'
                  }`}
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
                </svg>
                {renamingId === section.id ? (
                  <input
                    type="text"
                    value={renameValue}
                    onChange={(e) => setRenameValue(e.target.value)}
                    onBlur={() => handleRename(section.id)}
                    onKeyDown={(e) => {
                      if (e.key === 'Enter') handleRename(section.id);
                      if (e.key === 'Escape') setRenamingId(null);
                    }}
                    className="flex-1 bg-gray-800 px-1 py-0 text-xs text-white rounded focus:outline-none focus:ring-1 focus:ring-blue-500"
                    autoFocus
                    onClick={(e) => e.stopPropagation()}
                  />
                ) : (
                  <span className="text-xs font-semibold text-gray-400 uppercase truncate">
                    {section.name}
                  </span>
                )}
              </div>
              <span className="text-[10px] text-gray-600 opacity-0 group-hover:opacity-100">
                {sectionChannels.length}
              </span>
            </div>

            {/* Section Channels */}
            {!section.collapsed && (
              <div className="ml-2">
                <ChannelList
                  channels={sectionChannels}
                  activeChannel={activeChannel}
                  onSelectChannel={onSelectChannel}
                  onLeaveChannel={onLeaveChannel}
                />
              </div>
            )}
          </div>
        );
      })}

      {/* Unsectioned channels */}
      {unsectionedChannels.length > 0 && (
        <div>
          <div className="flex items-center px-2 py-1">
            <span className="text-xs font-semibold text-gray-500 uppercase">Other</span>
          </div>
          <ChannelList
            channels={unsectionedChannels}
            activeChannel={activeChannel}
            onSelectChannel={onSelectChannel}
            onLeaveChannel={onLeaveChannel}
          />
        </div>
      )}

      {channels.length === 0 && (
        <p className="px-2 py-2 text-xs text-gray-500">No channels yet</p>
      )}

      {contextMenu && (
        <ContextMenu
          x={contextMenu.x}
          y={contextMenu.y}
          onClose={() => setContextMenu(null)}
          items={[
            {
              label: 'Rename',
              onClick: () => {
                setRenameValue(contextMenu.sectionName);
                setRenamingId(contextMenu.sectionId);
              },
            },
            {
              label: 'Delete',
              danger: true,
              onClick: () => handleDelete(contextMenu.sectionId),
            },
          ]}
        />
      )}
    </div>
  );
}
