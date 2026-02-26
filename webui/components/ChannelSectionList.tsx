'use client';

import { useState, useCallback, useMemo } from 'react';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import {
  DndContext,
  DragOverlay,
  closestCenter,
  KeyboardSensor,
  PointerSensor,
  useSensor,
  useSensors,
  type DragStartEvent,
  type DragEndEvent,
} from '@dnd-kit/core';
import {
  SortableContext,
  sortableKeyboardCoordinates,
  verticalListSortingStrategy,
  useSortable,
} from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';
import { apiClient } from '@/lib/api';
import type { Channel, ChannelSection } from '@/lib/types';
import ContextMenu from './ContextMenu';
import ChannelList from './ChannelList';

interface ChannelSectionListProps {
  channels: Channel[];
  activeChannel: Channel | null;
  onSelectChannel: (channel: Channel) => void;
  onLeaveChannel?: (channelId: string) => void;
}

function SortableSectionHeader({
  section,
  channelCount,
  isRenaming,
  renameValue,
  onRenameChange,
  onRenameSubmit,
  onRenameCancel,
  onToggleCollapse,
  onContextMenu,
}: {
  section: ChannelSection;
  channelCount: number;
  isRenaming: boolean;
  renameValue: string;
  onRenameChange: (val: string) => void;
  onRenameSubmit: () => void;
  onRenameCancel: () => void;
  onToggleCollapse: () => void;
  onContextMenu: (e: React.MouseEvent) => void;
}) {
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({ id: `section-${section.id}` });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.5 : 1,
  };

  return (
    <div ref={setNodeRef} style={style}>
      <div
        className="group flex items-center justify-between px-2 py-1 cursor-pointer hover:bg-gray-800 rounded"
        onClick={onToggleCollapse}
        onContextMenu={onContextMenu}
      >
        <div className="flex items-center gap-1 min-w-0">
          {/* Drag handle */}
          <div
            {...attributes}
            {...listeners}
            className="flex-shrink-0 cursor-grab opacity-0 group-hover:opacity-100 text-gray-600 hover:text-gray-400 mr-0.5"
            onClick={(e) => e.stopPropagation()}
          >
            <svg className="h-3 w-3" viewBox="0 0 20 20" fill="currentColor">
              <path d="M7 2a2 2 0 1 0 0 4 2 2 0 0 0 0-4zM13 2a2 2 0 1 0 0 4 2 2 0 0 0 0-4zM7 8a2 2 0 1 0 0 4 2 2 0 0 0 0-4zM13 8a2 2 0 1 0 0 4 2 2 0 0 0 0-4zM7 14a2 2 0 1 0 0 4 2 2 0 0 0 0-4zM13 14a2 2 0 1 0 0 4 2 2 0 0 0 0-4z" />
            </svg>
          </div>
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
          {isRenaming ? (
            <input
              type="text"
              value={renameValue}
              onChange={(e) => onRenameChange(e.target.value)}
              onBlur={onRenameSubmit}
              onKeyDown={(e) => {
                if (e.key === 'Enter') onRenameSubmit();
                if (e.key === 'Escape') onRenameCancel();
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
          {channelCount}
        </span>
      </div>
    </div>
  );
}

function SortableChannelItem({
  channel,
  activeChannel,
  onSelectChannel,
  onLeaveChannel,
}: {
  channel: Channel;
  activeChannel: Channel | null;
  onSelectChannel: (channel: Channel) => void;
  onLeaveChannel?: (channelId: string) => void;
}) {
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({ id: `channel-${channel.id}` });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.5 : 1,
  };

  return (
    <div ref={setNodeRef} style={style} className="group/drag flex items-center">
      <div
        {...attributes}
        {...listeners}
        className="flex-shrink-0 cursor-grab opacity-0 group-hover/drag:opacity-100 text-gray-600 hover:text-gray-400 pl-1"
      >
        <svg className="h-3 w-3" viewBox="0 0 20 20" fill="currentColor">
          <path d="M7 2a2 2 0 1 0 0 4 2 2 0 0 0 0-4zM13 2a2 2 0 1 0 0 4 2 2 0 0 0 0-4zM7 8a2 2 0 1 0 0 4 2 2 0 0 0 0-4zM13 8a2 2 0 1 0 0 4 2 2 0 0 0 0-4zM7 14a2 2 0 1 0 0 4 2 2 0 0 0 0-4zM13 14a2 2 0 1 0 0 4 2 2 0 0 0 0-4z" />
        </svg>
      </div>
      <div className="flex-1">
        <ChannelList
          channels={[channel]}
          activeChannel={activeChannel}
          onSelectChannel={onSelectChannel}
          onLeaveChannel={onLeaveChannel}
        />
      </div>
    </div>
  );
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
  const [activeDragId, setActiveDragId] = useState<string | null>(null);

  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 5 } }),
    useSensor(KeyboardSensor, { coordinateGetter: sortableKeyboardCoordinates }),
  );

  const { data: sections = [], isLoading } = useQuery({
    queryKey: ['channel-sections'],
    queryFn: () => apiClient.listChannelSections(),
  });

  const sectionIds = useMemo(() => sections.map(s => `section-${s.id}`), [sections]);

  const handleContextMenu = useCallback((e: React.MouseEvent, sectionId: string, sectionName: string) => {
    e.preventDefault();
    setContextMenu({ x: e.clientX, y: e.clientY, sectionId, sectionName });
  }, []);

  const handleToggleCollapse = async (section: ChannelSection) => {
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

  const handleDragStart = (event: DragStartEvent) => {
    setActiveDragId(event.active.id as string);
  };

  const handleDragEnd = async (event: DragEndEvent) => {
    setActiveDragId(null);
    const { active, over } = event;
    if (!over || active.id === over.id) return;

    const activeIdStr = active.id as string;
    const overIdStr = over.id as string;

    // Section reorder
    if (activeIdStr.startsWith('section-') && overIdStr.startsWith('section-')) {
      const activeSectionId = activeIdStr.replace('section-', '');
      const overSectionId = overIdStr.replace('section-', '');

      const oldIndex = sections.findIndex(s => s.id === activeSectionId);
      const newIndex = sections.findIndex(s => s.id === overSectionId);
      if (oldIndex === -1 || newIndex === -1) return;

      // Optimistic reorder
      const newSections = [...sections];
      const [moved] = newSections.splice(oldIndex, 1);
      newSections.splice(newIndex, 0, moved);

      const reordered = newSections.map((s, i) => ({ ...s, position: i }));
      queryClient.setQueryData<ChannelSection[]>(['channel-sections'], reordered);

      try {
        await apiClient.reorderSections({
          order: reordered.map(s => ({ id: s.id, position: s.position })),
        });
      } catch {
        queryClient.invalidateQueries({ queryKey: ['channel-sections'] });
      }
      return;
    }

    // Channel reorder within or across sections
    if (activeIdStr.startsWith('channel-') && overIdStr.startsWith('channel-')) {
      const activeChannelId = activeIdStr.replace('channel-', '');
      const overChannelId = overIdStr.replace('channel-', '');

      // Find which sections contain each channel
      const fromSection = sections.find(s => s.channel_ids.includes(activeChannelId));
      const toSection = sections.find(s => s.channel_ids.includes(overChannelId));

      if (!fromSection || !toSection) return;

      if (fromSection.id === toSection.id) {
        // Same section reorder
        const oldIds = [...fromSection.channel_ids];
        const oldIndex = oldIds.indexOf(activeChannelId);
        const newIndex = oldIds.indexOf(overChannelId);
        if (oldIndex === -1 || newIndex === -1) return;

        const [moved] = oldIds.splice(oldIndex, 1);
        oldIds.splice(newIndex, 0, moved);

        // Optimistic update
        queryClient.setQueryData<ChannelSection[]>(['channel-sections'], (old) =>
          old?.map(s => s.id === fromSection.id ? { ...s, channel_ids: oldIds } : s)
        );

        try {
          await apiClient.reorderSectionItems(fromSection.id, {
            order: oldIds.map((id, i) => ({ channel_id: id, position: i })),
          });
        } catch {
          queryClient.invalidateQueries({ queryKey: ['channel-sections'] });
        }
      } else {
        // Cross-section move
        const fromIds = fromSection.channel_ids.filter(id => id !== activeChannelId);
        const toIds = [...toSection.channel_ids];
        const overIndex = toIds.indexOf(overChannelId);
        toIds.splice(overIndex, 0, activeChannelId);

        // Optimistic update
        queryClient.setQueryData<ChannelSection[]>(['channel-sections'], (old) =>
          old?.map(s => {
            if (s.id === fromSection.id) return { ...s, channel_ids: fromIds };
            if (s.id === toSection.id) return { ...s, channel_ids: toIds };
            return s;
          })
        );

        try {
          await apiClient.removeChannelFromSection(fromSection.id, activeChannelId);
          await apiClient.addChannelToSection(toSection.id, activeChannelId, overIndex);
        } catch {
          queryClient.invalidateQueries({ queryKey: ['channel-sections'] });
        }
      }
    }
  };

  // Build a set of all channel IDs assigned to any section
  const assignedChannelIds = new Set(sections.flatMap(s => s.channel_ids));
  const unsectionedChannels = channels.filter(ch => !assignedChannelIds.has(ch.id));

  if (isLoading) {
    return <div className="px-2 py-2 text-xs text-gray-500">Loading sections...</div>;
  }

  return (
    <DndContext
      sensors={sensors}
      collisionDetection={closestCenter}
      onDragStart={handleDragStart}
      onDragEnd={handleDragEnd}
    >
      <div className="space-y-1">
        <SortableContext items={sectionIds} strategy={verticalListSortingStrategy}>
          {sections.map((section) => {
            const sectionChannels = section.channel_ids
              .map(id => channels.find(ch => ch.id === id))
              .filter((ch): ch is Channel => ch !== undefined);

            const channelDndIds = sectionChannels.map(ch => `channel-${ch.id}`);

            return (
              <div key={section.id}>
                <SortableSectionHeader
                  section={section}
                  channelCount={sectionChannels.length}
                  isRenaming={renamingId === section.id}
                  renameValue={renameValue}
                  onRenameChange={setRenameValue}
                  onRenameSubmit={() => handleRename(section.id)}
                  onRenameCancel={() => setRenamingId(null)}
                  onToggleCollapse={() => handleToggleCollapse(section)}
                  onContextMenu={(e) => handleContextMenu(e, section.id, section.name)}
                />

                {!section.collapsed && (
                  <div className="ml-2">
                    <SortableContext items={channelDndIds} strategy={verticalListSortingStrategy}>
                      {sectionChannels.map((channel) => (
                        <SortableChannelItem
                          key={channel.id}
                          channel={channel}
                          activeChannel={activeChannel}
                          onSelectChannel={onSelectChannel}
                          onLeaveChannel={onLeaveChannel}
                        />
                      ))}
                    </SortableContext>
                    {sectionChannels.length === 0 && (
                      <p className="px-2 py-1 text-xs text-gray-600 italic">Drop channels here</p>
                    )}
                  </div>
                )}
              </div>
            );
          })}
        </SortableContext>

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

      <DragOverlay>
        {activeDragId && activeDragId.startsWith('section-') && (
          <div className="rounded bg-gray-700 px-3 py-1.5 text-xs font-semibold text-gray-300 uppercase shadow-lg">
            {sections.find(s => `section-${s.id}` === activeDragId)?.name || 'Section'}
          </div>
        )}
        {activeDragId && activeDragId.startsWith('channel-') && (
          <div className="rounded bg-gray-700 px-3 py-1.5 text-sm text-gray-300 shadow-lg">
            # {channels.find(ch => `channel-${ch.id}` === activeDragId)?.name || 'Channel'}
          </div>
        )}
      </DragOverlay>
    </DndContext>
  );
}
