import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router-dom';

interface WelcomeCardProps {
  onCreate: () => void;
}

export function WelcomeCard({ onCreate }: WelcomeCardProps) {
  const { t } = useTranslation();
  const navigate = useNavigate();

  return (
    <div className="flex flex-col items-center justify-center px-6 py-12 text-center"
         style={{ minHeight: 'calc(100vh - 120px)' }}>
      <div className="text-5xl mb-4">👋</div>
      <h2 className="text-xl font-semibold mb-2"
          style={{ color: 'var(--tg-text-color)' }}>
        {t('welcome.title')}
      </h2>
      <p className="mb-8"
         style={{ color: 'var(--tg-hint-color)' }}>
        {t('welcome.description')}
      </p>
      <div className="flex flex-col gap-3 w-full max-w-xs">
        <button
          onClick={onCreate}
          className="w-full py-3 rounded-xl font-medium text-base"
          style={{
            backgroundColor: 'var(--tg-button-color)',
            color: 'var(--tg-button-text-color)',
          }}
        >
          {t('welcome.createFirst')}
        </button>
        <div className="flex gap-3">
          <button
            onClick={() => navigate('/settings')}
            className="flex-1 py-2.5 rounded-xl text-sm"
            style={{
              backgroundColor: 'var(--tg-secondary-bg-color)',
              color: 'var(--tg-text-color)',
            }}
          >
            {t('welcome.hintSettings')}
          </button>
          <button
            onClick={() => navigate('/profile')}
            className="flex-1 py-2.5 rounded-xl text-sm"
            style={{
              backgroundColor: 'var(--tg-secondary-bg-color)',
              color: 'var(--tg-text-color)',
            }}
          >
            {t('welcome.hintProfile')}
          </button>
        </div>
      </div>
    </div>
  );
}
