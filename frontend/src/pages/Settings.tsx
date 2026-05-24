import { useEffect, useState, useCallback } from 'react';
import { useNavigate } from 'react-router-dom';
import { api } from '../api/client';
import { useTelegram } from '../hooks/useTelegram';
import { t } from '../i18n/ru';

interface NudgeSettings {
  enabled: boolean;
  daily_count: number;
  active_window_start: string;
  active_window_end: string;
}

const DEFAULT_SETTINGS: NudgeSettings = {
  enabled: true,
  daily_count: 5,
  active_window_start: '09:00',
  active_window_end: '21:00',
};

export function Settings() {
  const { tg } = useTelegram();
  const navigate = useNavigate();
  const [settings, setSettings] = useState<NudgeSettings>(DEFAULT_SETTINGS);
  const [original, setOriginal] = useState<string>('');
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchSettings = useCallback(async () => {
    try {
      const data = await api.getNudgeSettings();
      const s = { ...DEFAULT_SETTINGS, ...data };
      setSettings(s);
      setOriginal(JSON.stringify(s));
    } catch {
      setError(t.error.network);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchSettings();
  }, [fetchSettings]);

  const isDirty = JSON.stringify(settings) !== original;

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
        await api.saveNudgeSettings(settings);
        setOriginal(JSON.stringify(settings));
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
        {t.nudge.title}
      </h1>

      {error && (
        <div className="mb-4 p-3 rounded-lg bg-red-50 text-[var(--tg-destructive-color)] text-sm">
          {error}
        </div>
      )}

      <div className="space-y-5">
        {/* Enabled toggle */}
        <div className="flex items-center justify-between p-4 rounded-xl bg-[var(--tg-secondary-bg-color)]">
          <span className="text-sm font-medium text-[var(--tg-text-color)]">
            {t.nudge.enabled}
          </span>
          <button
            onClick={() => setSettings((s) => ({ ...s, enabled: !s.enabled }))}
            className={`relative w-12 h-7 rounded-full transition-colors ${
              settings.enabled ? 'bg-[var(--tg-button-color)]' : 'bg-[var(--tg-hint-color)]'
            }`}
          >
            <span
              className={`absolute top-0.5 w-6 h-6 bg-white rounded-full shadow transition-transform ${
                settings.enabled ? 'translate-x-5' : 'translate-x-0.5'
              }`}
            />
          </button>
        </div>

        {/* Daily count */}
        <div>
          <label className="block text-sm font-medium text-[var(--tg-text-color)] mb-1.5">
            {t.nudge.countLabel}
          </label>
          <div className="flex items-center gap-3">
            <button
              onClick={() =>
                setSettings((s) => ({
                  ...s,
                  daily_count: Math.max(1, s.daily_count - 1),
                }))
              }
              className="w-10 h-10 rounded-full bg-[var(--tg-secondary-bg-color)] text-[var(--tg-text-color)] text-lg font-bold flex items-center justify-center"
            >
              −
            </button>
            <span className="text-2xl font-bold text-[var(--tg-text-color)] w-8 text-center">
              {settings.daily_count}
            </span>
            <button
              onClick={() =>
                setSettings((s) => ({
                  ...s,
                  daily_count: Math.min(20, s.daily_count + 1),
                }))
              }
              className="w-10 h-10 rounded-full bg-[var(--tg-secondary-bg-color)] text-[var(--tg-text-color)] text-lg font-bold flex items-center justify-center"
            >
              +
            </button>
          </div>
        </div>

        {/* Active window */}
        <div>
          <label className="block text-sm font-medium text-[var(--tg-text-color)] mb-1.5">
            {t.nudge.windowLabel}
          </label>
          <div className="flex items-center gap-3">
            <div className="flex-1">
              <label className="block text-xs text-[var(--tg-hint-color)] mb-1">
                {t.nudge.from}
              </label>
              <input
                type="time"
                value={settings.active_window_start}
                onChange={(e) =>
                  setSettings((s) => ({
                    ...s,
                    active_window_start: e.target.value,
                  }))
                }
                className="w-full rounded-xl px-3 py-2.5 text-sm bg-[var(--tg-secondary-bg-color)] text-[var(--tg-text-color)] outline-none focus:ring-2 focus:ring-[var(--tg-button-color)]"
              />
            </div>
            <div className="flex-1">
              <label className="block text-xs text-[var(--tg-hint-color)] mb-1">
                {t.nudge.to}
              </label>
              <input
                type="time"
                value={settings.active_window_end}
                onChange={(e) =>
                  setSettings((s) => ({
                    ...s,
                    active_window_end: e.target.value,
                  }))
                }
                className="w-full rounded-xl px-3 py-2.5 text-sm bg-[var(--tg-secondary-bg-color)] text-[var(--tg-text-color)] outline-none focus:ring-2 focus:ring-[var(--tg-button-color)]"
              />
            </div>
          </div>
        </div>

        {/* Link to profile */}
        <button
          onClick={() => navigate('/profile')}
          className="w-full py-3 text-sm text-[var(--tg-button-color)] text-center"
        >
          {t.nav.profile} →
        </button>
      </div>
    </div>
  );
}
