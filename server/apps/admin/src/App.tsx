import { FormEvent, useEffect, useMemo, useState } from 'react';
import {
  ApiToken,
  cancelIndexJob,
  createApiToken,
  createIndexJob,
  createProject,
  createUser,
  createSource,
  deleteSource,
  IndexJob,
  listApiTokens,
  listIndexJobs,
  listProjects,
  listUsers,
  listSources,
  loadSession,
  login,
  logout,
  Project,
  revokeApiToken,
  saveSession,
  searchProject,
  SearchHit,
  Session,
  Source,
  TenantUser,
  updateProject,
  updateUserRole,
} from './api';

function LoginScreen({ onLogin }: { onLogin: (session: Session) => void }) {
  const [tenant, setTenant] = useState('default');
  const [email, setEmail] = useState('admin@example.com');
  const [password, setPassword] = useState('');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);

  async function submit(event: FormEvent) {
    event.preventDefault();
    setLoading(true);
    setError('');
    try {
      const session = await login(tenant, email, password);
      saveSession(session);
      onLogin(session);
    } catch {
      setError('Credenciais inválidas ou servidor indisponível.');
    } finally {
      setLoading(false);
    }
  }

  return (
    <main className="login-shell">
      <section className="login-panel">
        <div className="brand-mark">GF</div>
        <p className="eyebrow">GraphFinder Server</p>
        <h1>Control room for distributed knowledge indexing.</h1>
        <form onSubmit={submit} className="login-form">
          <label>
            Tenant
            <input value={tenant} onChange={(event) => setTenant(event.target.value)} required />
          </label>
          <label>
            Email
            <input type="email" value={email} onChange={(event) => setEmail(event.target.value)} required />
          </label>
          <label>
            Password
            <input type="password" value={password} onChange={(event) => setPassword(event.target.value)} required />
          </label>
          {error && <p className="form-error">{error}</p>}
          <button disabled={loading}>{loading ? 'Authenticating…' : 'Enter console'}</button>
        </form>
      </section>
    </main>
  );
}


