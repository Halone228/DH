const API_BASE = '';

class ApiClient {
  private initData: string;

  constructor() {
    // @ts-ignore - Telegram injects this
    this.initData = window.Telegram?.WebApp?.initData ?? '';
  }

  private async request(method: string, path: string, body?: unknown) {
    const headers: Record<string, string> = {
      'Authorization': `tma ${this.initData}`,
      'Content-Type': 'application/json',
    };
    const opts: RequestInit = { method, headers };
    if (body) opts.body = JSON.stringify(body);

    const res = await fetch(`${API_BASE}${path}`, opts);
    if (!res.ok) {
      const text = await res.text();
      throw new Error(text || res.statusText);
    }
    if (res.status === 204) return null;
    return res.json();
  }

  getMe() { return this.request('GET', '/api/me'); }
  getReminders() { return this.request('GET', '/api/reminders'); }
  createReminder(data: unknown) { return this.request('POST', '/api/reminders', data); }
  cancelReminder(id: string) { return this.request('DELETE', `/api/reminders/${id}`); }
  getNudgeSettings() { return this.request('GET', '/api/nudge-settings'); }
  saveNudgeSettings(s: unknown) { return this.request('PUT', '/api/nudge-settings', s); }
  updateTimezone(tz: string) { return this.request('PATCH', '/api/me', { timezone: tz }); }
}

export const api = new ApiClient();
