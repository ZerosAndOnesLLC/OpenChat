// Global toast manager
export type ToastType = 'success' | 'error' | 'info' | 'warning';

export interface ToastMessage {
  id: string;
  message: string;
  type: ToastType;
}

type ToastListener = (toast: ToastMessage) => void;

class ToastManager {
  private listeners: Set<ToastListener> = new Set();

  subscribe(listener: ToastListener) {
    this.listeners.add(listener);
    return () => {
      this.listeners.delete(listener);
    };
  }

  show(message: string, type: ToastType = 'info') {
    const toast: ToastMessage = {
      id: `${Date.now()}-${Math.random()}`,
      message,
      type,
    };

    this.listeners.forEach(listener => listener(toast));
  }

  success(message: string) {
    this.show(message, 'success');
  }

  error(message: string) {
    this.show(message, 'error');
  }

  info(message: string) {
    this.show(message, 'info');
  }

  warning(message: string) {
    this.show(message, 'warning');
  }
}

export const toastManager = new ToastManager();

// Expose globally for use in API client
if (typeof window !== 'undefined') {
  (window as any).showToast = (message: string, type: ToastType = 'info') => {
    toastManager.show(message, type);
  };
}
