# DayHelper

> Telegram-бот для напоминаний и анти-прокрастинации с Linux desktop-клиентом

## Возможности

- **4 типа напоминаний** — разовые, ежедневные, еженедельные (любые дни недели), ежемесячные
- **Анти-прокрастинация** — случайные нуджи в течение дня в заданном окне (по умолчанию 09:00–21:00)
- **Desktop-клиент** — отслеживание активности на Wayland, локальные уведомления через D-Bus
- **Telegram Mini App** — веб-интерфейс на React для управления напоминаниями
- **Timezone-aware** — каждый расчёт учитывает часовой пояс пользователя и переход на летнее/зимнее время (DST)

## Быстрый старт

### Требования

- Rust 1.78+ (устанавливается через `nix develop`)
- SQLite 3
- Node.js 20+ (для сборки frontend)
- Nix (опционально, но рекомендуется для dev shell)

### Установка и запуск

```bash
git clone git@github.com:Halone228/DH.git dayhelper
cd dayhelper

# С Nix (рекомендуется):
nix develop                    # rust toolchain + sqlx-cli + sqlite

# Без Nix:
# Убедитесь, что rustc, cargo, sqlx-cli и sqlite установлены

cp .env.example .env           # заполнить TELOXIDE_TOKEN
cargo run -p dayhelper-app     # сервер: бот + scheduler + TMA HTTP
```

### Сборка frontend

```bash
cd frontend
npm install
npm run build                  # → frontend/dist/
```

Axum автоматически раздаёт `frontend/dist/` как статические файлы.

## Команды бота

| Команда | Описание | Пример |
|---------|----------|--------|
| `/start` | Приветствие | |
| `/help` | Список команд | |
| `/once` | Разовое напоминание | `/once 2026-05-25T09:00 купить молоко` |
| `/daily` | Ежедневное | `/daily 09:00 зарядка` |
| `/weekly` | Еженедельное | `/weekly Mon,Wed,Fri 09:00 спорт` |
| `/monthly` | Ежемесячное | `/monthly 15 10:00 отчёт` |
| `/list` | Список напоминаний | |
| `/cancel` | Отменить | `/cancel <id>` |
| `/pair` | Код для desktop-клиента | |
| `/timezone` | Сменить часовой пояс | `/timezone Europe/Moscow` |
| `/nudge` | Вкл/выкл нуджи | `/nudge on` или `/nudge off` |
| `/nudge_window` | Активное окно | `/nudge_window 09:00 21:00` |
| `/settings` | Текущие настройки | |

## Desktop-клиент

CLI-утилита `dayhelper-cli` для Linux: отслеживает активное окно на Wayland, синхронизирует данные с сервером, показывает локальные уведомления.

### Подключение

```bash
# 1. Получи код в боте: отправь /pair
# 2. Введи код на устройстве:
cargo run -p dayhelper-desktop-app -- login <код> \
    --server-url https://your-server.example
```

### Запуск

```bash
cargo run -p dayhelper-desktop-app -- daemon
```

URL сервера можно задать через `--server-url` или `DAYHELPER_SERVER_URL`.

### Autostart (systemd)

```bash
cp contrib/dayhelper-daemon.service ~/.config/systemd/user/
systemctl --user enable --now dayhelper-daemon
```

### Поддерживаемые композиторы

Wayland (niri, sway, hyprland, river, wayfire). GNOME и KDE пока не поддерживаются — для их поддержки нужен отдельный адаптер.

## Архитектура

Hexagonal / Ports & Adapters. Зависимости направлены внутрь к `domain`.

```
        ┌──────────────── app (binary, composition root) ────────────────┐
        │                                                                │
        │   bot          tma           scheduler                         │
        │     \           |              /                               │
        │      \          |             /                                │
        │       └──────► application ◄─┘                                 │
        │                    │                                           │
        │                    ▼                                           │
        │                  ports ────────► domain                        │
        │                    ▲                                           │
        │      ┌─────────────┼──────────────┐                            │
        │      │             │              │                            │
        │   adapter-      adapter-       adapter-                        │
        │   sqlite        telegram       system                          │
        └────────────────────────────────────────────────────────────────┘
```

Desktop side:

```
       desktop-app (binary `dayhelper-cli`)
                  │
   ┌──────────────┼──────────────┐
   │              │              │
   v              v              v
 daemon ─► desktop-application ──┐
                  │              │
                  v              │
            desktop-ports ◄──────┘
                  ▲
   ┌──────┬───────┼───────┬───────┐
   │      │       │       │       │
adapter- adapter- adapter- adapter- adapter-
wayland  dbus     http    sqlite   (gnome…)
```

### Краты

| Крат | Описание |
|------|----------|
| `domain` | Чистые типы: User, Reminder, Recurrence, NudgeSettings, Weekday |
| `ports` | async-trait интерфейсы: UserRepo, ReminderRepo, JobQueue и т.д. |
| `application` | Use cases: CreateReminder, FireDueJobs, EnsureUser и т.д. |
| `adapter-sqlite` | SQLite-реализации портов + миграции |
| `adapter-telegram` | Обёртка над teloxide для отправки сообщений |
| `adapter-system` | Системные адаптеры (clock, pair codes) |
| `bot` | Telegram bot: teloxide dispatcher + обработчики команд |
| `tma` | TMA HTTP API: axum router + initData auth |
| `scheduler` | Background job scheduler (persistent SQL queue) |
| `app` | Composition root — собирает Container, запускает все слои |
| `protocol` | Wire types для desktop ↔ server |
| `server-desktop-api` | REST API для desktop-клиента |
| `desktop-domain` | Desktop domain types |
| `desktop-ports` | Desktop async-trait interfaces |
| `desktop-application` | Desktop use cases |
| `desktop-adapter-wayland` | Wayland window tracker (zwlr_foreign_toplevel) |
| `desktop-adapter-dbus` | D-Bus уведомления (libnotify) |
| `desktop-adapter-http` | HTTP sync с сервером |
| `desktop-adapter-sqlite` | Локальный SQLite для уведомлений и активности |
| `desktop-app` | CLI binary: login, daemon, status, logout |
| `frontend` | React 19 + Tailwind CSS TMA SPA |

## Переменные окружения

| Переменная | Обязательная | По умолчанию | Описание |
|------------|--------------|--------------|----------|
| `TELOXIDE_TOKEN` | Да | — | Токен бота от @BotFather |
| `DATABASE_URL` | Да | `sqlite://dayhelper.db` | Путь к SQLite |
| `TMA_PUBLIC_URL` | Нет | — | Публичный URL Mini App |
| `TMA_BIND_ADDR` | Нет | `0.0.0.0:8080` | Адрес HTTP-сервера |
| `RUST_LOG` | Нет | `info,sqlx=warn` | Уровень логирования |

## Разработка

```bash
nix develop                                  # rust + sqlx-cli + sqlite
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
```

## Tech Stack

**Server:** Rust, teloxide, axum, sqlx, SQLite, tokio

**Desktop:** Rust, wayland-client, zbus (D-Bus), reqwest

**Frontend:** React 19, TypeScript, Vite, Tailwind CSS, @tma.js/sdk-react

## Лицензия

MIT OR Apache-2.0
