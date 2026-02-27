'use client';

import { useEffect, useRef } from 'react';
import { useAuth } from '../lib/auth';
import { initCrypto, generateOneTimeKeys, getDeviceId } from '../lib/crypto';
import { apiClient } from '../lib/api';

export function useCryptoInit() {
  const { user } = useAuth();
  const initialized = useRef(false);

  useEffect(() => {
    if (!user || initialized.current) return;
    initialized.current = true;

    (async () => {
      try {
        const keys = await initCrypto(user.id);
        if (!keys) return;

        // Register device with server
        const otk = generateOneTimeKeys(10);
        await apiClient.registerCryptoDevice({
          device_id: keys.device_id,
          identity_key: keys.identity_key,
          signing_key: keys.signing_key,
          one_time_keys: otk,
        });
      } catch (err) {
        console.error('Failed to initialize crypto:', err);
      }
    })();
  }, [user]);

  return { deviceId: getDeviceId() };
}
