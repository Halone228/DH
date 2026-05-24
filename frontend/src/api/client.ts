const API_BASE = '';

class ApiClient {
  private initData: string;

  constructor() {
    // @ts-ignore - Telegram injects this
    this.initData = window.Telegram?.WebApp?.initData ?? '';
  }

  private async request(
    method: string,
    path: string,
    body?: unknown,
    retries = 1,
  ): Promise<any> {
    const headers: Record<string, string> = {
      Authorization: `tma ${this.initData}`,
      'Content-Type': 'application/json',
    };
    const opts: RequestInit = { method, headers };
    if (body) opts.body = JSON.stringify(body);

    try {
      const res = await fetch(`${API_BASE}${path}`, opts);
      if (!res.ok) {
        if (res.status >= 500 && retries > 0) {
          await new Promise((r) => setTimeout(r, 1000));
          return this.request(method, path, body, retries - 1);
        }
        const text = await res.text();
        throw new Error(text || res.statusText);
      }
      if (res.status === 204) return null;
      return res.json();
    } catch (err) {
      if (err instanceof TypeError && retries > 0) {
        await new Promise((r) => setTimeout(r, 1000));
        return this.request(method, path, body, retries - 1);
      }
      throw err;
    }
  }

  getMe() {
    return this.request('GET', '/api/me');
  }
  getReminders() {
    return this.request('GET', '/api/reminders');
  }
  createReminder(data: unknown) {
    return this.request('POST', '/api/reminders', data);
  }
  cancelReminder(id: string) {
    return this.request('DELETE', `/api/reminders/${id}`);
  }
  getNudgeSettings() {
    return this.request('GET', '/api/nudge-settings');
  }
  saveNudgeSettings(s: unknown) {
    return this.request('PUT', '/api/nudge-settings', s);
  }
  updateTimezone(tz: string) {
    return this.request('PATCH', '/api/me', { timezone: tz });
  }
}

export const api = new ApiClient();
