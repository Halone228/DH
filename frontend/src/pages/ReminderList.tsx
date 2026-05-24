import { useEffect, useState, useCallback } from 'react';
import { useNavigate } from 'react-router-dom';
import { api } from '../api/client';
import { useTelegram } from '../hooks/useTelegram';
import { ReminderCard } from '../components/ReminderCard';
import { t } from '../i18n/ru';

interface Reminder {
  id: string;
  text: string;
  recurrence: string;
  time?: string;
  weekdays?: string[];
  day_of_month?: number;
  next_at?: string;
}

export function ReminderList() {
  const { tg } = useTelegram();
  const navigate = useNavigate();
  const [reminders, setReminders] = useState<Reminder[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchReminders = useCallback(async () => {
    try {
      setError(null);
      const data = await api.getReminders();
      setReminders(Array.isArray(data) ? data : []);
    } catch {
      setError(t.error.network);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchReminders();
  }, [fetchReminders]);

  useEffect(() => {
    if (!tg) return;
    tg.MainButton.setParams({
      text: t.reminder.create,
      color: 'var(--tg-button-color)',
      text_color: 'var(--tg-button-text-color)',
      is_visible: true,
    });
    const handler = () => navigate('/create');
    tg.MainButton.onClick(handler);
    return () => tg.MainButton.offClick(handler);
  }, [tg, navigate]);

  const handleCancel = async (id: string) => {
    try {
      await api.cancelReminder(id);
      setReminders((prev) => prev.filter((r) => r.id !== id));
      tg?.HapticFeedback?.notificationOccurred('success');
    } catch {
      setError(t.error.network);
    }
  };

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
        {t.nav.main}
      </h1>

      {error && (
        <div className="mb-4 p-3 rounded-lg bg-red-50 text-[var(--tg-destructive-color)] text-sm">
          {error}
        </div>
      )}

      {reminders.length === 0 ? (
        <div className="flex flex-col items-center justify-center py-16 text-center">
          <span className="text-5xl mb-4">📭</span>
          <p className="text-[var(--tg-text-color)] font-medium mb-1">
            {t.reminder.empty}
          </p>
          <p className="text-sm text-[var(--tg-hint-color)]">
            {t.reminder.emptyHint}
          </p>
        </div>
      ) : (
        <div>
          {reminders.map((r) => (
            <ReminderCard
              key={r.id}
              id={r.id}
              text={r.text}
              recurrence={r.recurrence}
              time={r.time}
              weekdays={r.weekdays}
              dayOfMonth={r.day_of_month}
              nextAt={r.next_at}
              onCancel={handleCancel}
              tg={tg}
            />
          ))}
        </div>
      )}
    </div>
  );
}
