'use client';

import { useState } from 'react';
import { X, Copy, Check, AlertTriangle } from 'lucide-react';
import { exportKeys, importKeys } from '@/lib/crypto';

interface KeyBackupWizardProps {
  mode: 'export' | 'import';
  onClose: () => void;
}

export default function KeyBackupWizard({ mode, onClose }: KeyBackupWizardProps) {
  const [step, setStep] = useState(1);
  const [passphrase, setPassphrase] = useState('');
  const [confirmPassphrase, setConfirmPassphrase] = useState('');
  const [backupData, setBackupData] = useState('');
  const [importData, setImportData] = useState('');
  const [error, setError] = useState('');
  const [copied, setCopied] = useState(false);
  const [loading, setLoading] = useState(false);

  const handleExport = async () => {
    if (passphrase.length < 8) {
      setError('Passphrase must be at least 8 characters');
      return;
    }
    if (passphrase !== confirmPassphrase) {
      setError('Passphrases do not match');
      return;
    }
    setError('');
    setLoading(true);
    try {
      const data = await exportKeys(passphrase);
      setBackupData(data);
      setStep(2);
    } catch (err) {
      setError('Failed to export keys. Make sure crypto is initialized.');
    } finally {
      setLoading(false);
    }
  };

  const handleImport = async () => {
    if (!importData.trim()) {
      setError('Please paste your backup data');
      return;
    }
    if (!passphrase) {
      setError('Please enter your passphrase');
      return;
    }
    setError('');
    setLoading(true);
    try {
      await importKeys(importData, passphrase);
      setStep(2);
    } catch (err) {
      setError('Failed to import keys. Check your backup data and passphrase.');
    } finally {
      setLoading(false);
    }
  };

  const handleCopy = async () => {
    await navigator.clipboard.writeText(backupData);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60">
      <div className="w-full max-w-md rounded-lg border border-gray-700 bg-gray-900 p-6">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold text-white">
            {mode === 'export' ? 'Export Encryption Keys' : 'Import Encryption Keys'}
          </h3>
          <button onClick={onClose} className="text-gray-400 hover:text-white">
            <X className="h-5 w-5" />
          </button>
        </div>

        {mode === 'export' && step === 1 && (
          <div className="space-y-4">
            <div className="flex items-start gap-2 rounded-lg bg-yellow-900/20 border border-yellow-700/30 p-3">
              <AlertTriangle className="h-5 w-5 flex-shrink-0 text-yellow-400 mt-0.5" />
              <p className="text-sm text-yellow-300">
                This passphrase protects your encryption keys. If you lose it, you won't be able to restore your keys.
              </p>
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-300 mb-1">Passphrase</label>
              <input
                type="password"
                value={passphrase}
                onChange={(e) => setPassphrase(e.target.value)}
                placeholder="Enter a strong passphrase"
                className="w-full rounded-md border border-gray-600 bg-gray-800 px-3 py-2 text-sm text-white placeholder-gray-500 focus:border-blue-500 focus:outline-none"
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-300 mb-1">Confirm Passphrase</label>
              <input
                type="password"
                value={confirmPassphrase}
                onChange={(e) => setConfirmPassphrase(e.target.value)}
                placeholder="Confirm your passphrase"
                className="w-full rounded-md border border-gray-600 bg-gray-800 px-3 py-2 text-sm text-white placeholder-gray-500 focus:border-blue-500 focus:outline-none"
              />
            </div>
            {error && <p className="text-sm text-red-400">{error}</p>}
            <button
              onClick={handleExport}
              disabled={loading}
              className="w-full rounded-md bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700 disabled:bg-gray-700 disabled:text-gray-400"
            >
              {loading ? 'Exporting...' : 'Export Keys'}
            </button>
          </div>
        )}

        {mode === 'export' && step === 2 && (
          <div className="space-y-4">
            <p className="text-sm text-gray-300">
              Copy and save this backup data in a secure location.
            </p>
            <div className="relative">
              <textarea
                readOnly
                value={backupData}
                className="w-full h-32 rounded-md border border-gray-600 bg-gray-800 px-3 py-2 text-xs text-gray-300 font-mono resize-none focus:outline-none"
              />
              <button
                onClick={handleCopy}
                className="absolute top-2 right-2 rounded-md bg-gray-700 p-1.5 text-gray-300 hover:bg-gray-600"
                title="Copy to clipboard"
              >
                {copied ? <Check className="h-4 w-4 text-green-400" /> : <Copy className="h-4 w-4" />}
              </button>
            </div>
            <button
              onClick={onClose}
              className="w-full rounded-md bg-green-600 px-4 py-2 text-sm font-medium text-white hover:bg-green-700"
            >
              Done
            </button>
          </div>
        )}

        {mode === 'import' && step === 1 && (
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-300 mb-1">Backup Data</label>
              <textarea
                value={importData}
                onChange={(e) => setImportData(e.target.value)}
                placeholder="Paste your backup data here"
                className="w-full h-32 rounded-md border border-gray-600 bg-gray-800 px-3 py-2 text-xs text-white font-mono placeholder-gray-500 resize-none focus:border-blue-500 focus:outline-none"
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-300 mb-1">Passphrase</label>
              <input
                type="password"
                value={passphrase}
                onChange={(e) => setPassphrase(e.target.value)}
                placeholder="Enter the passphrase used during export"
                className="w-full rounded-md border border-gray-600 bg-gray-800 px-3 py-2 text-sm text-white placeholder-gray-500 focus:border-blue-500 focus:outline-none"
              />
            </div>
            {error && <p className="text-sm text-red-400">{error}</p>}
            <button
              onClick={handleImport}
              disabled={loading}
              className="w-full rounded-md bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700 disabled:bg-gray-700 disabled:text-gray-400"
            >
              {loading ? 'Importing...' : 'Import Keys'}
            </button>
          </div>
        )}

        {mode === 'import' && step === 2 && (
          <div className="space-y-4">
            <div className="flex items-center gap-3 rounded-lg bg-green-900/20 border border-green-700/30 p-4">
              <Check className="h-6 w-6 text-green-400" />
              <p className="text-sm text-green-300">Keys imported successfully! You can now decrypt messages.</p>
            </div>
            <button
              onClick={onClose}
              className="w-full rounded-md bg-green-600 px-4 py-2 text-sm font-medium text-white hover:bg-green-700"
            >
              Done
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
