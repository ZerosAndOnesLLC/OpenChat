'use client';

import { useState, useEffect } from 'react';
import { apiClient } from '@/lib/api';
import type { UserGroup, UserGroupMember, User } from '@/lib/types';

export default function UserGroupsPage() {
  const [groups, setGroups] = useState<UserGroup[]>([]);
  const [users, setUsers] = useState<User[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Create form
  const [showCreate, setShowCreate] = useState(false);
  const [newName, setNewName] = useState('');
  const [newHandle, setNewHandle] = useState('');
  const [newDescription, setNewDescription] = useState('');
  const [creating, setCreating] = useState(false);

  // Expanded group for member management
  const [expandedGroupId, setExpandedGroupId] = useState<string | null>(null);
  const [groupMembers, setGroupMembers] = useState<UserGroupMember[]>([]);
  const [memberSearch, setMemberSearch] = useState('');
  const [loadingMembers, setLoadingMembers] = useState(false);

  useEffect(() => {
    fetchGroups();
    fetchUsers();
  }, []);

  const fetchGroups = async () => {
    try {
      setLoading(true);
      setError(null);
      const data = await apiClient.listUserGroups();
      setGroups(data);
    } catch (err) {
      setError((err as Error).message);
    } finally {
      setLoading(false);
    }
  };

  const fetchUsers = async () => {
    try {
      const data = await apiClient.listUsers();
      setUsers(data);
    } catch (err) {
      console.error('Failed to load users:', err);
    }
  };

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newName.trim() || !newHandle.trim()) return;

    try {
      setCreating(true);
      setError(null);
      await apiClient.createUserGroup({
        name: newName,
        handle: newHandle,
        description: newDescription || undefined,
      });
      setNewName('');
      setNewHandle('');
      setNewDescription('');
      setShowCreate(false);
      fetchGroups();
    } catch (err) {
      setError((err as Error).message);
    } finally {
      setCreating(false);
    }
  };

  const handleDelete = async (groupId: string) => {
    if (!confirm('Are you sure you want to delete this group?')) return;
    try {
      await apiClient.deleteUserGroup(groupId);
      setGroups(groups.filter(g => g.id !== groupId));
      if (expandedGroupId === groupId) setExpandedGroupId(null);
    } catch (err) {
      setError((err as Error).message);
    }
  };

  const toggleExpand = async (groupId: string) => {
    if (expandedGroupId === groupId) {
      setExpandedGroupId(null);
      return;
    }
    setExpandedGroupId(groupId);
    setLoadingMembers(true);
    try {
      const members = await apiClient.getGroupMembers(groupId);
      setGroupMembers(members);
    } catch (err) {
      console.error('Failed to load members:', err);
    } finally {
      setLoadingMembers(false);
    }
  };

  const handleAddMember = async (userId: string) => {
    if (!expandedGroupId) return;
    try {
      const member = await apiClient.addGroupMember(expandedGroupId, userId);
      setGroupMembers([...groupMembers, member]);
      setMemberSearch('');
    } catch (err) {
      setError((err as Error).message);
    }
  };

  const handleRemoveMember = async (userId: string) => {
    if (!expandedGroupId) return;
    try {
      await apiClient.removeGroupMember(expandedGroupId, userId);
      setGroupMembers(groupMembers.filter(m => m.user_id !== userId));
    } catch (err) {
      setError((err as Error).message);
    }
  };

  const autoSlugHandle = (name: string) => {
    return name.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '');
  };

  const memberUserIds = new Set(groupMembers.map(m => m.user_id));
  const filteredSearchUsers = users.filter(u =>
    !memberUserIds.has(u.id) &&
    (u.display_name.toLowerCase().includes(memberSearch.toLowerCase()) ||
     u.email.toLowerCase().includes(memberSearch.toLowerCase()))
  ).slice(0, 8);

  return (
    <div className="min-h-screen bg-black p-8">
      <div className="mx-auto max-w-4xl">
        <div className="mb-8 flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-bold text-white">User Groups</h1>
            <p className="mt-1 text-sm text-gray-400">
              Manage user groups for @mention teams
            </p>
          </div>
          <button
            onClick={() => setShowCreate(!showCreate)}
            className="rounded-lg bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700"
          >
            Create Group
          </button>
        </div>

        {error && (
          <div className="mb-4 rounded-lg bg-red-900/50 border border-red-700 p-3 text-sm text-red-300">
            {error}
          </div>
        )}

        {showCreate && (
          <form onSubmit={handleCreate} className="mb-6 rounded-lg bg-gray-900 border border-gray-800 p-4">
            <h3 className="mb-3 text-sm font-semibold text-white">New User Group</h3>
            <div className="mb-3">
              <label className="mb-1 block text-xs text-gray-400">Name</label>
              <input
                type="text"
                value={newName}
                onChange={(e) => {
                  setNewName(e.target.value);
                  if (!newHandle || newHandle === autoSlugHandle(newName)) {
                    setNewHandle(autoSlugHandle(e.target.value));
                  }
                }}
                className="w-full rounded bg-gray-800 px-3 py-2 text-sm text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-blue-500"
                placeholder="Engineering Team"
                autoFocus
              />
            </div>
            <div className="mb-3">
              <label className="mb-1 block text-xs text-gray-400">Handle (used for @mentions)</label>
              <div className="flex items-center">
                <span className="mr-1 text-sm text-gray-500">@</span>
                <input
                  type="text"
                  value={newHandle}
                  onChange={(e) => setNewHandle(e.target.value.toLowerCase().replace(/[^a-z0-9_-]/g, ''))}
                  className="w-full rounded bg-gray-800 px-3 py-2 text-sm text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-blue-500"
                  placeholder="engineering-team"
                />
              </div>
            </div>
            <div className="mb-3">
              <label className="mb-1 block text-xs text-gray-400">Description (optional)</label>
              <input
                type="text"
                value={newDescription}
                onChange={(e) => setNewDescription(e.target.value)}
                className="w-full rounded bg-gray-800 px-3 py-2 text-sm text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-blue-500"
                placeholder="The engineering team"
              />
            </div>
            <div className="flex gap-2">
              <button
                type="submit"
                disabled={creating || !newName.trim() || !newHandle.trim()}
                className="rounded bg-blue-600 px-4 py-2 text-sm text-white hover:bg-blue-700 disabled:opacity-50"
              >
                {creating ? 'Creating...' : 'Create'}
              </button>
              <button
                type="button"
                onClick={() => setShowCreate(false)}
                className="rounded bg-gray-700 px-4 py-2 text-sm text-gray-300 hover:bg-gray-600"
              >
                Cancel
              </button>
            </div>
          </form>
        )}

        {loading ? (
          <div className="text-center text-gray-400">Loading...</div>
        ) : groups.length === 0 ? (
          <div className="rounded-lg bg-gray-900 border border-gray-800 p-8 text-center text-gray-400">
            No user groups yet. Create one to get started.
          </div>
        ) : (
          <div className="space-y-2">
            {groups.map((group) => (
              <div key={group.id} className="rounded-lg bg-gray-900 border border-gray-800">
                <div
                  className="flex cursor-pointer items-center justify-between p-4 hover:bg-gray-800/50"
                  onClick={() => toggleExpand(group.id)}
                >
                  <div className="flex items-center gap-3">
                    <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-purple-600/20 text-purple-400">
                      <svg className="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
                      </svg>
                    </div>
                    <div>
                      <div className="font-medium text-white">{group.name}</div>
                      <div className="text-sm text-gray-400">@{group.handle}{group.description ? ` - ${group.description}` : ''}</div>
                    </div>
                  </div>
                  <div className="flex items-center gap-2">
                    <button
                      onClick={(e) => { e.stopPropagation(); handleDelete(group.id); }}
                      className="rounded p-1.5 text-gray-500 hover:bg-gray-700 hover:text-red-400"
                      title="Delete group"
                    >
                      <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                      </svg>
                    </button>
                    <svg className={`h-4 w-4 text-gray-400 transition-transform ${expandedGroupId === group.id ? 'rotate-180' : ''}`} fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
                    </svg>
                  </div>
                </div>

                {expandedGroupId === group.id && (
                  <div className="border-t border-gray-800 p-4">
                    <h4 className="mb-3 text-sm font-semibold text-gray-300">Members</h4>

                    {/* Search to add members */}
                    <div className="relative mb-3">
                      <input
                        type="text"
                        value={memberSearch}
                        onChange={(e) => setMemberSearch(e.target.value)}
                        className="w-full rounded bg-gray-800 px-3 py-2 text-sm text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-blue-500"
                        placeholder="Search users to add..."
                      />
                      {memberSearch && filteredSearchUsers.length > 0 && (
                        <div className="absolute top-full left-0 right-0 z-10 mt-1 rounded-lg bg-gray-800 border border-gray-700 shadow-lg max-h-48 overflow-y-auto">
                          {filteredSearchUsers.map(u => (
                            <button
                              key={u.id}
                              onClick={() => handleAddMember(u.id)}
                              className="flex w-full items-center gap-2 px-3 py-2 text-left text-sm text-gray-300 hover:bg-gray-700"
                            >
                              <div className="flex h-6 w-6 items-center justify-center rounded-full bg-gray-600 text-xs font-semibold">
                                {u.display_name.charAt(0).toUpperCase()}
                              </div>
                              <span>{u.display_name}</span>
                              <span className="text-xs text-gray-500">{u.email}</span>
                            </button>
                          ))}
                        </div>
                      )}
                    </div>

                    {loadingMembers ? (
                      <div className="text-sm text-gray-400">Loading members...</div>
                    ) : groupMembers.length === 0 ? (
                      <div className="text-sm text-gray-500">No members yet</div>
                    ) : (
                      <div className="space-y-1">
                        {groupMembers.map(member => {
                          const memberUser = users.find(u => u.id === member.user_id);
                          return (
                            <div key={member.id} className="flex items-center justify-between rounded bg-gray-800/50 px-3 py-2">
                              <div className="flex items-center gap-2">
                                <div className="flex h-7 w-7 items-center justify-center rounded-full bg-gray-600 text-xs font-semibold text-white">
                                  {memberUser?.display_name?.charAt(0).toUpperCase() || '?'}
                                </div>
                                <div>
                                  <div className="text-sm text-white">{memberUser?.display_name || member.user_id}</div>
                                  <div className="text-xs text-gray-500">{memberUser?.email}</div>
                                </div>
                              </div>
                              <button
                                onClick={() => handleRemoveMember(member.user_id)}
                                className="rounded p-1 text-gray-500 hover:bg-gray-700 hover:text-red-400"
                                title="Remove member"
                              >
                                <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                                </svg>
                              </button>
                            </div>
                          );
                        })}
                      </div>
                    )}
                  </div>
                )}
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
