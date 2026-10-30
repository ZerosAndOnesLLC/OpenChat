'use client';

import { useState, useEffect, useEffectEvent, useRef } from 'react';

interface ScannerProps {
  onCodeScanned: (code: string) => void;
  onClose: () => void;
}

/** Pairing code from an `openchat://pair?code=ABC123` QR payload, or an error message. */
function parsePairingQr(data: string): { code: string } | { error: string } {
  try {
    const url = new URL(data);
    if (url.protocol === 'openchat:' && url.hostname === 'pair') {
      const code = url.searchParams.get('code');
      if (code && code.length === 6) {
        return { code };
      }
      return { error: 'Invalid QR code format. Please scan a valid OpenChat pairing code.' };
    }
    return { error: 'This is not an OpenChat pairing code. Please scan the correct QR code.' };
  } catch {
    return { error: 'Invalid QR code format. Please scan a valid OpenChat pairing code.' };
  }
}

export default function Scanner({ onCodeScanned, onClose }: ScannerProps) {
  const [error, setError] = useState<string | null>(null);
  const [hasPermission, setHasPermission] = useState<boolean | null>(null);
  const [scanning, setScanning] = useState(false);
  const videoRef = useRef<HTMLVideoElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const streamRef = useRef<MediaStream | null>(null);

  const stopCamera = () => {
    setScanning(false);
    if (streamRef.current) {
      streamRef.current.getTracks().forEach((track) => track.stop());
      streamRef.current = null;
    }
  };

  // Request the camera on mount; release it on unmount.
  useEffect(() => {
    let cancelled = false;
    navigator.mediaDevices
      .getUserMedia({ video: { facingMode: 'environment' } })
      .then((stream) => {
        if (cancelled) {
          stream.getTracks().forEach((track) => track.stop());
          return;
        }
        streamRef.current = stream;
        setHasPermission(true);
        setScanning(true);
      })
      .catch((err) => {
        if (cancelled) return;
        console.error('Camera access error:', err);
        setError('Unable to access camera. Please check permissions.');
        setHasPermission(false);
      });
    return () => {
      cancelled = true;
      streamRef.current?.getTracks().forEach((track) => track.stop());
      streamRef.current = null;
    };
  }, []);

  // The <video> only mounts once permission is granted; attach the stream then.
  useEffect(() => {
    if (hasPermission && videoRef.current && streamRef.current) {
      videoRef.current.srcObject = streamRef.current;
    }
  }, [hasPermission]);

  const handleQRCodeDetected = useEffectEvent((data: string) => {
    const result = parsePairingQr(data);
    if ('code' in result) {
      stopCamera();
      onCodeScanned(result.code);
      onClose();
    } else {
      // Keep scanning so the user can present the right code.
      setError(result.error);
    }
  });

  const scanQRCode = useEffectEvent(async () => {
    if (!videoRef.current || !canvasRef.current) return;

    const video = videoRef.current;
    const canvas = canvasRef.current;
    const context = canvas.getContext('2d');

    if (!context || video.readyState !== video.HAVE_ENOUGH_DATA) return;

    canvas.width = video.videoWidth;
    canvas.height = video.videoHeight;
    context.drawImage(video, 0, 0, canvas.width, canvas.height);

    const imageData = context.getImageData(0, 0, canvas.width, canvas.height);

    try {
      // Use jsQR library to decode QR code
      const jsQR = (await import('jsqr')).default;
      const code = jsQR(imageData.data, imageData.width, imageData.height, {
        inversionAttempts: 'dontInvert',
      });

      if (code && code.data) {
        handleQRCodeDetected(code.data);
      }
    } catch (err) {
      console.error('QR scanning error:', err);
    }
  });

  // Scan for QR codes every 500ms while scanning is active.
  useEffect(() => {
    if (!scanning) return;
    const interval = setInterval(() => {
      scanQRCode();
    }, 500);
    return () => clearInterval(interval);
  }, [scanning]);

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-90">
      <div className="relative w-full max-w-2xl mx-4">
        <button
          onClick={() => {
            stopCamera();
            onClose();
          }}
          className="absolute top-4 right-4 z-10 bg-white dark:bg-gray-800 text-gray-900 dark:text-white p-2 rounded-full hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
          aria-label="Close scanner"
        >
          <svg
            className="w-6 h-6"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M6 18L18 6M6 6l12 12"
            />
          </svg>
        </button>

        <div className="bg-white dark:bg-gray-800 rounded-2xl overflow-hidden shadow-2xl">
          <div className="p-6">
            <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-4 text-center">
              Scan QR Code
            </h2>

            {hasPermission === null && (
              <div className="text-center py-8">
                <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto"></div>
                <p className="mt-4 text-gray-600 dark:text-gray-400">
                  Requesting camera access...
                </p>
              </div>
            )}

            {hasPermission === false && (
              <div className="text-center py-8">
                <svg
                  className="w-16 h-16 text-red-500 mx-auto mb-4"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
                  />
                </svg>
                <p className="text-red-600 dark:text-red-400 font-semibold mb-2">
                  Camera Access Denied
                </p>
                <p className="text-gray-600 dark:text-gray-400 text-sm">
                  Please allow camera access in your system settings to scan QR codes.
                </p>
              </div>
            )}

            {hasPermission && (
              <div className="relative">
                <video
                  ref={videoRef}
                  autoPlay
                  playsInline
                  muted
                  className="w-full rounded-lg"
                  style={{ maxHeight: '500px' }}
                />
                <canvas ref={canvasRef} className="hidden" />

                {/* Scanning overlay */}
                <div className="absolute inset-0 flex items-center justify-center pointer-events-none">
                  <div className="relative w-64 h-64">
                    <div className="absolute inset-0 border-2 border-blue-500 rounded-lg">
                      {/* Corner decorations */}
                      <div className="absolute top-0 left-0 w-8 h-8 border-t-4 border-l-4 border-blue-500"></div>
                      <div className="absolute top-0 right-0 w-8 h-8 border-t-4 border-r-4 border-blue-500"></div>
                      <div className="absolute bottom-0 left-0 w-8 h-8 border-b-4 border-l-4 border-blue-500"></div>
                      <div className="absolute bottom-0 right-0 w-8 h-8 border-b-4 border-r-4 border-blue-500"></div>
                    </div>
                    {scanning && (
                      <div className="absolute inset-0 flex items-center justify-center">
                        <div className="w-full h-1 bg-blue-500 opacity-75 animate-pulse"></div>
                      </div>
                    )}
                  </div>
                </div>

                <p className="text-center mt-4 text-gray-600 dark:text-gray-400">
                  Position the QR code within the frame
                </p>
              </div>
            )}

            {error && (
              <div className="mt-4 bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-4">
                <p className="text-sm text-yellow-800 dark:text-yellow-200">{error}</p>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
