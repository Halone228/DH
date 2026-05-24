import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { api } from '../api/client';
import { useTelegram } from '../hooks/useTelegram';
import { RecurrenceForm } from '../components/RecurrenceForm';
import { WeekdayChips } from '../components/WeekdayChips';
import { t } from '../i18n/ru';

type RecurrenceKind = 'once' | 'daily' | 'weekly' | 'monthly';

export function CreateReminder() {
  const { tg } = useTelegram();
  const navigate = useNavigate();

  const [text, setText] = useState('');
  const [recurrence, setRecurrence] = useState<RecurrenceKind>('once');
  const [time, setTime] = useState('09:00');
  const [date, setDate] = useState('');
  const [weekdays, setWeekdays] = useState<string[]>([]);
  const [dayOfMonth, setDayOfMonth] = useState(1);
  const [timezone, setTimezone] = useState('Europe/Moscow');
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => {
    api.getMe().then((me) => {
      if (me?.timezone) setTimezone(me.timezone);
    }).catch(() => {});
  }, []);

  useEffect(() => {
    if (!tg) return;
    tg.MainButton.setParams({
      text: t.reminder.createShort,
      color: 'var(--tg-button-color)',
      text_color: 'var(--tg-button-text-color)',
      is_visible: true,
    });
    const handler = () => handleSubmit();
    tg.MainButton.onClick(handler);
    return () => tg.MainButton.offClick(handler);
  });

  const isValid = () => {
    if (!text.trim()) return false;
    if (recurrence === 'once' && !date) return false;
    if (recurrence === 'weekly' && weekdays.length === 0) return false;
    if (recurrence === 'monthly' && (dayOfMonth < 1 || dayOfMonth > 31)) return false;
    return true;
  };

  useEffect(() => {
    if (tg) {
      if (isValid()) {
        tg.MainButton.enable();
      } else {
        tg.MainButton.disable();
      }
    }
  }, [text, recurrence, date, weekdays, dayOfMonth, tg]);

  const handleSubmit = async () => {
    if (!isValid() || submitting) return;
    setSubmitting(true);
    try {
      const payload: Record<string, unknown> = {
        text: text.trim(),
        recurrence,
        time,
        timezone,
      };
      if (recurrence === 'once') {
        payload.date = date;
      } else if (recurrence === 'weekly') {
        payload.weekdays = weekdays;
      } else if (recurrence === 'monthly') {
        payload.day_of_month = dayOfMonth;
      }

      await api.createReminder(payload);
      tg?.HapticFeedback?.notificationOccurred('success');
      navigate('/');
    } catch {
      tg?.showAlert?.(t.error.network);
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div className="px-4 pt-4">
      <h1 className="text-xl font-bold mb-4 text-[var(--tg-text-color)]">
        {t.reminder.create}
      </h1>

      <div className="space-y-5">
        {/* Text input */}
        <div>
          <label className="block text-sm font-medium text-[var(--tg-text-color)] mb-1.5">
            {t.reminder.textLabel}
          </label>
          <textarea
            value={text}
            onChange={(e) => setText(e.target.value)}
            placeholder={t.reminder.textPlaceholder}
            rows={3}
            className="w-full rounded-xl px-3 py-2.5 text-sm bg-[var(--tg-secondary-bg-color)] text-[var(--tg-text-color)] placeholder-[var(--tg-hint-color)] resize-none outline-none focus:ring-2 focus:ring-[var(--tg-button-color)]"
          />
        </div>

        {/* Recurrence type */}
        <div>
          <label className="block text-sm font-medium text-[var(--tg-text-color)] mb-1.5">
            {t.reminder.typeLabel}
          </label>
          <RecurrenceForm value={recurrence} onChange={setRecurrence} />
        </div>

        {/* Conditional fields */}
        {recurrence === 'once' && (
          <div>
            <label className="block text-sm font-medium text-[var(--tg-text-color)] mb-1.5">
              {t.reminder.dateLabel}
            </label>
            <input
              type="date"
              value={date}
              onChange={(e) => setDate(e.target.value)}
              className="w-full rounded-xl px-3 py-2.5 text-sm bg-[var(--tg-secondary-bg-color)] text-[var(--tg-text-color)] outline-none focus:ring-2 focus:ring-[var(--tg-button-color)]"
            />
          </div>
        )}

        {(recurrence === 'daily' || recurrence === 'weekly' || recurrence === 'monthly' || recurrence === 'once') && (
          <div>
            <label className="block text-sm font-medium text-[var(--tg-text-color)] mb-1.5">
              {t.reminder.timeLabel}
            </label>
            <input
              type="time"
              value={time}
              onChange={(e) => setTime(e.target.value)}
              className="w-full rounded-xl px-3 py-2.5 text-sm bg-[var(--tg-secondary-bg-color)] text-[var(--tg-text-color)] outline-none focus:ring-2 focus:ring-[var(--tg-button-color)]"
            />
          </div>
        )}

        {recurrence === 'weekly' && (
          <div>
            <label className="block text-sm font-medium text-[var(--tg-text-color)] mb-1.5">
              {t.reminder.weekdaysLabel}
            </label>
            <WeekdayChips selected={weekdays} onChange={setWeekdays} />
          </div>
        )}

        {recurrence === 'monthly' && (
          <div>
            <label className="block text-sm font-medium text-[var(--tg-text-color)] mb-1.5">
              {t.reminder.dayOfMonthLabel}
            </label>
            <input
              type="number"
              min={1}
              max={31}
              value={dayOfMonth}
              onChange={(e) => setDayOfMonth(Number(e.target.value))}
              className="w-full rounded-xl px-3 py-2.5 text-sm bg-[var(--tg-secondary-bg-color)] text-[var(--tg-text-color)] outline-none focus:ring-2 focus:ring-[var(--tg-button-color)]"
            />
          </div>
        )}
      </div>
    </div>
  );
}
