import { t } from '../i18n/ru';

type RecurrenceKind = 'once' | 'daily' | 'weekly' | 'monthly';

interface RecurrenceFormProps {
  value: RecurrenceKind;
  onChange: (v: RecurrenceKind) => void;
}

const OPTIONS: { key: RecurrenceKind; label: string }[] = [
  { key: 'once', label: t.reminder.typeOnce },
  { key: 'daily', label: t.reminder.typeDaily },
  { key: 'weekly', label: t.reminder.typeWeekly },
  { key: 'monthly', label: t.reminder.typeMonthly },
];

export function RecurrenceForm({ value, onChange }: RecurrenceFormProps) {
  return (
    <div className="flex gap-1 bg-[var(--tg-secondary-bg-color)] rounded-lg p-1">
      {OPTIONS.map((opt) => (
        <button
          key={opt.key}
          type="button"
          onClick={() => onChange(opt.key)}
          className={`flex-1 py-2 px-1 text-xs font-medium rounded-md transition-colors ${
            value === opt.key
              ? 'bg-[var(--tg-button-color)] text-[var(--tg-button-text-color)]'
              : 'text-[var(--tg-text-color)]'
          }`}
        >
          {opt.label}
        </button>
      ))}
    </div>
  );
}