function ProjectsManager({ session, projects, setProjects, status }: {
  session: Session;
  projects: Project[];
  setProjects: (projects: Project[]) => void;
  status: string;
}) {
  const [selectedId, setSelectedId] = useState('');
  const [sources, setSources] = useState<Source[]>([]);
  const [projectForm, setProjectForm] = useState({ slug: '', name: '', description: '' });
  const [sourceForm, setSourceForm] = useState({ uri: '', branch: 'main', include: '', exclude: '' });
  const [editName, setEditName] = useState('');
  const [message, setMessage] = useState('');
  const selected = projects.find((project) => project.id === selectedId) ?? projects[0];

  useEffect(() => {
    if (!selected) {
      setSources([]);
      return;
    }
    setSelectedId(selected.id);
    setEditName(selected.name);
    listSources(session.token, selected.id).then(setSources).catch(() => setSources([]));
  }, [session.token, selected?.id]);

  async function refreshProjects() {
    const items = await listProjects(session.token);
    setProjects(items);
  }

  async function submitProject(event: FormEvent) {
    event.preventDefault();
    setMessage('');
    try {
      const project = await createProject(session.token, {
        slug: projectForm.slug,
        name: projectForm.name,
        description: projectForm.description || null,
      });
      setProjectForm({ slug: '', name: '', description: '' });
      await refreshProjects();
      setSelectedId(project.id);
      setMessage('project created');
    } catch {
      setMessage('project creation failed');
    }
  }

  async function saveProject(event: FormEvent) {
    event.preventDefault();
    if (!selected) return;
    try {
      await updateProject(session.token, selected.id, { name: editName });
      await refreshProjects();
      setMessage('project updated');
    } catch {
      setMessage('project update failed');
    }
  }

  async function submitSource(event: FormEvent) {
    event.preventDefault();
    if (!selected) return;
    const split = (value: string) => value.split('\n').map((item) => item.trim()).filter(Boolean);
    try {
      await createSource(session.token, selected.id, {
        kind: 'git',
        uri: sourceForm.uri,
        branch: sourceForm.branch || null,
        include_patterns: split(sourceForm.include),
        exclude_patterns: split(sourceForm.exclude),
      });
      setSourceForm({ uri: '', branch: 'main', include: '', exclude: '' });
      setSources(await listSources(session.token, selected.id));
      setMessage('source created');
    } catch {
      setMessage('source creation failed');
    }
  }

  async function removeSource(id: string) {
    if (!selected) return;
    await deleteSource(session.token, id).catch(() => undefined);
    setSources(await listSources(session.token, selected.id).catch(() => []));
  }

  return (
    <div className="management-grid">
      <section>
        <div className="panel-heading"><h2>Projects</h2><span>{status}</span></div>
        <div className="project-list">
          {projects.map((project) => (
            <button key={project.id} className={`project-row selectable ${selected?.id === project.id ? 'selected' : ''}`} onClick={() => setSelectedId(project.id)}>
              <div><strong>{project.name}</strong><span>{project.slug}</span></div>
              <code>gen {project.active_generation}</code>
            </button>
          ))}
          {projects.length === 0 && <p className="empty-state">No projects visible for this principal.</p>}
        </div>
        <form className="inline-form" onSubmit={submitProject}>
          <h3>New project</h3>
          <input placeholder="slug" value={projectForm.slug} onChange={(e) => setProjectForm({ ...projectForm, slug: e.target.value })} required />
          <input placeholder="name" value={projectForm.name} onChange={(e) => setProjectForm({ ...projectForm, name: e.target.value })} required />
          <input placeholder="description" value={projectForm.description} onChange={(e) => setProjectForm({ ...projectForm, description: e.target.value })} />
          <button>Create project</button>
        </form>
      </section>
      <section>
        {selected ? (
          <>
            <div className="panel-heading"><h2>{selected.slug}</h2><span>generation {selected.active_generation}</span></div>
            <form className="inline-form horizontal" onSubmit={saveProject}>
              <input value={editName} onChange={(e) => setEditName(e.target.value)} />
              <button>Save name</button>
            </form>
            <div className="source-block">
              <h3>Git sources</h3>
              {sources.map((source) => (
                <article className="source-row" key={source.id}>
                  <div>
                    <strong>{source.uri}</strong>
                    <span>branch {source.branch ?? 'default'} · {source.indexing_status} · commit {source.last_indexed_commit ? source.last_indexed_commit.slice(0, 10) : 'not indexed'} · include {source.include_patterns.length} · exclude {source.exclude_patterns.length}</span>
                  </div>
                  <button className="danger-button" onClick={() => removeSource(source.id)}>Remove</button>
                </article>
              ))}
              {sources.length === 0 && <p className="empty-state">No git sources configured.</p>}
            </div>
            <form className="inline-form" onSubmit={submitSource}>
              <h3>Add source</h3>
              <input placeholder="https://github.com/org/repo.git" value={sourceForm.uri} onChange={(e) => setSourceForm({ ...sourceForm, uri: e.target.value })} required />
              <input placeholder="branch" value={sourceForm.branch} onChange={(e) => setSourceForm({ ...sourceForm, branch: e.target.value })} />
              <textarea placeholder="include patterns, one per line" value={sourceForm.include} onChange={(e) => setSourceForm({ ...sourceForm, include: e.target.value })} />
              <textarea placeholder="exclude patterns, one per line" value={sourceForm.exclude} onChange={(e) => setSourceForm({ ...sourceForm, exclude: e.target.value })} />
              <button>Add git source</button>
            </form>
          </>
        ) : <p className="empty-state">Create a project to configure sources.</p>}
        {message && <p className="operation-message">{message}</p>}
      </section>
    </div>
  );
}


