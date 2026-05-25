import { useTranslation } from 'react-i18next';

const WEEKDAY_KEYS = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'] as const;

interface WeekdayChipsProps {
  selected: string[];
  onChange: (selected: string[]) => void;
}

export function WeekdayChips({ selected, onChange }: WeekdayChipsProps) {
  const { t } = useTranslation();

  const toggle = (day: string) => {
    if (selected.includes(day)) {
      onChange(selected.filter((d) => d !== day));
    } else {
      onChange([...selected, day]);
    }
  };

  return (
    <div className="flex flex-wrap gap-2">
      {WEEKDAY_KEYS.map((day) => {
        const isActive = selected.includes(day);
        return (
          <button
            key={day}
            type="button"
            onClick={() => toggle(day)}
            className={`w-10 h-10 rounded-full text-sm font-medium transition-colors ${
              isActive
                ? 'bg-[var(--tg-button-color)] text-[var(--tg-button-text-color)]'
                : 'bg-[var(--tg-secondary-bg-color)] text-[var(--tg-text-color)]'
            }`}
          >
            {t(`weekdays.${day}`)}
          </button>
        );
      })}
    </div>
  );
}
