import { useTranslation } from 'react-i18next';

type RecurrenceKind = 'once' | 'daily' | 'weekly' | 'monthly';

interface RecurrenceFormProps {
  value: RecurrenceKind;
  onChange: (v: RecurrenceKind) => void;
}

const OPTIONS: { key: RecurrenceKind; labelKey: string }[] = [
  { key: 'once', labelKey: 'reminder.typeOnce' },
  { key: 'daily', labelKey: 'reminder.typeDaily' },
  { key: 'weekly', labelKey: 'reminder.typeWeekly' },
  { key: 'monthly', labelKey: 'reminder.typeMonthly' },
];

export function RecurrenceForm({ value, onChange }: RecurrenceFormProps) {
  const { t } = useTranslation();

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
          {t(opt.labelKey)}
        </button>
      ))}
    </div>
  );
}
