# Progress

## Status
In Progress

## Tasks

### Phase 1 — Foundation (completed)
- [x] setMyCommands — register bot commands with Telegram on startup
- [x] Nudge bank — expanded from 7 to 50+ messages
- [x] CI/CD — GitHub Actions workflow for Rust + frontend
- [x] Docker — multi-stage Dockerfile + .dockerignore

### Phase 2 — Core Reliability (completed)
- [x] Graceful shutdown — broadcast channel + drain with 10s timeout
- [x] SQLite backup — WAL checkpoint + file copy every hour

### Phase 3 — Observability & UX (completed)
- [x] Structured logging — JSON format behind RUST_LOG_FORMAT=json, command spans
- [x] Bot error UX — friendly Russian error messages
- [x] Interactive /cancel — inline keyboard buttons

### Phase 4 — Production Hardening (in progress)
- [ ] Rate limiting — per-user on TMA and desktop API
- [x] Frontend polish — error boundary, offline detection, API retry

### Phase 5 — Tests (not started)
- [ ] Use case fakes + unit tests

## Files Changed

### This session
- `frontend/src/components/ErrorBoundary.tsx` — new: React error boundary with retry
- `frontend/src/hooks/useOnlineStatus.ts` — new: navigator.onLine hook
- `frontend/src/components/OfflineBanner.tsx` — new: fixed offline banner
- `frontend/src/api/client.ts` — retry on 5xx and network errors
- `frontend/src/App.tsx` — wrapped with ErrorBoundary + OfflineBanner
- `frontend/src/pages/ReminderList.tsx` — retry button on error
- `frontend/src/pages/Settings.tsx` — retry button on error
- `frontend/src/i18n/ru.ts` — added error.retry, error.offline, error.generic

## Notes
- Frontend build validated: `npm run build` succeeds
- All changes use Telegram CSS variables for theming
- API retry: 1 retry, 1s delay, only on 5xx and TypeError (network)
