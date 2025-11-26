'use client';

import { useState, useEffect, useCallback } from 'react';
import { QRCodeSVG } from 'qrcode.react';
import { apiClient } from '@/lib/api';
import { Copy, Check, Monitor, QrCode, Smartphone, RefreshCw, ExternalLink, AlertCircle } from 'lucide-react';

interface PairingCodeData {
  code: string;
  expiresAt: number;
}

export default function DesktopLogin() {
  const [activeTab, setActiveTab] = useState<'pairing' | 'deeplink'>('pairing');
  const [pairingData, setPairingData] = useState<PairingCodeData | null>(null);
  const [timeRemaining, setTimeRemaining] = useState<number>(0);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);

  const generateCode = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      const response = await apiClient.generatePairingCode();
      const expiresAt = Date.now() + response.expires_in * 1000;
      setPairingData({
        code: response.code,
        expiresAt,
      });
      setTimeRemaining(response.expires_in);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to generate pairing code');
      setPairingData(null);
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    if (!pairingData) return;

    const interval = setInterval(() => {
      const remaining = Math.max(0, Math.floor((pairingData.expiresAt - Date.now()) / 1000));
      setTimeRemaining(remaining);

      if (remaining === 0) {
        setPairingData(null);
      }
    }, 1000);

    return () => clearInterval(interval);
  }, [pairingData]);

  const handleCopyCode = async () => {
    if (!pairingData) return;
    try {
      await navigator.clipboard.writeText(pairingData.code);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error('Failed to copy code:', err);
    }
  };

  const handleOpenDeepLink = () => {
    const token = apiClient.getToken();
    if (!token) {
      setError('No authentication token found');
      return;
    }

    // For now, we'll use a simple base64 encoding
    // In production, this should be encrypted
    const payload = btoa(JSON.stringify({ token }));
    const deepLinkUrl = `openchat://login?payload=${payload}`;

    window.location.href = deepLinkUrl;
  };

  const formatTime = (seconds: number): string => {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  };

  const qrCodeUrl = pairingData ? `openchat://pair?code=${pairingData.code}` : '';

  return (
    <div className="w-full max-w-2xl mx-auto">
      <div className="bg-white dark:bg-gray-800 rounded-2xl shadow-xl overflow-hidden border border-gray-200 dark:border-gray-700">
        {/* Header */}
        <div className="bg-gradient-to-r from-blue-600 to-indigo-600 px-8 py-6">
          <div className="flex items-center gap-3">
            <Monitor className="w-8 h-8 text-white" />
            <div>
              <h2 className="text-2xl font-bold text-white">Desktop App Login</h2>
              <p className="text-blue-100 text-sm mt-1">Connect your desktop app in seconds</p>
            </div>
          </div>
        </div>

        {/* Tabs */}
        <div className="border-b border-gray-200 dark:border-gray-700">
          <div className="flex">
            <button
              onClick={() => setActiveTab('pairing')}
              className={`flex-1 px-6 py-4 text-sm font-medium transition-all ${
                activeTab === 'pairing'
                  ? 'text-blue-600 dark:text-blue-400 border-b-2 border-blue-600 dark:border-blue-400 bg-blue-50 dark:bg-blue-900/20'
                  : 'text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200 hover:bg-gray-50 dark:hover:bg-gray-700/50'
              }`}
            >
              <div className="flex items-center justify-center gap-2">
                <QrCode className="w-4 h-4" />
                Pairing Code
              </div>
            </button>
            <button
              onClick={() => setActiveTab('deeplink')}
              className={`flex-1 px-6 py-4 text-sm font-medium transition-all ${
                activeTab === 'deeplink'
                  ? 'text-blue-600 dark:text-blue-400 border-b-2 border-blue-600 dark:border-blue-400 bg-blue-50 dark:bg-blue-900/20'
                  : 'text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200 hover:bg-gray-50 dark:hover:bg-gray-700/50'
              }`}
            >
              <div className="flex items-center justify-center gap-2">
                <ExternalLink className="w-4 h-4" />
                Quick Login
              </div>
            </button>
          </div>
        </div>

        {/* Content */}
        <div className="px-8 py-8">
          {error && (
            <div className="mb-6 p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg flex items-start gap-3">
              <AlertCircle className="w-5 h-5 text-red-600 dark:text-red-400 flex-shrink-0 mt-0.5" />
              <div>
                <p className="text-sm font-medium text-red-800 dark:text-red-200">Error</p>
                <p className="text-sm text-red-700 dark:text-red-300 mt-1">{error}</p>
              </div>
            </div>
          )}

          {activeTab === 'pairing' && (
            <div className="space-y-6">
              {!pairingData ? (
                <div className="text-center py-12">
                  <div className="inline-flex items-center justify-center w-20 h-20 bg-blue-100 dark:bg-blue-900/30 rounded-full mb-6">
                    <Smartphone className="w-10 h-10 text-blue-600 dark:text-blue-400" />
                  </div>
                  <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-2">
                    Ready to Connect
                  </h3>
                  <p className="text-gray-600 dark:text-gray-400 mb-8 max-w-md mx-auto">
                    Generate a pairing code to log into your desktop app. The code will be valid for 5 minutes.
                  </p>
                  <button
                    onClick={generateCode}
                    disabled={isLoading}
                    className="inline-flex items-center gap-2 px-8 py-3.5 bg-blue-600 hover:bg-blue-700 disabled:bg-blue-400 text-white font-semibold rounded-xl transition-all shadow-lg hover:shadow-xl hover:scale-105 disabled:scale-100"
                  >
                    {isLoading ? (
                      <>
                        <RefreshCw className="w-5 h-5 animate-spin" />
                        Generating...
                      </>
                    ) : (
                      <>
                        <QrCode className="w-5 h-5" />
                        Generate Pairing Code
                      </>
                    )}
                  </button>
                </div>
              ) : (
                <div className="space-y-8">
                  {/* Timer */}
                  <div className="text-center">
                    <div className="inline-flex items-center gap-2 px-4 py-2 bg-amber-100 dark:bg-amber-900/30 text-amber-800 dark:text-amber-200 rounded-full font-mono font-semibold">
                      <div className={`w-2 h-2 rounded-full ${timeRemaining > 60 ? 'bg-green-500' : 'bg-amber-500 animate-pulse'}`} />
                      Expires in {formatTime(timeRemaining)}
                    </div>
                  </div>

                  {/* Code Display */}
                  <div className="bg-gradient-to-br from-gray-50 to-gray-100 dark:from-gray-900 dark:to-gray-800 rounded-2xl p-8 text-center border-2 border-dashed border-gray-300 dark:border-gray-600">
                    <p className="text-sm font-medium text-gray-600 dark:text-gray-400 mb-3">
                      Your Pairing Code
                    </p>
                    <div className="text-6xl font-bold tracking-wider text-gray-900 dark:text-white font-mono mb-6 select-all">
                      {pairingData.code}
                    </div>
                    <button
                      onClick={handleCopyCode}
                      className="inline-flex items-center gap-2 px-6 py-2.5 bg-white dark:bg-gray-700 hover:bg-gray-50 dark:hover:bg-gray-600 border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-200 font-medium rounded-lg transition-all shadow-sm hover:shadow"
                    >
                      {copied ? (
                        <>
                          <Check className="w-4 h-4 text-green-600 dark:text-green-400" />
                          Copied!
                        </>
                      ) : (
                        <>
                          <Copy className="w-4 h-4" />
                          Copy Code
                        </>
                      )}
                    </button>
                  </div>

                  {/* QR Code */}
                  <div className="flex flex-col items-center">
                    <p className="text-sm font-medium text-gray-600 dark:text-gray-400 mb-4">
                      Or scan this QR code
                    </p>
                    <div className="bg-white p-6 rounded-2xl shadow-lg border border-gray-200">
                      <QRCodeSVG
                        value={qrCodeUrl}
                        size={200}
                        level="H"
                        includeMargin={false}
                      />
                    </div>
                  </div>

                  {/* Instructions */}
                  <div className="bg-blue-50 dark:bg-blue-900/20 rounded-xl p-6 border border-blue-200 dark:border-blue-800">
                    <h4 className="font-semibold text-blue-900 dark:text-blue-100 mb-3">How to use:</h4>
                    <ol className="space-y-2 text-sm text-blue-800 dark:text-blue-200">
                      <li className="flex gap-3">
                        <span className="font-bold">1.</span>
                        <span>Open the OpenChat desktop app</span>
                      </li>
                      <li className="flex gap-3">
                        <span className="font-bold">2.</span>
                        <span>Enter the code above or scan the QR code</span>
                      </li>
                      <li className="flex gap-3">
                        <span className="font-bold">3.</span>
                        <span>You'll be logged in automatically</span>
                      </li>
                    </ol>
                  </div>

                  {/* Regenerate Button */}
                  <div className="text-center">
                    <button
                      onClick={generateCode}
                      disabled={isLoading}
                      className="text-sm text-blue-600 dark:text-blue-400 hover:text-blue-700 dark:hover:text-blue-300 font-medium inline-flex items-center gap-2"
                    >
                      <RefreshCw className="w-4 h-4" />
                      Generate New Code
                    </button>
                  </div>
                </div>
              )}
            </div>
          )}

          {activeTab === 'deeplink' && (
            <div className="text-center py-12">
              <div className="inline-flex items-center justify-center w-20 h-20 bg-gradient-to-br from-blue-500 to-indigo-600 rounded-full mb-6">
                <ExternalLink className="w-10 h-10 text-white" />
              </div>
              <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-2">
                One-Click Login
              </h3>
              <p className="text-gray-600 dark:text-gray-400 mb-8 max-w-md mx-auto">
                Click the button below to open the desktop app and log in instantly with your current session.
              </p>
              <button
                onClick={handleOpenDeepLink}
                className="inline-flex items-center gap-3 px-8 py-3.5 bg-gradient-to-r from-blue-600 to-indigo-600 hover:from-blue-700 hover:to-indigo-700 text-white font-semibold rounded-xl transition-all shadow-lg hover:shadow-xl hover:scale-105"
              >
                <Monitor className="w-5 h-5" />
                Open Desktop App
              </button>

              <div className="mt-10 bg-gray-50 dark:bg-gray-900/50 rounded-xl p-6 border border-gray-200 dark:border-gray-700 max-w-md mx-auto">
                <p className="text-xs text-gray-500 dark:text-gray-400 mb-3">
                  Don't have the desktop app installed?
                </p>
                <a
                  href="https://openchat.com/download"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-sm text-blue-600 dark:text-blue-400 hover:text-blue-700 dark:hover:text-blue-300 font-medium inline-flex items-center gap-1"
                >
                  Download it here
                  <ExternalLink className="w-3 h-3" />
                </a>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
