import { useTranslation } from 'react-i18next';

interface ReminderCardProps {
  id: string;
  text: string;
  recurrence: string;
  time?: string;
  weekdays?: string[];
  dayOfMonth?: number;
  nextAt?: string;
  onCancel: (id: string) => void;
  tg: any;
}

export function ReminderCard({
  id,
  text,
  recurrence,
  time,
  weekdays,
  dayOfMonth,
  nextAt,
  onCancel,
  tg,
}: ReminderCardProps) {
  const { t } = useTranslation();

  const recurrenceLabel =
    t(`recurrence.${recurrence}`, recurrence);

  const handleCancel = () => {
    if (!tg) return;
    tg.showConfirm(t('reminder.deleteConfirm'), (ok: boolean) => {
      if (ok) onCancel(id);
    });
  };

  return (
    <div className="rounded-xl p-4 mb-3 bg-[var(--tg-secondary-bg-color)]">
      <div className="flex justify-between items-start">
        <div className="flex-1 min-w-0 pr-3">
          <p className="text-sm font-medium text-[var(--tg-text-color)] break-words">
            {text}
          </p>
          <div className="flex flex-wrap gap-1.5 mt-2">
            <span className="inline-block text-xs px-2 py-0.5 rounded-full bg-[var(--tg-button-color)] text-[var(--tg-button-text-color)]">
              {recurrenceLabel}
            </span>
            {time && (
              <span className="inline-block text-xs px-2 py-0.5 rounded-full bg-[var(--tg-secondary-bg-color)] text-[var(--tg-text-color)] border border-[var(--tg-hint-color)]">
                {time}
              </span>
            )}
            {weekdays && weekdays.length > 0 && (
              <span className="inline-block text-xs px-2 py-0.5 rounded-full bg-[var(--tg-secondary-bg-color)] text-[var(--tg-text-color)] border border-[var(--tg-hint-color)]">
                {weekdays.map((d) => t(`weekdays.${d}`, d)).join(', ')}
              </span>
            )}
            {dayOfMonth != null && (
              <span className="inline-block text-xs px-2 py-0.5 rounded-full bg-[var(--tg-secondary-bg-color)] text-[var(--tg-text-color)] border border-[var(--tg-hint-color)]">
                {dayOfMonth} {t('reminder.dayOfMonthSuffix')}
              </span>
            )}
          </div>
          {nextAt && (
            <p className="text-xs text-[var(--tg-hint-color)] mt-1.5">
              {t('reminder.nextAt')} {nextAt}
            </p>
          )}
        </div>
        <button
          onClick={handleCancel}
          className="shrink-0 w-8 h-8 flex items-center justify-center rounded-full hover:bg-red-50 active:bg-red-100 text-[var(--tg-destructive-color)] transition-colors"
          aria-label={t('reminder.cancelled')}
        >
          ✕
        </button>
      </div>
    </div>
  );
}
