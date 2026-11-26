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

export type Source = {
  id: string;
  project_id: string;
  kind: string;
  uri: string;
  branch?: string | null;
  include_patterns: string[];
  exclude_patterns: string[];
  created_at: string;
  updated_at: string;
  last_indexed_commit?: string | null;
  indexing_status: string;
};

export function createProject(token: string, input: { slug: string; name: string; description?: string | null }) {
  return request<Project>('/api/v1/projects', { method: 'POST', body: JSON.stringify(input) }, token);
}

export function updateProject(token: string, id: string, input: { name?: string; description?: string | null }) {
  return request<Project>(`/api/v1/projects/${id}`, { method: 'PATCH', body: JSON.stringify(input) }, token);
}

export function listSources(token: string, projectId: string) {
  return request<Source[]>(`/api/v1/projects/${projectId}/sources`, {}, token);
}

export function createSource(token: string, projectId: string, input: {
  kind: string;
  uri: string;
  branch?: string | null;
  include_patterns?: string[];
  exclude_patterns?: string[];
}) {
  return request<Source>(`/api/v1/projects/${projectId}/sources`, { method: 'POST', body: JSON.stringify(input) }, token);
}

export function deleteSource(token: string, sourceId: string) {
  return request<void>(`/api/v1/sources/${sourceId}`, { method: 'DELETE' }, token);
}

export type IndexJob = {
  id: string;
  project_id: string;
  source_id?: string | null;
  generation?: number | null;
  status: string;
  priority: number;
  attempts: number;
  max_attempts: number;
  error?: string | null;
  progress_percent: number;
  current_stage?: string | null;
  created_at: string;
  started_at?: string | null;
  finished_at?: string | null;
};

export function listIndexJobs(token: string, projectId: string) {
  return request<IndexJob[]>(`/api/v1/projects/${projectId}/index-jobs`, {}, token);
}

export function createIndexJob(token: string, projectId: string, sourceId?: string) {
  return request<IndexJob>(`/api/v1/projects/${projectId}/index-jobs`, {
    method: 'POST',
    body: JSON.stringify({ source_id: sourceId ?? null, priority: 0, max_attempts: 3, payload: { trigger: 'admin-panel' } }),
  }, token);
}

export function cancelIndexJob(token: string, jobId: string) {
  return request<void>(`/api/v1/index-jobs/${jobId}/cancel`, { method: 'POST' }, token);
}

export type ApiToken = {
  id: string;
  user_id?: string | null;
  name: string;
  scopes: string[];
  project_ids?: string[] | null;
  expires_at?: string | null;
  revoked_at?: string | null;
  last_used_at?: string | null;
  created_at: string;
};

export type TenantUser = {
  id: string;
  email: string;
  display_name: string;
  status: string;
  role?: string | null;
  created_at: string;
};

export function listApiTokens(token: string) {
  return request<ApiToken[]>('/api/v1/tokens', {}, token);
}

export function createApiToken(token: string, input: {
  name: string;
  scopes: string[];
  project_ids?: string[];
  expires_in_days?: number;
}) {
  return request<{ id: string; token: string; expires_at?: string | null }>('/api/v1/tokens', {
    method: 'POST',
    body: JSON.stringify(input),
  }, token);
}

export function revokeApiToken(token: string, id: string) {
  return request<void>(`/api/v1/tokens/${id}`, { method: 'DELETE' }, token);
}

export function listUsers(token: string) {
  return request<TenantUser[]>('/api/v1/users', {}, token);
}

export function createUser(token: string, input: {
  email: string;
  display_name: string;
  password: string;
  role: string;
}) {
  return request<TenantUser>('/api/v1/users', { method: 'POST', body: JSON.stringify(input) }, token);
}

export function updateUserRole(token: string, id: string, role: string) {
  return request<void>(`/api/v1/users/${id}/role`, { method: 'PATCH', body: JSON.stringify({ role }) }, token);
}