function JobsManager({ session, projects }: { session: Session; projects: Project[] }) {
  const [projectId, setProjectId] = useState('');
  const [sources, setSources] = useState<Source[]>([]);
  const [sourceId, setSourceId] = useState('');
  const [jobs, setJobs] = useState<IndexJob[]>([]);
  const [message, setMessage] = useState('');
  const project = projects.find((item) => item.id === projectId) ?? projects[0];

  useEffect(() => {
    if (!project) return;
    setProjectId(project.id);
    listSources(session.token, project.id).then((items) => {
      setSources(items);
      setSourceId((current) => current || items[0]?.id || '');
    }).catch(() => setSources([]));
  }, [session.token, project?.id]);

  useEffect(() => {
    if (!project) return;
    let cancelled = false;
    async function load() {
      try {
        const items = await listIndexJobs(session.token, project.id);
        if (!cancelled) setJobs(items);
      } catch {
        if (!cancelled) setMessage('job polling failed');
      }
    }
    load();
    const interval = window.setInterval(load, 5000);
    return () => {
      cancelled = true;
      window.clearInterval(interval);
    };
  }, [session.token, project?.id]);

  async function trigger(source?: string) {
    if (!project) return;
    try {
      await createIndexJob(session.token, project.id, source);
      setJobs(await listIndexJobs(session.token, project.id));
      setMessage('index job enqueued');
    } catch {
      setMessage('failed to enqueue index job');
    }
  }

  async function cancel(job: IndexJob) {
    if (!project) return;
    await cancelIndexJob(session.token, job.id).catch(() => undefined);
    setJobs(await listIndexJobs(session.token, project.id).catch(() => jobs));
  }

  function duration(job: IndexJob) {
    if (!job.started_at) return 'not started';
    const end = job.finished_at ? Date.parse(job.finished_at) : Date.now();
    return `${Math.max(0, Math.round((end - Date.parse(job.started_at)) / 1000))}s`;
  }

  if (!project) return <p className="empty-state">Create a project before monitoring jobs.</p>;

  return (
    <section>
      <div className="panel-heading">
        <h2>Index jobs · {project.slug}</h2>
        <span>polling 5s</span>
      </div>
      <div className="job-toolbar">
        <select value={project.id} onChange={(event) => setProjectId(event.target.value)}>
          {projects.map((item) => <option key={item.id} value={item.id}>{item.slug}</option>)}
        </select>
        <select value={sourceId} onChange={(event) => setSourceId(event.target.value)}>
          {sources.map((source) => <option key={source.id} value={source.id}>{source.uri}</option>)}
        </select>
        <button onClick={() => trigger(sourceId || undefined)} disabled={!sourceId}>Trigger index</button>
      </div>
      <div className="job-list">
        {jobs.map((job) => (
          <article className="job-row" key={job.id}>
            <div className="job-main">
              <div className="job-title">
                <strong>{job.status}</strong>
                <code>{job.id.slice(0, 8)}</code>
              </div>
              <div className="progress-track"><span style={{ width: `${job.progress_percent}%` }} /></div>
              <div className="job-meta">
                <span>{job.current_stage ?? 'waiting'}</span>
                <span>{job.progress_percent}%</span>
                <span>attempt {job.attempts}/{job.max_attempts}</span>
                <span>{duration(job)}</span>
                {job.generation && <span>gen {job.generation}</span>}
              </div>
              {job.error && <p className="job-error">{job.error}</p>}
            </div>
            <div className="job-actions">
              {(job.status === 'queued' || job.status === 'running') && <button onClick={() => cancel(job)}>Cancel</button>}
              {(job.status === 'failed' || job.status === 'cancelled') && <button onClick={() => trigger(job.source_id ?? undefined)}>Retry</button>}
            </div>
          </article>
        ))}
        {jobs.length === 0 && <p className="empty-state">No indexing jobs for this project.</p>}
      </div>
      {message && <p className="operation-message">{message}</p>}
    </section>
  );
}

