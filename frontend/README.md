# DayHelper TMA Frontend

React 19 + TypeScript + Tailwind CSS SPA для Telegram Mini App.

## Разработка

```bash
npm install
npm run dev
```

## Сборка

```bash
npm run build   # → dist/
```

Собранные файлы раздаются сервером через Axum (`tower_http::ServeDir`) из `frontend/dist/`.
