import { useEffect, useState, useCallback } from 'react';
import { api } from '../api/client';
import { useTelegram } from '../hooks/useTelegram';
import { t } from '../i18n/ru';

const COMMON_ZONES = [
  'Europe/Moscow',
  'Europe/Samara',
  'Asia/Yekaterinburg',
  'Asia/Omsk',
  'Asia/Novosibirsk',
  'Asia/Krasnoyarsk',
  'Asia/Irkutsk',
  'Asia/Yakutsk',
  'Asia/Vladivostok',
  'Asia/Magadan',
  'Asia/Kamchatka',
  'Europe/Kaliningrad',
  'Europe/Berlin',
  'Europe/London',
  'US/Eastern',
  'US/Pacific',
  'UTC',
];

function getAllTimezones(): string[] {
  try {
    const all = Intl.supportedValuesOf('timeZone') as string[];
    return all;
  } catch {
    return COMMON_ZONES;
  }
}

export function Profile() {
  const { tg, user } = useTelegram();
  const [timezone, setTimezone] = useState('Europe/Moscow');
  const [originalTz, setOriginalTz] = useState('Europe/Moscow');
  const [allZones] = useState(getAllTimezones);
  const [search, setSearch] = useState('');
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchMe = useCallback(async () => {
    try {
      const data = await api.getMe();
      const tz = data?.timezone ?? 'Europe/Moscow';
      setTimezone(tz);
      setOriginalTz(tz);
    } catch {
      setError(t.error.network);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchMe();
  }, [fetchMe]);

  const isDirty = timezone !== originalTz;

  useEffect(() => {
    if (!tg) return;
    if (isDirty) {
      tg.MainButton.setParams({
        text: t.settings.save,
        color: 'var(--tg-button-color)',
        text_color: 'var(--tg-button-text-color)',
        is_visible: true,
      });
      tg.MainButton.enable();
    } else {
      tg.MainButton.hide?.();
    }
  }, [tg, isDirty]);

  useEffect(() => {
    if (!tg) return;
    const handler = async () => {
      if (!isDirty || saving) return;
      setSaving(true);
      try {
        await api.updateTimezone(timezone);
        setOriginalTz(timezone);
        tg.HapticFeedback?.notificationOccurred('success');
      } catch {
        setError(t.error.network);
      } finally {
        setSaving(false);
      }
    };
    tg.MainButton.onClick(handler);
    return () => tg.MainButton.offClick(handler);
  });

  const filteredCommon = COMMON_ZONES.filter((z) =>
    z.toLowerCase().includes(search.toLowerCase()),
  );
  const filteredOther = allZones
    .filter((z) => !COMMON_ZONES.includes(z))
    .filter((z) => z.toLowerCase().includes(search.toLowerCase()));

  if (loading) {
    return (
      <div className="flex items-center justify-center py-20">
        <div className="text-[var(--tg-hint-color)]">Загрузка...</div>
      </div>
    );
  }

  return (
    <div className="px-4 pt-4">
      <h1 className="text-xl font-bold mb-4 text-[var(--tg-text-color)]">
        {t.nav.profile}
      </h1>

      {error && (
        <div className="mb-4 p-3 rounded-lg bg-red-50 text-[var(--tg-destructive-color)] text-sm">
          {error}
        </div>
      )}

      <div className="space-y-5">
        {/* User info */}
        {user && (
          <div className="p-4 rounded-xl bg-[var(--tg-secondary-bg-color)]">
            <p className="text-lg font-medium text-[var(--tg-text-color)]">
              {user.first_name}
              {user.last_name ? ` ${user.last_name}` : ''}
            </p>
            {user.username && (
              <p className="text-sm text-[var(--tg-hint-color)] mt-0.5">
                @{user.username}
              </p>
            )}
          </div>
        )}

        {/* Timezone */}
        <div>
          <label className="block text-sm font-medium text-[var(--tg-text-color)] mb-1.5">
            {t.profile.timezone}
          </label>

          <input
            type="text"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Поиск..."
            className="w-full rounded-xl px-3 py-2.5 text-sm bg-[var(--tg-secondary-bg-color)] text-[var(--tg-text-color)] placeholder-[var(--tg-hint-color)] outline-none focus:ring-2 focus:ring-[var(--tg-button-color)] mb-2"
          />

          <div className="max-h-64 overflow-y-auto rounded-xl border border-[var(--tg-secondary-bg-color)]">
            {filteredCommon.map((z) => (
              <button
                key={z}
                onClick={() => setTimezone(z)}
                className={`w-full text-left px-3 py-2.5 text-sm transition-colors ${
                  timezone === z
                    ? 'bg-[var(--tg-button-color)] text-[var(--tg-button-text-color)]'
                    : 'text-[var(--tg-text-color)] hover:bg-[var(--tg-secondary-bg-color)]'
                }`}
              >
                {z}
              </button>
            ))}
            {filteredOther.length > 0 && (
              <>
                <div className="px-3 py-1.5 text-xs text-[var(--tg-hint-color)] border-t border-[var(--tg-secondary-bg-color)]">
                  Все часовые пояса
                </div>
                {filteredOther.slice(0, 100).map((z) => (
                  <button
                    key={z}
                    onClick={() => setTimezone(z)}
                    className={`w-full text-left px-3 py-2.5 text-sm transition-colors ${
                      timezone === z
                        ? 'bg-[var(--tg-button-color)] text-[var(--tg-button-text-color)]'
                        : 'text-[var(--tg-text-color)] hover:bg-[var(--tg-secondary-bg-color)]'
                    }`}
                  >
                    {z}
                  </button>
                ))}
              </>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
