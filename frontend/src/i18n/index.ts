import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';

import ru from './locales/ru.json';
import en from './locales/en.json';

function detectLanguage(): string {
  try {
    const webapp = (window as any).Telegram?.WebApp;
    const code = webapp?.initDataUnsafe?.user?.language_code;
    if (code) return code.startsWith('en') ? 'en' : 'ru';
  } catch {}
  return navigator.language?.startsWith('en') ? 'en' : 'ru';
}

i18n.use(initReactI18next).init({
  resources: {
    ru: { translation: ru },
    en: { translation: en },
  },
  lng: detectLanguage(),
  fallbackLng: 'ru',
  interpolation: { escapeValue: false },
});

export default i18n;