function TokensUsersManager({ session, projects }: { session: Session; projects: Project[] }) {
  const [tokens, setTokens] = useState<ApiToken[]>([]);
  const [users, setUsers] = useState<TenantUser[]>([]);
  const [tokenForm, setTokenForm] = useState({ name: 'mcp-client', scopes: 'search:read,code:read', projectId: '', days: '30' });
  const [userForm, setUserForm] = useState({ email: '', display_name: '', password: '', role: 'viewer' });
  const [createdToken, setCreatedToken] = useState('');
  const [message, setMessage] = useState('');

  async function load() {
    const [nextTokens, nextUsers] = await Promise.all([listApiTokens(session.token), listUsers(session.token)]);
    setTokens(nextTokens);
    setUsers(nextUsers);
  }

  useEffect(() => { load().catch(() => setMessage('failed to load tokens/users')); }, [session.token]);

  async function submitToken(event: FormEvent) {
    event.preventDefault();
    const projectIds = tokenForm.projectId ? [tokenForm.projectId] : undefined;
    const created = await createApiToken(session.token, {
      name: tokenForm.name,
      scopes: tokenForm.scopes.split(',').map((scope) => scope.trim()).filter(Boolean),
      project_ids: projectIds,
      expires_in_days: tokenForm.days ? Number(tokenForm.days) : undefined,
    }).catch(() => undefined);
    if (!created) return setMessage('failed to create token');
    setCreatedToken(created.token);
    setMessage('token created — copy it now; it will not be shown again');
    await load();
  }

  async function submitUser(event: FormEvent) {
    event.preventDefault();
    const created = await createUser(session.token, userForm).catch(() => undefined);
    if (!created) return setMessage('failed to create user');
    setMessage(`user ${created.email} created/updated`);
    setUserForm({ email: '', display_name: '', password: '', role: 'viewer' });
    await load();
  }

  const snippet = createdToken ? JSON.stringify({
    mcpServers: { graphfinder: { url: `${window.location.origin}/mcp`, headers: { Authorization: `Bearer ${createdToken}` } } },
  }, null, 2) : '';

  return (
    <section className="tokens-grid">
      <div>
        <div className="panel-heading"><h2>API tokens</h2><span>{tokens.length} registered</span></div>
        <form className="inline-form compact" onSubmit={submitToken}>
          <input placeholder="token name" value={tokenForm.name} onChange={(e) => setTokenForm({ ...tokenForm, name: e.target.value })} required />
          <input placeholder="scopes comma-separated" value={tokenForm.scopes} onChange={(e) => setTokenForm({ ...tokenForm, scopes: e.target.value })} />
          <select value={tokenForm.projectId} onChange={(e) => setTokenForm({ ...tokenForm, projectId: e.target.value })}>
            <option value="">all allowed projects</option>
            {projects.map((project) => <option key={project.id} value={project.id}>{project.slug}</option>)}
          </select>
          <input type="number" min="1" placeholder="expires days" value={tokenForm.days} onChange={(e) => setTokenForm({ ...tokenForm, days: e.target.value })} />
          <button>Create token</button>
        </form>
        {createdToken && (
          <div className="token-secret-panel">
            <strong>Copy MCP configuration</strong>
            <pre>{snippet}</pre>
          </div>
        )}
        <div className="token-list">
          {tokens.map((token) => (
            <article className="token-row" key={token.id}>
              <div>
                <strong>{token.name}</strong>
                <div className="job-meta">
                  <span>{token.scopes.join(', ')}</span>
                  <span>{token.project_ids?.length ? `${token.project_ids.length} project(s)` : 'all projects'}</span>
                  <span>expires {token.expires_at?.slice(0, 10) ?? 'never'}</span>
                  <span>last used {token.last_used_at ? token.last_used_at.slice(0, 10) : 'never'}</span>
                </div>
              </div>
              {!token.revoked_at && <button onClick={() => revokeApiToken(session.token, token.id).then(load)}>Revoke</button>}
            </article>
          ))}
        </div>
      </div>
      <div>
        <div className="panel-heading"><h2>Users</h2><span>{users.length} members</span></div>
        <form className="inline-form compact" onSubmit={submitUser}>
          <input type="email" placeholder="email" value={userForm.email} onChange={(e) => setUserForm({ ...userForm, email: e.target.value })} required />
          <input placeholder="display name" value={userForm.display_name} onChange={(e) => setUserForm({ ...userForm, display_name: e.target.value })} required />
          <input type="password" placeholder="temporary password" value={userForm.password} onChange={(e) => setUserForm({ ...userForm, password: e.target.value })} required />
          <select value={userForm.role} onChange={(e) => setUserForm({ ...userForm, role: e.target.value })}>
            {['viewer', 'member', 'admin', 'owner'].map((role) => <option key={role}>{role}</option>)}
          </select>
          <button>Create/update user</button>
        </form>
        <div className="token-list">
          {users.map((user) => (
            <article className="token-row" key={user.id}>
              <div>
                <strong>{user.display_name || user.email}</strong>
                <div className="job-meta"><span>{user.email}</span><span>{user.status}</span></div>
              </div>
              <select value={user.role ?? 'viewer'} onChange={(e) => updateUserRole(session.token, user.id, e.target.value).then(load)}>
                {['viewer', 'member', 'admin', 'owner'].map((role) => <option key={role}>{role}</option>)}
              </select>
            </article>
          ))}
        </div>
        {message && <p className="operation-message">{message}</p>}
      </div>
    </section>
  );
}

