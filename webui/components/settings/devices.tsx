'use client';

import { useState, useEffect } from 'react';
import { apiClient } from '@/lib/api';
import { DeviceSession } from '@/lib/types';
import { Monitor, Smartphone, Globe, Trash2, AlertCircle, CheckCircle, RefreshCw } from 'lucide-react';
import { formatDistanceToNow } from 'date-fns';

export default function DeviceManagement() {
  const [devices, setDevices] = useState<DeviceSession[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [revokingId, setRevokingId] = useState<string | null>(null);
  const [showConfirmDialog, setShowConfirmDialog] = useState<string | null>(null);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);

  const loadDevices = async () => {
    setIsLoading(true);
    setError(null);
    try {
      const sessions = await apiClient.getDeviceSessions();
      setDevices(sessions);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load devices');
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    loadDevices();
  }, []);

  const handleRevoke = async (deviceId: string) => {
    setRevokingId(deviceId);
    setError(null);
    setSuccessMessage(null);
    try {
      await apiClient.revokeDeviceSession(deviceId);
      setDevices((prev) => prev.filter((d) => d.id !== deviceId));
      setSuccessMessage('Device session revoked successfully');
      setTimeout(() => setSuccessMessage(null), 3000);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to revoke device session');
    } finally {
      setRevokingId(null);
      setShowConfirmDialog(null);
    }
  };

  const getDeviceIcon = (type: string) => {
    switch (type) {
      case 'desktop':
        return <Monitor className="w-5 h-5" />;
      case 'mobile':
        return <Smartphone className="w-5 h-5" />;
      case 'web':
        return <Globe className="w-5 h-5" />;
      default:
        return <Monitor className="w-5 h-5" />;
    }
  };

  const getDeviceColor = (type: string) => {
    switch (type) {
      case 'desktop':
        return 'bg-blue-500/10 text-blue-400';
      case 'mobile':
        return 'bg-purple-500/10 text-purple-400';
      case 'web':
        return 'bg-green-500/10 text-green-400';
      default:
        return 'bg-gray-700 text-gray-400';
    }
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-12">
        <RefreshCw className="w-6 h-6 text-blue-400 animate-spin" />
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Description and Refresh */}
      <div className="flex items-start justify-between gap-4">
        <p className="text-sm text-gray-400">
          Manage devices that have access to your account. Revoke access for any device you don&apos;t recognize.
        </p>
        <button
          onClick={loadDevices}
          className="flex-shrink-0 inline-flex items-center gap-2 px-3 py-1.5 text-sm font-medium text-gray-300 bg-gray-800 border border-gray-700 rounded-lg hover:bg-gray-700 transition-colors"
        >
          <RefreshCw className="w-4 h-4" />
          Refresh
        </button>
      </div>

      {/* Success Message */}
      {successMessage && (
        <div className="p-4 bg-green-500/10 border border-green-500/20 rounded-lg flex items-start gap-3">
          <CheckCircle className="w-5 h-5 text-green-400 flex-shrink-0 mt-0.5" />
          <p className="text-sm text-green-400">{successMessage}</p>
        </div>
      )}

      {/* Error Message */}
      {error && (
        <div className="p-4 bg-red-500/10 border border-red-500/20 rounded-lg flex items-start gap-3">
          <AlertCircle className="w-5 h-5 text-red-400 flex-shrink-0 mt-0.5" />
          <div>
            <p className="text-sm font-medium text-red-400">Error</p>
            <p className="text-sm text-red-400/80 mt-1">{error}</p>
          </div>
        </div>
      )}

      {/* Device List */}
      {devices.length === 0 ? (
        <div className="text-center py-12 bg-gray-900 rounded-xl border border-gray-800">
          <Monitor className="w-12 h-12 text-gray-600 mx-auto mb-4" />
          <p className="text-gray-400 font-medium">No active devices</p>
          <p className="text-sm text-gray-500 mt-1">
            Your device sessions will appear here
          </p>
        </div>
      ) : (
        <div className="space-y-3">
          {devices.map((device) => (
            <div
              key={device.id}
              className="bg-gray-900 border border-gray-800 rounded-xl p-4 hover:border-gray-700 transition-colors"
            >
              <div className="flex items-start justify-between gap-4">
                <div className="flex items-start gap-4 flex-1 min-w-0">
                  {/* Device Icon */}
                  <div className={`p-2.5 rounded-lg flex-shrink-0 ${getDeviceColor(device.device_type)}`}>
                    {getDeviceIcon(device.device_type)}
                  </div>

                  {/* Device Info */}
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2 flex-wrap mb-1">
                      <h4 className="font-medium text-white">
                        {device.device_name || `${device.device_type.charAt(0).toUpperCase() + device.device_type.slice(1)} Device`}
                      </h4>
                      <span className="px-2 py-0.5 text-xs font-medium bg-gray-800 text-gray-400 rounded-full capitalize">
                        {device.device_type}
                      </span>
                    </div>

                    <div className="space-y-0.5">
                      <p className="text-sm text-gray-400">
                        Last active:{' '}
                        <span className="text-gray-300">
                          {formatDistanceToNow(new Date(device.last_active_at), { addSuffix: true })}
                        </span>
                      </p>
                      <p className="text-xs text-gray-500">
                        Connected {formatDistanceToNow(new Date(device.created_at), { addSuffix: true })}
                      </p>
                      {device.device_fingerprint && (
                        <p className="text-xs font-mono text-gray-600 truncate">
                          ID: {device.device_fingerprint.slice(0, 16)}...
                        </p>
                      )}
                    </div>
                  </div>
                </div>

                {/* Action Button */}
                <button
                  onClick={() => setShowConfirmDialog(device.id)}
                  disabled={revokingId === device.id}
                  className="flex-shrink-0 inline-flex items-center gap-2 px-3 py-1.5 text-sm font-medium text-red-400 bg-red-500/10 border border-red-500/20 rounded-lg hover:bg-red-500/20 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  {revokingId === device.id ? (
                    <>
                      <RefreshCw className="w-4 h-4 animate-spin" />
                      <span className="hidden sm:inline">Revoking...</span>
                    </>
                  ) : (
                    <>
                      <Trash2 className="w-4 h-4" />
                      <span className="hidden sm:inline">Revoke</span>
                    </>
                  )}
                </button>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Confirmation Dialog */}
      {showConfirmDialog && (
        <div className="fixed inset-0 bg-black/70 flex items-center justify-center z-50 p-4">
          <div className="bg-gray-900 border border-gray-800 rounded-2xl shadow-2xl max-w-md w-full p-6">
            <div className="flex items-start gap-4 mb-6">
              <div className="p-3 bg-red-500/10 rounded-full">
                <AlertCircle className="w-6 h-6 text-red-400" />
              </div>
              <div>
                <h3 className="text-lg font-semibold text-white mb-2">
                  Revoke Device Access?
                </h3>
                <p className="text-sm text-gray-400">
                  This device will be immediately logged out and will need to authenticate again to regain access.
                </p>
              </div>
            </div>

            <div className="flex gap-3">
              <button
                onClick={() => setShowConfirmDialog(null)}
                className="flex-1 px-4 py-2.5 text-sm font-medium text-gray-300 bg-gray-800 border border-gray-700 rounded-lg hover:bg-gray-700 transition-colors"
              >
                Cancel
              </button>
              <button
                onClick={() => handleRevoke(showConfirmDialog)}
                disabled={revokingId === showConfirmDialog}
                className="flex-1 px-4 py-2.5 text-sm font-medium text-white bg-red-600 hover:bg-red-700 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {revokingId === showConfirmDialog ? 'Revoking...' : 'Revoke Access'}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Info Box */}
      <div className="bg-blue-500/10 border border-blue-500/20 rounded-xl p-4">
        <h4 className="text-sm font-semibold text-blue-400 mb-2">
          Security Note
        </h4>
        <p className="text-sm text-blue-300/80">
          If you notice any unfamiliar devices, revoke their access immediately and consider changing your password.
        </p>
      </div>
    </div>
  );
}
