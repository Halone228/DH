import { useOnlineStatus } from '../hooks/useOnlineStatus';

export function OfflineBanner() {
  const isOnline = useOnlineStatus();
  if (isOnline) return null;
  return (
    <div
      className="fixed top-0 left-0 right-0 z-50 py-2 text-center text-sm text-white"
      style={{ backgroundColor: 'var(--tg-destructive-text-color, #e53935)' }}
    >
      Нет подключения к интернету
    </div>
  );
}
