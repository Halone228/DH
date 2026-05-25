# i18n Bot Implementation — Progress

## Status: ✅ COMPLETE

## Changes Made

### 1. `Locale` enum in domain (`crates/domain/src/locale.rs`) — NEW
- `Locale::Ru` (default), `Locale::En`
- `code()` → `"ru"` / `"en"`
- `from_code()` parses string codes (case-insensitive, defaults to `Ru`)

### 2. `User.locale` field type changed (`crates/domain/src/user.rs`)
- **Before:** `pub locale: String` (always `"ru"`)
- **After:** `pub locale: Locale` (defaults to `Locale::Ru`)

### 3. No new DB migration needed
- `users.locale` column already existed in `0001_initial.sql` as `TEXT NOT NULL DEFAULT 'ru'`
- Adapter converts via `Locale::code()` / `Locale::from_code()`

### 4. Bot message catalog (`crates/application/src/l10n.rs`) — NEW
- `BotMessages` struct with ~35 `&'static str` fields covering every hardcoded Russian string in the bot
- `BotMessages::for_locale(Locale) -> &'static BotMessages` dispatches to static `RU` / `EN`
- Both Russian and English translations provided for all keys

### 5. Nudge messages locale-aware (`crates/application/src/messages.rs`)
- Added `MESSAGES_EN` (65 English nudge strings matching `MESSAGES_RU`)
- `nudge_text()` now takes `Locale` parameter and selects the correct bank

### 6. Bot handlers rewritten (`crates/bot/src/lib.rs`)
- All hardcoded Russian strings replaced with `BotMessages::for_locale(user.locale)` lookups
- String interpolation uses `.replace("{}", val)` (since `format!` requires string literals)
- Parse errors now use English strings (these are developer-facing hints)
- Command descriptions changed to English (visible to all users in autocomplete)

### 7. `ScheduleDailyNudges.execute()` signature updated
- Now takes `locale: Locale` as 4th parameter
- Scheduler's `plan_nudges_round()` passes `user.locale` through

### 8. `crates/adapter-sqlite/src/user_repo.rs` updated
- Writes `user.locale.code()` to DB
- Reads back with `Locale::from_code(&row.locale)`

### 9. `crates/domain/src/lib.rs` updated
- Added `pub mod locale;` and `pub use locale::Locale;`

### 10. `crates/application/src/lib.rs` updated
- Added `pub mod l10n;`

## Validation
- ✅ `cargo build --workspace` — passes
- ✅ `cargo clippy --workspace --all-targets -- -D warnings` — clean
- ✅ `cargo test --workspace` — all 50 tests pass

## Open Items / Future Work
- **Locale detection from Telegram `language_code`**: Currently defaults to `Ru`. Bot can detect from `msg.from().language_code` and pass to `EnsureUser` or a new `UpdateLocale` use case.
- **User-facing locale switching**: No `/language` command yet — user has no way to change locale after creation.
- **`reply_error` locale-awareness**: Currently uses a default English message. Could be made locale-aware by threading user through error paths.
- **Bot command descriptions**: Changed to English; could be locale-aware via `set_my_commands` per-user.
