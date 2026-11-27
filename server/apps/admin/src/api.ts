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


export type OidcPublicConfig = {
  enabled: boolean;
  button_label: string;
};

export function getOidcConfig() {
  return request<OidcPublicConfig>('/api/v1/auth/oidc/config');
}

export function login(email: string, password: string) {
  return request<Session>('/api/v1/auth/login', {
    method: 'POST',
    body: JSON.stringify({ email, password }),
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
  credential_ref?: string | null;
  auto_index_interval_seconds?: number | null;
  next_index_at?: string | null;
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
  credential_ref?: string | null;
  auto_index_interval_seconds?: number | null;
  include_patterns?: string[];
  exclude_patterns?: string[];
}) {
  return request<Source>(`/api/v1/projects/${projectId}/sources`, { method: 'POST', body: JSON.stringify(input) }, token);
}

export function deleteSource(token: string, sourceId: string) {
  return request<void>(`/api/v1/sources/${sourceId}`, { method: 'DELETE' }, token);
}



export type GitProvider = {
  id: string;
  name: string;
  provider: string;
  host: string;
  username: string;
  secret_hint: string;
  sources_count: number;
  last_used_at?: string | null;
  created_at: string;
  updated_at: string;
};

export function listGitProviders(token: string) {
  return request<GitProvider[]>('/api/v1/git-providers', {}, token);
}

export function createGitProvider(token: string, input: {
  name: string;
  provider: string;
  host: string;
  username?: string;
  token: string;
}) {
  return request<GitProvider>('/api/v1/git-providers', { method: 'POST', body: JSON.stringify(input) }, token);
}

export function deleteGitProvider(token: string, id: string) {
  return request<void>(`/api/v1/git-providers/${id}`, { method: 'DELETE' }, token);
}

export type WebhookConfig = {
  source_id: string;
  project_id: string;
  provider: string;
  enabled: boolean;
  endpoint_path: string;
  last_delivery_at?: string | null;
  last_status?: string | null;
};

export function configureSourceWebhook(token: string, sourceId: string, input: { provider: string; secret: string; enabled: boolean }) {
  return request<WebhookConfig>(`/api/v1/sources/${sourceId}/webhook`, { method: 'PUT', body: JSON.stringify(input) }, token);
}

export type IndexGeneration = {
  project_id: string;
  generation: number;
  status: string;
  embedding_model: string;
  embedding_dimensions: number;
  source_revisions: Record<string, string>;
  error?: string | null;
  is_active: boolean;
  created_at: string;
  started_at?: string | null;
  published_at?: string | null;
};

export function listIndexGenerations(token: string, projectId: string) {
  return request<IndexGeneration[]>(`/api/v1/projects/${projectId}/generations`, {}, token);
}

export function rollbackIndexGeneration(token: string, projectId: string, generation: number, reason?: string) {
  return request<{ project_id: string; active_generation: number; status: string }>(`/api/v1/projects/${projectId}/generations/${generation}/rollback`, {
    method: 'POST',
    body: JSON.stringify({ confirm_generation: generation, reason: reason || null }),
  }, token);
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

export type SearchHit = {
  chunk_id: string;
  document_id: string;
  path: string;
  heading_path: string[];
  content: string;
  score: number;
};

export function searchProject(token: string, input: {
  project_id: string;
  query: string;
  mode?: string;
  language?: string;
  document_type?: string;
  limit?: number;
}) {
  return request<SearchHit[]>('/api/v1/search', { method: 'POST', body: JSON.stringify(input) }, token);
}

export type AuditEvent = {
  id: string;
  project_id?: string | null;
  actor_user_id?: string | null;
  api_token_id?: string | null;
  action: string;
  resource_type: string;
  resource_id?: string | null;
  summary: Record<string, unknown>;
  created_at: string;
};

export function listAuditEvents(token: string) {
  return request<AuditEvent[]>('/api/v1/audit-events', {}, token);
}

export type UpdateStatus = {
  current_version: string;
  latest_version?: string | null;
  tag_name?: string | null;
  name?: string | null;
  has_update: boolean;
  severity?: string | null;
  release_url?: string | null;
  release_notes?: string | null;
  published_at?: string | null;
  checked_at: string;
  check_failed: boolean;
  channel: string;
};

export function getUpdateStatus(token: string, force = false) {
  const suffix = force ? '?force=true' : '';
  return request<UpdateStatus>(`/api/v1/system/update-status${suffix}`, {}, token);
}

export type ProjectGraphNode = {
  id: string;
  path: string;
  title?: string | null;
  document_type: string;
  language?: string | null;
  size_bytes: number;
  degree: number;
  community: number;
  is_god_node: boolean;
  tags: string[];
};

export type ProjectGraphEdge = {
  source: string;
  target: string;
  kind: string;
};

export type GodNode = {
  id: string;
  path: string;
  title?: string | null;
  degree: number;
  centrality: number;
  community: number;
  role: string;
};

export type ProjectGraph = {
  project_id: string;
  generation: number;
  nodes: ProjectGraphNode[];
  edges: ProjectGraphEdge[];
  analytics: {
    god_nodes: GodNode[];
    cycles: { nodes: string[]; length: number }[];
    cross_community_edges: number;
    isolated_count: number;
  };
  truncated: boolean;
  total_documents: number;
  total_edges: number;
  community_count: number;
  edge_source: string;
};

export function getProjectGraph(token: string, projectId: string, limit?: number) {
  const suffix = limit ? `?limit=${limit}` : '';
  return request<ProjectGraph>(`/api/v1/projects/${projectId}/graph${suffix}`, {}, token);
}