function SearchLab({ session, projects }: { session: Session; projects: Project[] }) {
  const [projectId, setProjectId] = useState(projects[0]?.id ?? '');
  const [query, setQuery] = useState('');
  const [mode, setMode] = useState('hybrid');
  const [language, setLanguage] = useState('');
  const [documentType, setDocumentType] = useState('');
  const [hits, setHits] = useState<SearchHit[]>([]);
  const [message, setMessage] = useState('');
  const [searching, setSearching] = useState(false);

  useEffect(() => {
    if (!projectId && projects[0]) setProjectId(projects[0].id);
  }, [projectId, projects]);

  async function submit(event: FormEvent) {
    event.preventDefault();
    if (!projectId || !query.trim()) return;
    setSearching(true);
    setMessage('');
    try {
      const results = await searchProject(session.token, {
        project_id: projectId,
        query,
        mode,
        language: language || undefined,
        document_type: documentType || undefined,
        limit: 20,
      });
      setHits(results);
      if (results.length === 0) setMessage('no results in active generation');
    } catch {
      setMessage('search failed — check indexing status and embedding availability');
    } finally {
      setSearching(false);
    }
  }

  return (
    <section>
      <div className="panel-heading"><h2>Search inspection</h2><span>active generation only</span></div>
      <form className="search-console" onSubmit={submit}>
        <div className="search-query-row">
          <select value={projectId} onChange={(event) => setProjectId(event.target.value)}>
            {projects.map((project) => <option key={project.id} value={project.id}>{project.slug}</option>)}
          </select>
          <input value={query} onChange={(event) => setQuery(event.target.value)} placeholder="semantic query, symbol, concept or phrase" required />
          <button disabled={searching || !projectId}>{searching ? 'Searching…' : 'Run search'}</button>
        </div>
        <div className="search-filter-row">
          <select value={mode} onChange={(event) => setMode(event.target.value)}>
            <option value="hybrid">hybrid</option>
            <option value="lexical">lexical</option>
            <option value="vector">vector</option>
          </select>
          <input value={language} onChange={(event) => setLanguage(event.target.value)} placeholder="language filter (rust, markdown…)" />
          <input value={documentType} onChange={(event) => setDocumentType(event.target.value)} placeholder="document type filter" />
        </div>
      </form>
      <div className="search-results">
        {hits.map((hit) => {
          const heading = hit.heading_path.join(' / ');
          const deepLink = `[[${hit.path}${heading ? `#${hit.heading_path[hit.heading_path.length - 1]}` : ''}]]`;
          return (
            <article className="search-hit" key={hit.chunk_id}>
              <header>
                <div>
                  <strong>{hit.path}</strong>
                  {heading && <span>{heading}</span>}
                </div>
                <code>{hit.score.toFixed(4)}</code>
              </header>
              <p>{hit.content.length > 420 ? `${hit.content.slice(0, 420)}…` : hit.content}</p>
              <footer><code>{deepLink}</code></footer>
            </article>
          );
        })}
        {hits.length === 0 && <p className="empty-state">Run a query to inspect ranked chunks, scores and deep links.</p>}
      </div>
      {message && <p className="operation-message">{message}</p>}
    </section>
  );
}

const navItems = ['Overview', 'Projects', 'Sources', 'Index jobs', 'API tokens', 'Search lab'];

function App() {
  const [session, setSession] = useState<Session | null>(() => loadSession());
  const [active, setActive] = useState('Overview');
  const [projects, setProjects] = useState<Project[]>([]);
  const [status, setStatus] = useState('idle');

  useEffect(() => {
    if (!session) return;
    setStatus('loading');
    listProjects(session.token)
      .then((items) => {
        setProjects(items);
        setStatus('ready');
      })
      .catch(() => setStatus('error'));
  }, [session]);

  const metrics = useMemo(() => [
    ['Projects', projects.length],
    ['Active generations', projects.filter((p) => p.active_generation > 0).length],
    ['Control plane', status === 'error' ? 'degraded' : 'online'],
  ], [projects, status]);

  if (!session) return <LoginScreen onLogin={setSession} />;

  async function signOut() {
    if (session) await logout(session.token).catch(() => undefined);
    saveSession(null);
    setSession(null);
  }

  return (
    <main className="console-shell">
      <aside className="sidebar">
        <div className="sidebar-brand"><span>GF</span><strong>Server</strong></div>
        <nav>
          {navItems.map((item) => (
            <button key={item} className={item === active ? 'active' : ''} onClick={() => setActive(item)}>
              {item}
            </button>
          ))}
        </nav>
        <button className="logout" onClick={signOut}>Logout</button>
      </aside>
      <section className="workspace">
        <header className="workspace-header">
          <div>
            <p className="eyebrow">{active}</p>
            <h1>{active === 'Overview' ? 'Indexing operations at a glance.' : `${active} workspace`}</h1>
          </div>
          <div className="session-chip">tenant · {session.tenant_id.slice(0, 8)}</div>
        </header>
        <section className="metric-grid">
          {metrics.map(([label, value]) => (
            <article className="metric-card" key={label}>
              <span>{label}</span>
              <strong>{value}</strong>
            </article>
          ))}
        </section>
        <section className="content-panel">
          {active === 'Search lab' ? (
            <SearchLab session={session} projects={projects} />
          ) : active === 'API tokens' ? (
            <TokensUsersManager session={session} projects={projects} />
          ) : active === 'Index jobs' ? (
            <JobsManager session={session} projects={projects} />
          ) : active === 'Projects' || active === 'Overview' || active === 'Sources' ? (
            <ProjectsManager session={session} projects={projects} setProjects={setProjects} status={status} />
          ) : (
            <div className="empty-state large">{active} module scaffolded. Data wiring arrives in the next implementation step.</div>
          )}
        </section>
      </section>
    </main>
  );
}

export default App;
