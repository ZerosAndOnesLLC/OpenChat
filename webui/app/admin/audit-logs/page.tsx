'use client';

import { useState, useEffect } from 'react';
import { apiClient } from '@/lib/api';

interface AuditLog {
  id: string;
  user_id: string | null;
  action: string;
  resource_type: string;
  resource_id: string | null;
  metadata: unknown;
  ip_address: string | null;
  user_agent: string | null;
  timestamp: string;
}

interface AuditLogsResponse {
  logs: AuditLog[];
  total: number;
  limit: number;
  offset: number;
}

interface AuditLogFilters {
  userId: string;
  action: string;
  resourceType: string;
  resourceId: string;
  startDate: string;
  endDate: string;
}

const EMPTY_FILTERS: AuditLogFilters = {
  userId: '',
  action: '',
  resourceType: '',
  resourceId: '',
  startDate: '',
  endDate: '',
};

const API_BASE = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';

function authHeaders() {
  return { 'Authorization': `Bearer ${apiClient.getToken()}` };
}

async function fetchStringList(path: string): Promise<string[]> {
  const response = await fetch(`${API_BASE}${path}`, { headers: authHeaders() });
  if (!response.ok) {
    throw new Error(`Request failed: ${response.status}`);
  }
  return response.json();
}

async function fetchAuditLogs(
  filters: AuditLogFilters,
  limit: number,
  offset: number,
): Promise<AuditLogsResponse> {
  const params = new URLSearchParams();
  if (filters.userId) params.append('user_id', filters.userId);
  if (filters.action) params.append('action', filters.action);
  if (filters.resourceType) params.append('resource_type', filters.resourceType);
  if (filters.resourceId) params.append('resource_id', filters.resourceId);
  if (filters.startDate) params.append('start_date', new Date(filters.startDate).toISOString());
  if (filters.endDate) params.append('end_date', new Date(filters.endDate).toISOString());
  params.append('limit', limit.toString());
  params.append('offset', offset.toString());

  const response = await fetch(`${API_BASE}/api/audit-logs?${params}`, { headers: authHeaders() });
  if (!response.ok) {
    throw new Error('Failed to fetch audit logs');
  }
  return response.json();
}

