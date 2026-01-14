'use client';

import { useState, useEffect, useMemo } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { apiClient } from '@/lib/api';
import type { Channel, User, ChannelMember } from '@/lib/types';

interface AddMembersModalProps {
  channel: Channel;
  isOpen: boolean;
  onClose: () => void;
  onSuccess?: () => void;
}

export default function AddMembersModal({
  channel,
  isOpen,
  onClose,
  onSuccess,
}: AddMembersModalProps) {
  const [searchQuery, setSearchQuery] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [addingUserId, setAddingUserId] = useState<string | null>(null);
  const queryClient = useQueryClient();

  // Fetch all org users
  const { data: allUsers = [], isLoading: loadingUsers } = useQuery({
    queryKey: ['users'],
    queryFn: () => apiClient.listUsers(),
    enabled: isOpen,
  });

  // Fetch current channel members
  const { data: members = [], isLoading: loadingMembers } = useQuery({
    queryKey: ['channel-members', channel.id],
    queryFn: () => apiClient.listChannelMembers(channel.id),
    enabled: isOpen,
  });

  // Reset state when modal opens/closes
  useEffect(() => {
    if (isOpen) {
      setSearchQuery('');
      setError(null);
      setAddingUserId(null);
    }
  }, [isOpen]);

  // Get set of existing member user IDs for fast lookup
  const memberUserIds = useMemo(() => {
    return new Set(members.map((m: ChannelMember) => m.user_id));
  }, [members]);

  // Filter users who are not already members and match search query
  const availableUsers = useMemo(() => {
    return allUsers.filter((user: User) => {
      if (memberUserIds.has(user.id)) return false;
      if (!searchQuery.trim()) return true;
      const query = searchQuery.toLowerCase();
      return (
        user.display_name.toLowerCase().includes(query) ||
        user.email.toLowerCase().includes(query)
      );
    });
  }, [allUsers, memberUserIds, searchQuery]);

  // Get current members with user info for display
  const currentMembers = useMemo(() => {
    return members.map((member: ChannelMember) => {
      const user = allUsers.find((u: User) => u.id === member.user_id);
      return { ...member, user };
    });
  }, [members, allUsers]);

  const addMemberMutation = useMutation({
    mutationFn: (userId: string) =>
      apiClient.addChannelMember(channel.id, { user_id: userId, role: 'member' }),
    onSuccess: (_data, userId) => {
      // Optimistically add the new member to the cache immediately
      const user = allUsers.find((u: User) => u.id === userId);
      if (user) {
        queryClient.setQueryData<ChannelMember[]>(
          ['channel-members', channel.id],
          (oldMembers = []) => [
            ...oldMembers,
            {
              id: crypto.randomUUID(),
              channel_id: channel.id,
              user_id: userId,
              role: 'member',
              joined_at: new Date().toISOString(),
            },
          ]
        );
      }
      // Also invalidate to ensure eventual consistency with server
      queryClient.invalidateQueries({ queryKey: ['channel-members', channel.id] });
      setAddingUserId(null);
      onSuccess?.();
    },
    onError: (err: Error) => {
      setError(err.message || 'Failed to add member');
      setAddingUserId(null);
    },
  });

  const handleAddMember = (userId: string) => {
    setError(null);
    setAddingUserId(userId);
    addMemberMutation.mutate(userId);
  };

  if (!isOpen) return null;

  const isLoading = loadingUsers || loadingMembers;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-50">
      <div className="w-full max-w-lg rounded-lg bg-gray-900 p-6 shadow-xl">
        <div className="mb-4 flex items-center justify-between">
          <h2 className="text-xl font-semibold text-white">
            Add Members to #{channel.name}
          </h2>
          <button
            onClick={onClose}
            className="rounded p-1 text-gray-400 hover:bg-gray-800 hover:text-white"
          >
            <svg className="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        {/* Search input */}
        <div className="mb-4">
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full rounded-lg border border-gray-700 bg-gray-800 px-3 py-2 text-white placeholder-gray-500 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
            placeholder="Search users by name or email..."
            autoFocus
          />
        </div>

        {error && (
          <div className="mb-4 rounded-lg bg-red-900 bg-opacity-50 px-3 py-2 text-sm text-red-300">
            {error}
          </div>
        )}

        {isLoading ? (
          <div className="flex items-center justify-center py-8">
            <div className="h-6 w-6 animate-spin rounded-full border-2 border-blue-500 border-t-transparent"></div>
          </div>
        ) : (
          <>
            {/* Available users to add */}
            <div className="mb-4">
              <h3 className="mb-2 text-sm font-medium text-gray-400">
                Add Members ({availableUsers.length} available)
              </h3>
              <div className="max-h-48 overflow-y-auto rounded-lg border border-gray-700 bg-gray-800">
                {availableUsers.length === 0 ? (
                  <div className="px-4 py-3 text-center text-sm text-gray-500">
                    {searchQuery ? 'No users match your search' : 'All users are already members'}
                  </div>
                ) : (
                  availableUsers.map((user: User) => (
                    <div
                      key={user.id}
                      className="flex items-center justify-between border-b border-gray-700 px-4 py-2 last:border-b-0 hover:bg-gray-750"
                    >
                      <div className="flex items-center gap-3">
                        <div className="flex h-8 w-8 items-center justify-center rounded-full bg-gray-600 text-sm font-medium text-white">
                          {user.display_name.charAt(0).toUpperCase()}
                        </div>
                        <div>
                          <div className="text-sm font-medium text-white">{user.display_name}</div>
                          <div className="text-xs text-gray-500">{user.email}</div>
                        </div>
                      </div>
                      <button
                        onClick={() => handleAddMember(user.id)}
                        disabled={addingUserId === user.id}
                        className="rounded-lg bg-blue-600 px-3 py-1 text-sm text-white hover:bg-blue-700 disabled:cursor-not-allowed disabled:opacity-50"
                      >
                        {addingUserId === user.id ? 'Adding...' : 'Add'}
                      </button>
                    </div>
                  ))
                )}
              </div>
            </div>

            {/* Current members */}
            <div>
              <h3 className="mb-2 text-sm font-medium text-gray-400">
                Current Members ({currentMembers.length})
              </h3>
              <div className="max-h-32 overflow-y-auto rounded-lg border border-gray-700 bg-gray-800">
                {currentMembers.length === 0 ? (
                  <div className="px-4 py-3 text-center text-sm text-gray-500">
                    No members yet
                  </div>
                ) : (
                  currentMembers.map((member: ChannelMember & { user?: User }) => (
                    <div
                      key={member.id}
                      className="flex items-center gap-3 border-b border-gray-700 px-4 py-2 last:border-b-0"
                    >
                      <div className="flex h-8 w-8 items-center justify-center rounded-full bg-gray-600 text-sm font-medium text-white">
                        {member.user?.display_name?.charAt(0).toUpperCase() || '?'}
                      </div>
                      <div className="flex-1">
                        <div className="text-sm font-medium text-white">
                          {member.user?.display_name || 'Unknown User'}
                        </div>
                        <div className="text-xs text-gray-500">{member.user?.email}</div>
                      </div>
                      {member.role === 'admin' && (
                        <span className="rounded bg-blue-900 px-2 py-0.5 text-xs text-blue-300">
                          Admin
                        </span>
                      )}
                    </div>
                  ))
                )}
              </div>
            </div>
          </>
        )}

        <div className="mt-4 flex justify-end">
          <button
            type="button"
            onClick={onClose}
            className="rounded-lg px-4 py-2 text-gray-300 hover:bg-gray-800"
          >
            Done
          </button>
        </div>
      </div>
    </div>
  );
}
