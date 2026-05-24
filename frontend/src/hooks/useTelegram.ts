import { useEffect, useState } from 'react';

interface TelegramUser {
  id: number;
  first_name: string;
  last_name?: string;
  username?: string;
}

export function useTelegram() {
  const [tg, setTg] = useState<any>(null);
  const [user, setUser] = useState<TelegramUser | null>(null);

  useEffect(() => {
    const w = window as any;
    const webapp = w.Telegram?.WebApp;
    if (webapp) {
      webapp.ready();
      webapp.expand();
      webapp.setHeaderColor('bg_color');
      webapp.setBackgroundColor('bg_color');
      setTg(webapp);
      setUser(webapp.initDataUnsafe?.user ?? null);
    }
  }, []);

  return { tg, user };
}
