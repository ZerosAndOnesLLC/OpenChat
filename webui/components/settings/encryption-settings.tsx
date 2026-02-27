'use client';

import { useState, useEffect } from 'react';
import { Shield, Smartphone, Trash2, Check, Download, Upload, RefreshCw } from 'lucide-react';
import { apiClient } from '@/lib/api';
import type { CryptoDevice } from '@/lib/types';
import { getDeviceId, generateOneTimeKeys } from '@/lib/crypto';
import KeyBackupWizard from '../KeyBackupWizard';

export default function EncryptionSettings() {
  const [devices, setDevices] = useState<CryptoDevice[]>([]);
  const [loading, setLoading] = useState(true);
  const [showBackupWizard, setShowBackupWizard] = useState(false);
  const [showRestoreWizard, setShowRestoreWizard] = useState(false);

  const currentDeviceId = getDeviceId();

  useEffect(() => {
    loadDevices();
  }, []);

  const loadDevices = async () => {
    try {
      const data = await apiClient.listMyCryptoDevices();
      setDevices(data);
    } catch (err) {
      console.error('Failed to load devices:', err);
    } finally {
      setLoading(false);
    }
  };

  const handleVerify = async (deviceId: string) => {
    try {
      await apiClient.verifyCryptoDevice(deviceId);
      setDevices(prev => prev.map(d =>
        d.device_id === deviceId ? { ...d, verified: true } : d
      ));
    } catch (err) {
      console.error('Failed to verify device:', err);
    }
  };

  const handleRemove = async (deviceId: string) => {
    if (!confirm('Remove this device? It will no longer be able to decrypt messages.')) return;
    try {
      await apiClient.removeCryptoDevice(deviceId);
      setDevices(prev => prev.filter(d => d.device_id !== deviceId));
    } catch (err) {
      console.error('Failed to remove device:', err);
    }
  };

  const handleUploadKeys = async () => {
    if (!currentDeviceId) return;
    try {
      const otk = generateOneTimeKeys(10);
      await apiClient.uploadOneTimeKeys(currentDeviceId, otk);
    } catch (err) {
      console.error('Failed to upload keys:', err);
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center py-12">
        <RefreshCw className="h-6 w-6 animate-spin text-gray-400" />
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div>
        <h3 className="text-lg font-semibold text-white mb-1">Encryption</h3>
        <p className="text-sm text-gray-400">
          Manage your encryption devices and keys for end-to-end encrypted channels.
        </p>
      </div>

      {/* Devices */}
      <div>
        <h4 className="text-sm font-semibold text-gray-300 mb-3 flex items-center gap-2">
          <Smartphone className="h-4 w-4" />
          Your Devices
        </h4>
        {devices.length === 0 ? (
          <p className="text-sm text-gray-500">No encryption devices registered.</p>
        ) : (
          <div className="space-y-2">
            {devices.map((device) => (
              <div
                key={device.device_id}
                className="flex items-center justify-between rounded-lg border border-gray-700 bg-gray-800/50 px-4 py-3"
              >
                <div className="flex items-center gap-3">
                  <div className={`rounded-full p-2 ${device.verified ? 'bg-green-900/50 text-green-400' : 'bg-gray-700 text-gray-400'}`}>
                    <Shield className="h-4 w-4" />
                  </div>
                  <div>
                    <div className="flex items-center gap-2">
                      <span className="text-sm font-medium text-white">
                        {device.display_name || `Device ${device.device_id.slice(0, 8)}...`}
                      </span>
                      {device.device_id === currentDeviceId && (
                        <span className="rounded-full bg-blue-600/20 px-2 py-0.5 text-[10px] font-semibold text-blue-400">
                          This device
                        </span>
                      )}
                      {device.verified && (
                        <span className="rounded-full bg-green-600/20 px-2 py-0.5 text-[10px] font-semibold text-green-400">
                          Verified
                        </span>
                      )}
                    </div>
                    <p className="text-xs text-gray-500">
                      ID: {device.device_id.slice(0, 16)}... | Last seen: {new Date(device.last_seen_at).toLocaleDateString()}
                    </p>
                  </div>
                </div>
                <div className="flex items-center gap-2">
                  {!device.verified && device.device_id !== currentDeviceId && (
                    <button
                      onClick={() => handleVerify(device.device_id)}
                      className="rounded-md border border-green-600 px-3 py-1.5 text-xs font-medium text-green-400 hover:bg-green-900/30"
                      title="Verify device"
                    >
                      <Check className="h-3.5 w-3.5" />
                    </button>
                  )}
                  {device.device_id !== currentDeviceId && (
                    <button
                      onClick={() => handleRemove(device.device_id)}
                      className="rounded-md border border-red-600/50 px-3 py-1.5 text-xs font-medium text-red-400 hover:bg-red-900/30"
                      title="Remove device"
                    >
                      <Trash2 className="h-3.5 w-3.5" />
                    </button>
                  )}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Key Management */}
      <div>
        <h4 className="text-sm font-semibold text-gray-300 mb-3 flex items-center gap-2">
          <Shield className="h-4 w-4" />
          Key Management
        </h4>
        <div className="space-y-3">
          <button
            onClick={() => setShowBackupWizard(true)}
            className="flex w-full items-center gap-3 rounded-lg border border-gray-700 bg-gray-800/50 px-4 py-3 text-left hover:bg-gray-800 transition-colors"
          >
            <Download className="h-5 w-5 text-blue-400" />
            <div>
              <div className="text-sm font-medium text-white">Export Keys</div>
              <div className="text-xs text-gray-400">Back up your encryption keys with a passphrase</div>
            </div>
          </button>
          <button
            onClick={() => setShowRestoreWizard(true)}
            className="flex w-full items-center gap-3 rounded-lg border border-gray-700 bg-gray-800/50 px-4 py-3 text-left hover:bg-gray-800 transition-colors"
          >
            <Upload className="h-5 w-5 text-green-400" />
            <div>
              <div className="text-sm font-medium text-white">Import Keys</div>
              <div className="text-xs text-gray-400">Restore encryption keys from a backup</div>
            </div>
          </button>
          <button
            onClick={handleUploadKeys}
            className="flex w-full items-center gap-3 rounded-lg border border-gray-700 bg-gray-800/50 px-4 py-3 text-left hover:bg-gray-800 transition-colors"
          >
            <RefreshCw className="h-5 w-5 text-yellow-400" />
            <div>
              <div className="text-sm font-medium text-white">Replenish One-Time Keys</div>
              <div className="text-xs text-gray-400">Generate and upload fresh one-time keys</div>
            </div>
          </button>
        </div>
      </div>

      {showBackupWizard && (
        <KeyBackupWizard
          mode="export"
          onClose={() => setShowBackupWizard(false)}
        />
      )}
      {showRestoreWizard && (
        <KeyBackupWizard
          mode="import"
          onClose={() => setShowRestoreWizard(false)}
        />
      )}
    </div>
  );
}
