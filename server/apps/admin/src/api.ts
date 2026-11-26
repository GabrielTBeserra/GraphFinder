export type Session = {
  token: string;
  expires_at: string;
  user_id: string;
  tenant_id: string;
};

const API_BASE = import.meta.env.VITE_API_URL ?? '';
const SESSION_KEY = 'graphfinder.server.session';

export function loadSession(): Session | null {
  const raw = localStorage.getItem(SESSION_KEY);
  if (!raw) return null;
  try {
    const session = JSON.parse(raw) as Session;
    return Date.parse(session.expires_at) > Date.now() ? session : null;
  } catch {
    return null;
  }
}

export function saveSession(session: Session | null) {
  if (session) localStorage.setItem(SESSION_KEY, JSON.stringify(session));
  else localStorage.removeItem(SESSION_KEY);
}

async function request<T>(path: string, init: RequestInit = {}, token?: string): Promise<T> {
  const response = await fetch(`${API_BASE}${path}`, {
    ...init,
    headers: {
      'content-type': 'application/json',
      ...(token ? { authorization: `Bearer ${token}` } : {}),
      ...(init.headers ?? {}),
    },
  });
  if (!response.ok) {
    const text = await response.text();
    throw new Error(text || `request failed with ${response.status}`);
  }
  if (response.status === 204) return undefined as T;
  return response.json() as Promise<T>;
}

export function login(tenantSlug: string, email: string, password: string) {
  return request<Session>('/api/v1/auth/login', {
    method: 'POST',
    body: JSON.stringify({ tenant_slug: tenantSlug, email, password }),
  });
}

export function logout(token: string) {
  return request<void>('/api/v1/auth/logout', { method: 'POST' }, token);
}

export type Project = {
  id: string;
  slug: string;
  name: string;
  description?: string | null;
  active_generation: number;
  created_at: string;
  updated_at: string;
};

export function listProjects(token: string) {
  return request<Project[]>('/api/v1/projects', {}, token);
}