export default function AuditLogsPage() {
  const [logs, setLogs] = useState<AuditLog[]>([]);
  const [total, setTotal] = useState(0);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Filters (form inputs; applied to the query on Search / Reset)
  const [userId, setUserId] = useState('');
  const [action, setAction] = useState('');
  const [resourceType, setResourceType] = useState('');
  const [resourceId, setResourceId] = useState('');
  const [startDate, setStartDate] = useState('');
  const [endDate, setEndDate] = useState('');
  const [appliedFilters, setAppliedFilters] = useState<AuditLogFilters>(EMPTY_FILTERS);
  const [limit, setLimit] = useState(50);
  const [offset, setOffset] = useState(0);

  // Available filter options
  const [actions, setActions] = useState<string[]>([]);
  const [resourceTypes, setResourceTypes] = useState<string[]>([]);

  // Expanded row
  const [expandedRow, setExpandedRow] = useState<string | null>(null);

  useEffect(() => {
    fetchStringList('/api/audit-logs/actions')
      .then(setActions)
      .catch((err) => console.error('Failed to fetch actions:', err));
    fetchStringList('/api/audit-logs/resource-types')
      .then(setResourceTypes)
      .catch((err) => console.error('Failed to fetch resource types:', err));
  }, []);

  useEffect(() => {
    let cancelled = false;
    fetchAuditLogs(appliedFilters, limit, offset)
      .then((data) => {
        if (cancelled) return;
        setLogs(data.logs);
        setTotal(data.total);
      })
      .catch((err) => {
        if (cancelled) return;
        console.error('Failed to fetch logs:', err);
        setError('Failed to load audit logs');
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [appliedFilters, limit, offset]);

  /** Mark the list as reloading; the effect above performs the fetch. */
  const startReload = () => {
    setLoading(true);
    setError(null);
  };

  const handleSearch = () => {
    startReload();
    setOffset(0);
    setAppliedFilters({ userId, action, resourceType, resourceId, startDate, endDate });
  };

  const handleReset = () => {
    setUserId('');
    setAction('');
    setResourceType('');
    setResourceId('');
    setStartDate('');
    setEndDate('');
    startReload();
    setOffset(0);
    setAppliedFilters({ ...EMPTY_FILTERS });
  };

  const handleExport = async () => {
    try {
      const params = new URLSearchParams();
      if (userId) params.append('user_id', userId);
      if (action) params.append('action', action);
      if (resourceType) params.append('resource_type', resourceType);
      if (resourceId) params.append('resource_id', resourceId);
      if (startDate) params.append('start_date', new Date(startDate).toISOString());
      if (endDate) params.append('end_date', new Date(endDate).toISOString());

      const response = await fetch(`${process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080'}/api/audit-logs/export?${params}`, {
        headers: {
          'Authorization': `Bearer ${apiClient.getToken()}`,
        },
      });

      if (!response.ok) {
        throw new Error('Failed to export audit logs');
      }

      const blob = await response.blob();
      const url = window.URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `audit-logs-${new Date().toISOString()}.csv`;
      a.click();
      window.URL.revokeObjectURL(url);
    } catch (err) {
      console.error('Failed to export logs:', err);
      setError('Failed to export audit logs');
    }
  };

  const formatDate = (timestamp: string) => {
    return new Date(timestamp).toLocaleString();
  };

  const toggleExpandRow = (id: string) => {
    setExpandedRow(expandedRow === id ? null : id);
  };

  const nextPage = () => {
    startReload();
    setOffset(offset + limit);
  };

  const prevPage = () => {
    startReload();
    setOffset(Math.max(0, offset - limit));
  };

  return (
    <div className="min-h-screen bg-gray-50 p-6">
      <div className="max-w-7xl mx-auto">
        <div className="bg-white rounded-lg shadow p-6 mb-6">
          <h1 className="text-3xl font-bold mb-6">Audit Logs</h1>

          {/* Filters */}
          <div className="mb-6">
            <h2 className="text-lg font-semibold mb-4">Filters</h2>
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  User ID
                </label>
                <input
                  type="text"
                  value={userId}
                  onChange={(e) => setUserId(e.target.value)}
                  placeholder="Enter user ID"
                  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Action
                </label>
                <select
                  value={action}
                  onChange={(e) => setAction(e.target.value)}
                  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                >
                  <option value="">All Actions</option>
                  {actions.map((a) => (
                    <option key={a} value={a}>{a}</option>
                  ))}
                </select>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Resource Type
                </label>
                <select
                  value={resourceType}
                  onChange={(e) => setResourceType(e.target.value)}
                  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                >
                  <option value="">All Resource Types</option>
                  {resourceTypes.map((rt) => (
                    <option key={rt} value={rt}>{rt}</option>
                  ))}
                </select>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Resource ID
                </label>
                <input
                  type="text"
                  value={resourceId}
                  onChange={(e) => setResourceId(e.target.value)}
                  placeholder="Enter resource ID"
                  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Start Date
                </label>
                <input
                  type="datetime-local"
                  value={startDate}
                  onChange={(e) => setStartDate(e.target.value)}
                  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  End Date
                </label>
                <input
                  type="datetime-local"
                  value={endDate}
                  onChange={(e) => setEndDate(e.target.value)}
                  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
              </div>
            </div>

            <div className="mt-4 flex gap-2">
              <button
                onClick={handleSearch}
                className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500"
              >
                Search
              </button>
              <button
                onClick={handleReset}
                className="px-4 py-2 bg-gray-600 text-white rounded-md hover:bg-gray-700 focus:outline-none focus:ring-2 focus:ring-gray-500"
              >
                Reset
              </button>
              <button
                onClick={handleExport}
                className="px-4 py-2 bg-green-600 text-white rounded-md hover:bg-green-700 focus:outline-none focus:ring-2 focus:ring-green-500"
              >
                Export CSV
              </button>
            </div>
          </div>

          {error && (
            <div className="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded mb-4">
              {error}
            </div>
          )}

          {/* Results */}
          <div>
            <div className="flex justify-between items-center mb-4">
              <h2 className="text-lg font-semibold">
                Results ({total} total)
              </h2>
              <div className="flex items-center gap-2">
                <label className="text-sm">Per page:</label>
                <select
                  value={limit}
                  onChange={(e) => {
                    startReload();
                    setLimit(Number(e.target.value));
                    setOffset(0);
                  }}
                  className="px-2 py-1 border border-gray-300 rounded-md"
                >
                  <option value={25}>25</option>
                  <option value={50}>50</option>
                  <option value={100}>100</option>
                </select>
              </div>
            </div>

            {loading ? (
              <div className="text-center py-8">Loading...</div>
            ) : logs.length === 0 ? (
              <div className="text-center py-8 text-gray-500">No audit logs found</div>
            ) : (
              <div className="overflow-x-auto">
                <table className="min-w-full divide-y divide-gray-200">
                  <thead className="bg-gray-50">
                    <tr>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Timestamp
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Action
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Resource Type
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        User ID
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        IP Address
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Actions
                      </th>
                    </tr>
                  </thead>
                  <tbody className="bg-white divide-y divide-gray-200">
                    {logs.map((log) => (
                      <>
                        <tr key={log.id} className="hover:bg-gray-50">
                          <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                            {formatDate(log.timestamp)}
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                            {log.action}
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                            {log.resource_type}
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500 font-mono">
                            {log.user_id ? log.user_id.substring(0, 8) + '...' : '-'}
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                            {log.ip_address || '-'}
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap text-sm">
                            <button
                              onClick={() => toggleExpandRow(log.id)}
                              className="text-blue-600 hover:text-blue-900"
                            >
                              {expandedRow === log.id ? 'Hide Details' : 'Show Details'}
                            </button>
                          </td>
                        </tr>
                        {expandedRow === log.id && (
                          <tr>
                            <td colSpan={6} className="px-6 py-4 bg-gray-50">
                              <div className="space-y-2 text-sm">
                                <div>
                                  <span className="font-semibold">ID:</span> {log.id}
                                </div>
                                <div>
                                  <span className="font-semibold">User ID:</span> {log.user_id || 'N/A'}
                                </div>
                                <div>
                                  <span className="font-semibold">Resource ID:</span> {log.resource_id || 'N/A'}
                                </div>
                                <div>
                                  <span className="font-semibold">User Agent:</span>{' '}
                                  <span className="text-gray-600">{log.user_agent || 'N/A'}</span>
                                </div>
                                <div>
                                  <span className="font-semibold">Metadata:</span>
                                  <pre className="mt-1 p-2 bg-gray-100 rounded overflow-auto max-h-48">
                                    {JSON.stringify(log.metadata, null, 2)}
                                  </pre>
                                </div>
                              </div>
                            </td>
                          </tr>
                        )}
                      </>
                    ))}
                  </tbody>
                </table>
              </div>
            )}

            {/* Pagination */}
            {total > 0 && (
              <div className="mt-4 flex justify-between items-center">
                <div className="text-sm text-gray-700">
                  Showing {offset + 1} to {Math.min(offset + limit, total)} of {total} results
                </div>
                <div className="flex gap-2">
                  <button
                    onClick={prevPage}
                    disabled={offset === 0}
                    className="px-4 py-2 bg-gray-600 text-white rounded-md hover:bg-gray-700 disabled:bg-gray-300 disabled:cursor-not-allowed"
                  >
                    Previous
                  </button>
                  <button
                    onClick={nextPage}
                    disabled={offset + limit >= total}
                    className="px-4 py-2 bg-gray-600 text-white rounded-md hover:bg-gray-700 disabled:bg-gray-300 disabled:cursor-not-allowed"
                  >
                    Next
                  </button>
                </div>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
