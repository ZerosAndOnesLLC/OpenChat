'use client';

import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { apiClient } from '@/lib/api';
import type { DirectMessage, User } from '@/lib/types';
import { useAuth } from '@/lib/auth';

interface NewDmModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSelectDm: (dm: DirectMessage) => void;
  currentDms: DirectMessage[];
}

export default function NewDmModal({
  isOpen,
  onClose,
  onSelectDm,
  currentDms,
}: NewDmModalProps) {
  const { user: currentUser } = useAuth();
  const [searchQuery, setSearchQuery] = useState('');
  const queryClient = useQueryClient();

  const { data: users, isLoading } = useQuery({
    queryKey: ['users'],
    queryFn: () => apiClient.listUsers(),
    enabled: isOpen,
  });

  const createDmMutation = useMutation({
    mutationFn: async (userId: string) => {
      return apiClient.createDm({ participant_ids: [userId] });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['dms'] });
    },
  });

  const handleStartDm = async (user: User) => {
    // Check if DM already exists with this user
    const existingDm = currentDms.find((dm) =>
      dm.participants?.some((p) => p.id === user.id)
    );

    if (existingDm) {
      // Just navigate to existing DM
      onSelectDm(existingDm);
      onClose();
      return;
    }

    try {
      const newDm = await createDmMutation.mutateAsync(user.id);
      onSelectDm(newDm);
      onClose();
    } catch (error) {
      console.error('Failed to create DM:', error);
    }
  };

  if (!isOpen) return null;

  // Filter out current user and apply search
  const filteredUsers = (users || []).filter((user) => {
    if (user.id === currentUser?.id) return false;
    if (!searchQuery) return true;
    return (
      user.display_name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      user.email.toLowerCase().includes(searchQuery.toLowerCase())
    );
  });

  // Separate users into those with existing DMs and those without
  const usersWithExistingDm = filteredUsers.filter((user) =>
    currentDms.some((dm) => dm.participants?.some((p) => p.id === user.id))
  );
  const usersWithoutDm = filteredUsers.filter(
    (user) => !currentDms.some((dm) => dm.participants?.some((p) => p.id === user.id))
  );

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'online':
        return 'bg-green-500';
      case 'away':
        return 'bg-yellow-500';
      case 'dnd':
        return 'bg-red-500';
      default:
        return 'bg-gray-500';
    }
  };

  return (
    <>
      {/* Backdrop */}
      <div
        className="fixed inset-0 z-40 bg-black bg-opacity-50"
        onClick={onClose}
      />

      {/* Modal */}
      <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
        <div className="w-full max-w-2xl rounded-lg bg-gray-900 shadow-xl">
          {/* Header */}
          <div className="flex items-center justify-between border-b border-gray-700 px-6 py-4">
            <h2 className="text-xl font-semibold text-white">New Message</h2>
            <button
              onClick={onClose}
              className="text-gray-400 hover:text-white"
            >
              <svg className="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>

          {/* Search */}
          <div className="border-b border-gray-700 px-6 py-4">
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="Search users by name or email..."
              className="w-full rounded-md border border-gray-600 bg-gray-800 px-4 py-2 text-sm text-white placeholder-gray-400 focus:border-blue-500 focus:outline-none"
              autoFocus
            />
          </div>

          {/* Content */}
          <div className="max-h-[60vh] overflow-y-auto px-6 py-4">
            {isLoading ? (
              <div className="py-8 text-center text-gray-400">Loading users...</div>
            ) : (
              <>
                {/* New conversations */}
                {usersWithoutDm.length > 0 && (
                  <div className="mb-6">
                    <h3 className="mb-3 text-sm font-semibold text-gray-400">
                      Start a New Conversation ({usersWithoutDm.length})
                    </h3>
                    <div className="space-y-2">
                      {usersWithoutDm.map((user) => (
                        <button
                          key={user.id}
                          onClick={() => handleStartDm(user)}
                          disabled={createDmMutation.isPending}
                          className="flex w-full items-center gap-3 rounded-lg border border-gray-700 bg-gray-800 p-3 text-left transition-colors hover:border-gray-600 hover:bg-gray-750 disabled:cursor-not-allowed disabled:opacity-50"
                        >
                          <div className="relative">
                            {user.avatar_url ? (
                              <img
                                src={user.avatar_url}
                                alt={user.display_name}
                                className="h-10 w-10 rounded-full"
                              />
                            ) : (
                              <div className="flex h-10 w-10 items-center justify-center rounded-full bg-blue-600 text-white">
                                {user.display_name.charAt(0).toUpperCase()}
                              </div>
                            )}
                            <span
                              className={`absolute bottom-0 right-0 h-3 w-3 rounded-full border-2 border-gray-800 ${getStatusColor(user.status)}`}
                            />
                          </div>
                          <div className="flex-1 min-w-0">
                            <div className="font-medium text-white truncate">{user.display_name}</div>
                            <div className="text-sm text-gray-400 truncate">{user.email}</div>
                          </div>
                          <svg className="h-5 w-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z" />
                          </svg>
                        </button>
                      ))}
                    </div>
                  </div>
                )}

                {/* Existing conversations */}
                {usersWithExistingDm.length > 0 && (
                  <div>
                    <h3 className="mb-3 text-sm font-semibold text-gray-400">
                      Existing Conversations ({usersWithExistingDm.length})
                    </h3>
                    <div className="space-y-2">
                      {usersWithExistingDm.map((user) => {
                        const existingDm = currentDms.find((dm) =>
                          dm.participants?.some((p) => p.id === user.id)
                        );
                        return (
                          <button
                            key={user.id}
                            onClick={() => existingDm && handleStartDm(user)}
                            className="flex w-full items-center gap-3 rounded-lg border border-gray-700 bg-gray-800 p-3 text-left transition-colors hover:border-gray-600 hover:bg-gray-750"
                          >
                            <div className="relative">
                              {user.avatar_url ? (
                                <img
                                  src={user.avatar_url}
                                  alt={user.display_name}
                                  className="h-10 w-10 rounded-full"
                                />
                              ) : (
                                <div className="flex h-10 w-10 items-center justify-center rounded-full bg-blue-600 text-white">
                                  {user.display_name.charAt(0).toUpperCase()}
                                </div>
                              )}
                              <span
                                className={`absolute bottom-0 right-0 h-3 w-3 rounded-full border-2 border-gray-800 ${getStatusColor(user.status)}`}
                              />
                            </div>
                            <div className="flex-1 min-w-0">
                              <div className="font-medium text-white truncate">{user.display_name}</div>
                              <div className="text-sm text-gray-400 truncate">{user.email}</div>
                            </div>
                            <div className="flex items-center gap-2 text-sm text-green-500">
                              <svg className="h-4 w-4" fill="currentColor" viewBox="0 0 20 20">
                                <path
                                  fillRule="evenodd"
                                  d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
                                  clipRule="evenodd"
                                />
                              </svg>
                              <span>Chat</span>
                            </div>
                          </button>
                        );
                      })}
                    </div>
                  </div>
                )}

                {/* No results */}
                {filteredUsers.length === 0 && (
                  <div className="py-8 text-center text-gray-400">
                    {searchQuery
                      ? 'No users found matching your search'
                      : 'No users available'}
                  </div>
                )}
              </>
            )}
          </div>

          {/* Footer */}
          <div className="border-t border-gray-700 px-6 py-4">
            <button
              onClick={onClose}
              className="w-full rounded-md border border-gray-600 px-4 py-2 text-sm font-medium text-gray-300 transition-colors hover:bg-gray-800"
            >
              Close
            </button>
          </div>
        </div>
      </div>
    </>
  );
}
