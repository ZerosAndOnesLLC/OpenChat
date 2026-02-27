'use client';

import { useState } from 'react';
import { X, Shield, ShieldCheck } from 'lucide-react';
import { apiClient } from '@/lib/api';
import type { CryptoDevice } from '@/lib/types';

interface DeviceVerificationProps {
  device: CryptoDevice;
  isOpen: boolean;
  onClose: () => void;
  onVerified?: () => void;
}

function generateSafetyNumber(identityKey: string, deviceId: string): string {
  // Generate a visual safety number from the identity key
  // Uses first 40 chars of the identity key hash as 8 groups of 5 digits
  const combined = identityKey + deviceId;
  let hash = 0;
  for (let i = 0; i < combined.length; i++) {
    const char = combined.charCodeAt(i);
    hash = ((hash << 5) - hash) + char;
    hash = hash & hash; // Convert to 32bit integer
  }

  const groups: string[] = [];
  let seed = Math.abs(hash);
  for (let i = 0; i < 8; i++) {
    seed = (seed * 16807 + 12345) & 0x7fffffff;
    groups.push(String(seed % 100000).padStart(5, '0'));
  }
  return groups.join(' ');
}

export default function DeviceVerification({ device, isOpen, onClose, onVerified }: DeviceVerificationProps) {
  const [verifying, setVerifying] = useState(false);

  if (!isOpen) return null;

  const safetyNumber = generateSafetyNumber(device.identity_key, device.device_id);

  const handleVerify = async () => {
    setVerifying(true);
    try {
      await apiClient.verifyCryptoDevice(device.device_id);
      onVerified?.();
      onClose();
    } catch (err) {
      console.error('Failed to verify device:', err);
    } finally {
      setVerifying(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60">
      <div className="w-full max-w-md rounded-lg border border-gray-700 bg-gray-900 p-6">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold text-white flex items-center gap-2">
            <Shield className="h-5 w-5 text-blue-400" />
            Verify Device
          </h3>
          <button onClick={onClose} className="text-gray-400 hover:text-white">
            <X className="h-5 w-5" />
          </button>
        </div>

        <div className="space-y-4">
          <div>
            <p className="text-sm text-gray-300 mb-1">Device</p>
            <p className="text-sm font-medium text-white">
              {device.display_name || `Device ${device.device_id.slice(0, 8)}...`}
            </p>
            <p className="text-xs text-gray-500 mt-0.5">ID: {device.device_id}</p>
          </div>

          <div>
            <p className="text-sm text-gray-300 mb-2">Safety Number</p>
            <p className="text-xs text-gray-400 mb-2">
              Compare this number with the one shown on the other device. If they match, the device is verified.
            </p>
            <div className="rounded-lg bg-gray-800 border border-gray-700 p-4 text-center">
              <p className="font-mono text-lg text-white tracking-widest leading-relaxed">
                {safetyNumber}
              </p>
            </div>
          </div>

          <div className="flex gap-3 pt-2">
            <button
              onClick={onClose}
              className="flex-1 rounded-md border border-gray-600 px-4 py-2 text-sm font-medium text-gray-300 hover:bg-gray-800"
            >
              Cancel
            </button>
            <button
              onClick={handleVerify}
              disabled={verifying}
              className="flex-1 flex items-center justify-center gap-2 rounded-md bg-green-600 px-4 py-2 text-sm font-medium text-white hover:bg-green-700 disabled:bg-gray-700 disabled:text-gray-400"
            >
              <ShieldCheck className="h-4 w-4" />
              {verifying ? 'Verifying...' : 'Mark as Verified'}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
