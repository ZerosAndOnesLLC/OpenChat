import { useEffect, useState, useCallback } from 'react';

export const useNotificationSound = () => {
  const [enabled, setEnabled] = useState(false);
  const [audio, setAudio] = useState<HTMLAudioElement | null>(null);

  useEffect(() => {
    // Load preference from localStorage
    const stored = localStorage.getItem('notificationSoundEnabled');
    setEnabled(stored === 'true');

    // Create audio element with a simple notification sound
    // Using a data URI for a simple beep sound
    const audioElement = new Audio();
    audioElement.volume = 0.5;
    setAudio(audioElement);

    return () => {
      if (audioElement) {
        audioElement.pause();
        audioElement.src = '';
      }
    };
  }, []);

  const playNotificationSound = useCallback(() => {
    if (!enabled || !audio) return;

    // Generate a simple beep using Web Audio API as fallback
    try {
      const audioContext = new (window.AudioContext || (window as any).webkitAudioContext)();
      const oscillator = audioContext.createOscillator();
      const gainNode = audioContext.createGain();

      oscillator.connect(gainNode);
      gainNode.connect(audioContext.destination);

      oscillator.frequency.value = 800;
      oscillator.type = 'sine';

      gainNode.gain.setValueAtTime(0.3, audioContext.currentTime);
      gainNode.gain.exponentialRampToValueAtTime(0.01, audioContext.currentTime + 0.2);

      oscillator.start(audioContext.currentTime);
      oscillator.stop(audioContext.currentTime + 0.2);
    } catch (error) {
      console.error('Failed to play notification sound:', error);
    }
  }, [enabled, audio]);

  const toggleSound = useCallback((enable: boolean) => {
    setEnabled(enable);
    localStorage.setItem('notificationSoundEnabled', String(enable));
  }, []);

  return {
    enabled,
    toggleSound,
    playNotificationSound
  };
};
