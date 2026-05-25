import { useEffect } from 'react';
import { useLocation, useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { useTelegram } from '../hooks/useTelegram';

const NAV_ITEMS = [
  { path: '/', labelKey: 'nav.main', icon: '📋' },
  { path: '/settings', labelKey: 'nav.settings', icon: '⚙️' },
  { path: '/profile', labelKey: 'nav.profile', icon: '👤' },
] as const;

export function Layout({ children }: { children: React.ReactNode }) {
  const { t } = useTranslation();
  const location = useLocation();
  const navigate = useNavigate();
  const { tg } = useTelegram();
  const isRoot = location.pathname === '/';

  useEffect(() => {
    if (!tg) return;
    if (isRoot) {
      tg.BackButton.hide();
    } else {
      tg.BackButton.show();
    }
  }, [tg, isRoot]);

  useEffect(() => {
    if (!tg) return;
    const handler = () => navigate(-1);
    tg.BackButton.onClick(handler);
    return () => tg.BackButton.offClick(handler);
  }, [tg, navigate]);

  return (
    <div className="min-h-screen bg-[var(--tg-bg-color)] text-[var(--tg-text-color)] font-sans">
      <main className="pb-20">{children}</main>
      {isRoot && (
        <nav className="fixed bottom-0 left-0 right-0 bg-[var(--tg-bg-color)] border-t border-[var(--tg-secondary-bg-color)] flex justify-around py-1 px-2 z-50">
          {NAV_ITEMS.map((item) => {
            const active = location.pathname === item.path;
            return (
              <button
                key={item.path}
                onClick={() => navigate(item.path)}
                className={`flex flex-col items-center py-1.5 px-3 text-xs transition-colors ${
                  active
                    ? 'text-[var(--tg-button-color)]'
                    : 'text-[var(--tg-hint-color)]'
                }`}
              >
                <span className="text-lg mb-0.5">{item.icon}</span>
                <span>{t(item.labelKey)}</span>
              </button>
            );
          })}
        </nav>
      )}
    </div>
  );
}
