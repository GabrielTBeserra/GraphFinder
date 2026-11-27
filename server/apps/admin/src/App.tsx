import { FormEvent, useEffect, useMemo, useState } from 'react';
import {
  ApiToken,
  AuditEvent,
  cancelIndexJob,
  createApiToken,
  createIndexJob,
  createProject,
  createUser,
  createGitProvider,
  configureSourceWebhook,
  getOidcConfig,
  createSource,
  deleteSource,
  deleteGitProvider,
  GitProvider,
  IndexGeneration,
  IndexJob,
  listApiTokens,
  listAuditEvents,
  listIndexGenerations,
  listIndexJobs,
  listGitProviders,
  listProjects,
  listUsers,
  listSources,
  loadSession,
  login,
  logout,
  Project,
  revokeApiToken,
  rollbackIndexGeneration,
  saveSession,
  searchProject,
  SearchHit,
  Session,
  Source,
  TenantUser,
  OidcPublicConfig,
  updateProject,
  updateUserRole,
} from './api';

function LoginScreen({ onLogin }: { onLogin: (session: Session) => void }) {
  const [email, setEmail] = useState('admin@example.com');
  const [password, setPassword] = useState('');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);
  const [oidcConfig, setOidcConfig] = useState<OidcPublicConfig | null>(null);

  useEffect(() => {
    if (window.location.hash.startsWith('#oidc_token=')) {
      const params = new URLSearchParams(window.location.hash.slice(1));
      const token = params.get('oidc_token');
      const expiresAt = params.get('expires_at');
      const userId = params.get('user_id');
      const tenantId = params.get('tenant_id');
      if (token && expiresAt && userId && tenantId) {
        const session = { token, expires_at: expiresAt, user_id: userId, tenant_id: tenantId };
        saveSession(session);
        window.history.replaceState(null, '', window.location.pathname + window.location.search);
        onLogin(session);
        return;
      }
    }
    getOidcConfig().then(setOidcConfig).catch(() => setOidcConfig({ enabled: false, button_label: 'Continue with SSO' }));
  }, [onLogin]);

  async function submit(event: FormEvent) {
    event.preventDefault();
    setLoading(true);
    setError('');
    try {
      const session = await login(email, password);
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
            Email
            <input type="email" value={email} onChange={(event) => setEmail(event.target.value)} required />
          </label>
          <label>
            Password
            <input type="password" value={password} onChange={(event) => setPassword(event.target.value)} required />
          </label>
          {error && <p className="form-error">{error}</p>}
          <button disabled={loading}>{loading ? 'Authenticating…' : 'Enter console'}</button>
          {oidcConfig?.enabled && (
            <a className="sso-button" href="/api/v1/auth/oidc/start">
              {oidcConfig.button_label}
            </a>
          )}
        </form>
      </section>
    </main>
  );
}


type OverviewProjectData = {
  project: Project;
  sources: Source[];
  jobs: IndexJob[];
  generations: IndexGeneration[];
};

const overviewStatuses = [
  ['running', 'Running'],
  ['queued', 'Queued'],
  ['succeeded', 'Succeeded'],
  ['failed', 'Failed'],
  ['cancelled', 'Cancelled'],
] as const;

function formatCompactDate(value?: string | null) {
  if (!value) return '—';
  return new Intl.DateTimeFormat(undefined, { month: 'short', day: '2-digit', hour: '2-digit', minute: '2-digit' }).format(new Date(value));
}

function localDayKey(value: string | Date) {
  const date = typeof value === 'string' ? new Date(value) : value;
  return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}-${String(date.getDate()).padStart(2, '0')}`;
}

function formatDuration(startedAt?: string | null, finishedAt?: string | null) {
  if (!startedAt) return '—';
  const start = Date.parse(startedAt);
  const end = finishedAt ? Date.parse(finishedAt) : Date.now();
  const minutes = Math.max(0, Math.round((end - start) / 60000));
  if (minutes < 1) return '<1m';
  if (minutes < 60) return `${minutes}m`;
  return `${Math.floor(minutes / 60)}h ${minutes % 60}m`;
}

function OverviewDashboard({ session, projects, status }: { session: Session; projects: Project[]; status: string }) {
  const [rows, setRows] = useState<OverviewProjectData[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');

  useEffect(() => {
    let cancelled = false;
    let timer: number | undefined;

    async function load(initial = false) {
      if (initial) setLoading(true);
      if (projects.length === 0) {
        if (!cancelled) {
          setRows([]);
          setError('');
          setLoading(false);
          timer = window.setTimeout(() => void load(), 5000);
        }
        return;
      }
      try {
        const data = await Promise.all(projects.map(async (project) => {
          const [sources, jobs, generations] = await Promise.all([
            listSources(session.token, project.id),
            listIndexJobs(session.token, project.id),
            listIndexGenerations(session.token, project.id),
          ]);
          return { project, sources, jobs, generations };
        }));
        if (!cancelled) {
          setRows(data);
          setError('');
        }
      } catch {
        if (!cancelled) setError('overview telemetry unavailable');
      } finally {
        if (!cancelled) {
          if (initial) setLoading(false);
          timer = window.setTimeout(() => void load(), 5000);
        }
      }
    }

    void load(true);
    return () => {
      cancelled = true;
      if (timer) window.clearTimeout(timer);
    };
  }, [session.token, projects]);

  const allJobs = rows.flatMap((row) => row.jobs);
  const allSources = rows.flatMap((row) => row.sources);
  const countByStatus = (jobStatus: string) => allJobs.filter((job) => job.status === jobStatus).length;
  const runningCount = countByStatus('running');
  const queuedCount = countByStatus('queued');
  const succeededCount = countByStatus('succeeded');
  const failedCount = countByStatus('failed');
  const cancelledCount = countByStatus('cancelled');
  const terminalCount = succeededCount + failedCount + cancelledCount;
  const successRate = terminalCount ? Math.round((succeededCount / terminalCount) * 100) : 0;
  const successRateLabel = terminalCount ? `${successRate}%` : '—';
  const indexedProjects = projects.filter((project) => project.active_generation > 0).length;
  const indexedSources = allSources.filter((source) => Boolean(source.last_indexed_commit)).length;
  const sourceCoverage = allSources.length ? Math.round((indexedSources / allSources.length) * 100) : 0;
  const maxGeneration = Math.max(1, ...projects.map((project) => project.active_generation));
  const projectNames = new Map(rows.map((row) => [row.project.id, row.project.name]));
  const recentJobs = [...allJobs].sort((a, b) => Date.parse(b.created_at) - Date.parse(a.created_at)).slice(0, 5);

  const activity = (() => {
    const days = Array.from({ length: 7 }, (_, index) => {
      const day = new Date();
      day.setDate(day.getDate() - (6 - index));
      day.setHours(0, 0, 0, 0);
      return {
        key: localDayKey(day),
        label: new Intl.DateTimeFormat(undefined, { weekday: 'short' }).format(day),
        count: 0,
      };
    });
    const counts = new Map(days.map((day) => [day.key, day]));
    allJobs.forEach((job) => {
      const day = counts.get(localDayKey(job.created_at));
      if (day) day.count += 1;
    });
    return { days, max: Math.max(1, ...days.map((day) => day.count)) };
  })();

  if (loading) return <div className="empty-state large">Collecting operational telemetry…</div>;

  if (projects.length === 0) {
    return (
      <div className="overview-empty">
        <span className="overview-empty-mark">GF</span>
        <h2>No indexing telemetry yet</h2>
        <p>Projects, sources, generations and execution health will appear here as soon as the private server starts indexing.</p>
      </div>
    );
  }

  return (
    <div className="overview-dashboard">
      {error && <p className="operation-message">{error}</p>}
      <section className="overview-stat-grid" aria-label="Operational statistics">
        <article className="overview-stat hero">
          <span>Indexed projects</span>
          <strong>{indexedProjects}<small>/{projects.length}</small></strong>
          <em>{projects.length - indexedProjects} awaiting publication</em>
        </article>
        <article className="overview-stat">
          <span>Git sources</span>
          <strong>{allSources.length}</strong>
          <em>{sourceCoverage}% already indexed</em>
        </article>
        <article className="overview-stat">
          <span>Execution pressure</span>
          <strong>{runningCount + queuedCount}</strong>
          <em>{runningCount} running · {queuedCount} queued</em>
        </article>
        <article className={`overview-stat ${failedCount ? 'danger' : ''}`}>
          <span>Attention</span>
          <strong>{failedCount}</strong>
          <em>{failedCount === 1 ? 'failed job' : 'failed jobs'} need review</em>
        </article>
      </section>

      <section className="overview-chart-grid">
        <article className="overview-panel large-panel">
          <div className="panel-heading compact-heading"><h2>Indexing activity · 7 days</h2><span>live · 5s</span></div>
          <div className="activity-chart" role="img" aria-label={`${allJobs.length} indexing jobs in the last seven days`}>
            {activity.days.map((day) => (
              <div className="activity-column" key={day.key}>
                <div className="activity-bar-shell">
                  <span style={{ height: `${Math.max(day.count ? 10 : 2, (day.count / activity.max) * 100)}%` }} />
                </div>
                <strong>{day.count}</strong>
                <small>{day.label}</small>
              </div>
            ))}
          </div>
        </article>

        <article className="overview-panel success-panel">
          <div className="panel-heading compact-heading"><h2>Success rate</h2><span>{terminalCount} terminal</span></div>
          <div className="success-orbit" style={{ background: `conic-gradient(var(--accent) 0 ${successRate * 3.6}deg, rgba(255,255,255,.08) ${successRate * 3.6}deg 360deg)` }}>
            <div><strong>{successRateLabel}</strong><span>succeeded</span></div>
          </div>
          <p>{terminalCount ? `${succeededCount} successful · ${failedCount} failed · ${cancelledCount} cancelled` : 'No terminal executions yet'}</p>
        </article>

        <article className="overview-panel">
          <div className="panel-heading compact-heading"><h2>Job status mix</h2><span>{allJobs.length} total</span></div>
          <div className="status-bars">
            {overviewStatuses.map(([jobStatus, label]) => {
              const count = countByStatus(jobStatus);
              const width = allJobs.length ? (count / allJobs.length) * 100 : 0;
              return (
                <div className="status-bar-row" key={jobStatus}>
                  <span>{label}</span>
                  <div className="status-track"><i className={`status-fill ${jobStatus}`} style={{ width: `${width}%` }} /></div>
                  <strong>{count}</strong>
                </div>
              );
            })}
          </div>
        </article>

        <article className="overview-panel">
          <div className="panel-heading compact-heading"><h2>System signals</h2><span>{status}</span></div>
          <div className="signal-list">
            <div className="signal-row">
              <i className={runningCount || queuedCount ? 'signal active' : 'signal'} />
              <div><strong>Worker queue</strong><span>{runningCount || queuedCount ? 'processing index jobs' : 'idle and ready'}</span></div>
              <code>{runningCount + queuedCount}</code>
            </div>
            <div className="signal-row">
              <i className={sourceCoverage === 100 && allSources.length ? 'signal good' : 'signal active'} />
              <div><strong>Source coverage</strong><span>{indexedSources} of {allSources.length} sources indexed</span></div>
              <code>{sourceCoverage}%</code>
            </div>
            <div className="signal-row">
              <i className={failedCount ? 'signal danger' : 'signal good'} />
              <div><strong>Error surface</strong><span>{failedCount ? 'review failed executions' : 'no failed jobs detected'}</span></div>
              <code>{failedCount}</code>
            </div>
          </div>
        </article>
      </section>

      <section className="overview-bottom-grid">
        <article className="overview-panel large-panel">
          <div className="panel-heading compact-heading"><h2>Project generation map</h2><span>{projects.length} projects</span></div>
          <div className="project-stat-list">
            {rows.map((row) => {
              const rowJobs = row.jobs.length;
              const rowFailed = row.jobs.filter((job) => job.status === 'failed').length;
              return (
                <div className="project-stat-row" key={row.project.id}>
                  <div className="project-stat-name">
                    <strong>{row.project.name}</strong>
                    <span>{row.project.slug} · {row.sources.length} sources · {row.generations.length} generations</span>
                  </div>
                  <div className="generation-meter">
                    <div><span style={{ width: `${(row.project.active_generation / maxGeneration) * 100}%` }} /></div>
                    <code>gen {row.project.active_generation}</code>
                  </div>
                  <div className="project-stat-numbers">
                    <span>{rowJobs} jobs</span>
                    <span className={rowFailed ? 'danger-text' : ''}>{rowFailed} failed</span>
                  </div>
                </div>
              );
            })}
          </div>
        </article>

        <article className="overview-panel">
          <div className="panel-heading compact-heading"><h2>Latest executions</h2><span>newest first</span></div>
          <div className="recent-job-list">
            {recentJobs.map((job) => (
              <div className="recent-job" key={job.id}>
                <div>
                  <strong>{projectNames.get(job.project_id) ?? 'unknown project'}</strong>
                  <span>{job.id.slice(0, 8)} · {job.current_stage ?? job.status}</span>
                </div>
                <div className="recent-job-meta">
                  <code className={`job-status ${job.status}`}>{job.status}</code>
                  <span>{formatCompactDate(job.created_at)}</span>
                  <span>{formatDuration(job.started_at, job.finished_at)}</span>
                </div>
              </div>
            ))}
            {recentJobs.length === 0 && <p className="empty-state">No indexing executions recorded yet.</p>}
          </div>
        </article>
      </section>
    </div>
  );
}


function ProjectsManager({ session, projects, setProjects, status }: {
  session: Session;
  projects: Project[];
  setProjects: (projects: Project[]) => void;
  status: string;
}) {
  const [selectedId, setSelectedId] = useState('');
  const [projectForm, setProjectForm] = useState({ slug: '', name: '', description: '' });
  const [editName, setEditName] = useState('');
  const [message, setMessage] = useState('');
  const selected = projects.find((project) => project.id === selectedId) ?? projects[0];

  useEffect(() => {
    if (!selected) return;
    setSelectedId(selected.id);
    setEditName(selected.name);
  }, [selected?.id, selected?.name]);

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
            <div className="project-fact-grid">
              <div><span>Slug</span><strong>{selected.slug}</strong></div>
              <div><span>Active generation</span><strong>{selected.active_generation}</strong></div>
              <div><span>Created</span><strong>{formatCompactDate(selected.created_at)}</strong></div>
              <div><span>Updated</span><strong>{formatCompactDate(selected.updated_at)}</strong></div>
            </div>
            <p className="workspace-note">Git repositories and include/exclude rules are managed in the Sources workspace.</p>
          </>
        ) : <p className="empty-state">Create a project to start organizing indexed knowledge.</p>}
        {message && <p className="operation-message">{message}</p>}
      </section>
    </div>
  );
}


function SourcesManager({ session, projects }: { session: Session; projects: Project[] }) {
  const [projectId, setProjectId] = useState('');
  const [sources, setSources] = useState<Source[]>([]);
  const [providers, setProviders] = useState<GitProvider[]>([]);
  const [sourceForm, setSourceForm] = useState({ uri: '', branch: 'main', include: '', exclude: '', providerId: '', automation: 'manual', webhookProvider: 'github' });
  const [pushForm, setPushForm] = useState({ sourceId: '', provider: 'github' });
  const [webhookSetup, setWebhookSetup] = useState<{ endpoint: string; secret: string; provider: string } | null>(null);
  const [message, setMessage] = useState('');
  const project = projects.find((item) => item.id === projectId) ?? projects[0];

  useEffect(() => {
    listGitProviders(session.token).then(setProviders).catch(() => setProviders([]));
  }, [session.token]);

  useEffect(() => {
    if (!project) {
      setSources([]);
      return;
    }
    setProjectId(project.id);
    listSources(session.token, project.id).then(setSources).catch(() => setSources([]));
  }, [session.token, project?.id]);

  function randomSecret() {
    return Array.from(crypto.getRandomValues(new Uint8Array(24)), (byte) => byte.toString(16).padStart(2, '0')).join('');
  }

  async function enablePush(sourceId: string, provider: string) {
    const secret = randomSecret();
    const config = await configureSourceWebhook(session.token, sourceId, { provider, secret, enabled: true });
    setWebhookSetup({ endpoint: `${window.location.origin}${config.endpoint_path}`, secret, provider });
    return config;
  }

  async function submitSource(event: FormEvent) {
    event.preventDefault();
    if (!project) return;
    const split = (value: string) => value.split('\n').map((item) => item.trim()).filter(Boolean);
    try {
      const interval = /^\d+$/.test(sourceForm.automation) ? Number(sourceForm.automation) : null;
      const created = await createSource(session.token, project.id, {
        kind: 'git',
        uri: sourceForm.uri,
        branch: sourceForm.branch || null,
        credential_ref: sourceForm.providerId || null,
        auto_index_interval_seconds: interval,
        include_patterns: split(sourceForm.include),
        exclude_patterns: split(sourceForm.exclude),
      });
      if (sourceForm.automation === 'push') await enablePush(created.id, sourceForm.webhookProvider);
      setSourceForm({ uri: '', branch: 'main', include: '', exclude: '', providerId: '', automation: 'manual', webhookProvider: 'github' });
      setSources(await listSources(session.token, project.id));
      setMessage('source created');
    } catch {
      setMessage('source creation failed');
    }
  }

  async function submitPush(event: FormEvent) {
    event.preventDefault();
    if (!pushForm.sourceId) return;
    try {
      await enablePush(pushForm.sourceId, pushForm.provider);
      setMessage('push webhook enabled — copy the secret below now');
    } catch {
      setMessage('webhook configuration failed');
    }
  }

  async function removeSource(id: string) {
    if (!project) return;
    await deleteSource(session.token, id).catch(() => undefined);
    setSources(await listSources(session.token, project.id).catch(() => []));
  }

  if (!project) return <p className="empty-state">Create a project before configuring git sources.</p>;

  const providerName = (id?: string | null) => providers.find((provider) => provider.id === id)?.name ?? 'none';

  return (
    <section>
      <div className="panel-heading">
        <h2>Git sources · {project.slug}</h2>
        <span>{sources.length} configured</span>
      </div>
      <div className="source-toolbar">
        <select value={project.id} onChange={(event) => setProjectId(event.target.value)}>
          {projects.map((item) => <option key={item.id} value={item.id}>{item.slug}</option>)}
        </select>
      </div>
      <div className="source-manager-grid">
        <section>
          <h3 className="section-label">Configured sources</h3>
          {sources.map((source) => (
            <article className="source-row" key={source.id}>
              <div>
                <strong>{source.uri}</strong>
                <span>branch {source.branch ?? 'default'} · {source.indexing_status} · commit {source.last_indexed_commit ? source.last_indexed_commit.slice(0, 10) : 'not indexed'} · include {source.include_patterns.length} · exclude {source.exclude_patterns.length}</span>
                <span>auth {providerName(source.credential_ref)} · automation {source.auto_index_interval_seconds ? `every ${Math.round(source.auto_index_interval_seconds / 60)}m` : 'manual/push'} · next {source.next_index_at ? formatCompactDate(source.next_index_at) : '—'}</span>
              </div>
              <button className="danger-button" onClick={() => removeSource(source.id)}>Remove</button>
            </article>
          ))}
          {sources.length === 0 && <p className="empty-state">No git sources configured for this project.</p>}
        </section>
        <section>
          <form className="inline-form source-form" onSubmit={submitSource}>
            <h3>Add git source</h3>
            <input placeholder="https://github.com/org/repo.git" value={sourceForm.uri} onChange={(e) => setSourceForm({ ...sourceForm, uri: e.target.value })} required />
            <input placeholder="branch" value={sourceForm.branch} onChange={(e) => setSourceForm({ ...sourceForm, branch: e.target.value })} />
            <select value={sourceForm.providerId} onChange={(e) => setSourceForm({ ...sourceForm, providerId: e.target.value })}>
              <option value="">public repository / no provider</option>
              {providers.map((provider) => <option key={provider.id} value={provider.id}>{provider.name} · {provider.host}</option>)}
            </select>
            <select value={sourceForm.automation} onChange={(e) => setSourceForm({ ...sourceForm, automation: e.target.value })}>
              <option value="manual">manual indexing</option>
              <option value="push">after push · webhook</option>
              <option value="300">every 5 minutes</option>
              <option value="900">every 15 minutes</option>
              <option value="3600">every hour</option>
              <option value="21600">every 6 hours</option>
            </select>
            {sourceForm.automation === 'push' && (
              <select value={sourceForm.webhookProvider} onChange={(e) => setSourceForm({ ...sourceForm, webhookProvider: e.target.value })}>
                <option value="github">GitHub webhook</option>
                <option value="gitlab">GitLab webhook</option>
                <option value="azure_devops">Azure DevOps service hook</option>
                <option value="generic">generic HMAC webhook</option>
              </select>
            )}
            <textarea placeholder="include patterns, one per line" value={sourceForm.include} onChange={(e) => setSourceForm({ ...sourceForm, include: e.target.value })} />
            <textarea placeholder="exclude patterns, one per line" value={sourceForm.exclude} onChange={(e) => setSourceForm({ ...sourceForm, exclude: e.target.value })} />
            <button>Add git source</button>
          </form>
          <form className="inline-form source-form" onSubmit={submitPush}>
            <h3>Enable after-push indexing</h3>
            <select value={pushForm.sourceId} onChange={(e) => setPushForm({ ...pushForm, sourceId: e.target.value })} required>
              <option value="">select source</option>
              {sources.map((source) => <option key={source.id} value={source.id}>{source.uri}</option>)}
            </select>
            <select value={pushForm.provider} onChange={(e) => setPushForm({ ...pushForm, provider: e.target.value })}>
              <option value="github">GitHub webhook</option>
              <option value="gitlab">GitLab webhook</option>
              <option value="azure_devops">Azure DevOps service hook</option>
              <option value="generic">generic HMAC webhook</option>
            </select>
            <button>Generate webhook configuration</button>
          </form>
          {webhookSetup && (
            <div className="token-secret-panel">
              <strong>Configure this webhook once — the secret is shown only now</strong>
              <pre>{`URL: ${webhookSetup.endpoint}\nSecret/password: ${webhookSetup.secret}\nProvider: ${webhookSetup.provider}${webhookSetup.provider === 'azure_devops' ? '\nAzure basic auth username: graphfinder' : ''}`}</pre>
            </div>
          )}
          {message && <p className="operation-message">{message}</p>}
        </section>
      </div>
    </section>
  );
}

function ProvidersManager({ session }: { session: Session }) {
  const [providers, setProviders] = useState<GitProvider[]>([]);
  const [form, setForm] = useState({ name: '', provider: 'github', host: 'github.com', username: '', token: '' });
  const [message, setMessage] = useState('');

  async function load() {
    setProviders(await listGitProviders(session.token));
  }
  useEffect(() => { load().catch(() => setMessage('failed to load git providers')); }, [session.token]);

  async function submit(event: FormEvent) {
    event.preventDefault();
    try {
      await createGitProvider(session.token, {
        name: form.name,
        provider: form.provider,
        host: form.host,
        username: form.username || undefined,
        token: form.token,
      });
      setForm({ name: '', provider: form.provider, host: form.host, username: '', token: '' });
      setMessage('provider credential stored encrypted');
      await load();
    } catch {
      setMessage('provider creation failed — check encryption key and host');
    }
  }

  async function remove(id: string) {
    try {
      await deleteGitProvider(session.token, id);
      setMessage('provider removed');
      await load();
    } catch {
      setMessage('provider is still used by a source');
    }
  }

  return (
    <section className="tokens-grid">
      <div>
        <div className="panel-heading"><h2>Git providers</h2><span>{providers.length} configured</span></div>
        <div className="token-list">
          {providers.map((provider) => (
            <article className="token-row" key={provider.id}>
              <div>
                <strong>{provider.name}</strong>
                <div className="job-meta">
                  <span>{provider.provider}</span><span>{provider.host}</span><span>user {provider.username}</span><span>token {provider.secret_hint}</span><span>{provider.sources_count} source(s)</span><span>last used {provider.last_used_at ? formatCompactDate(provider.last_used_at) : 'never'}</span>
                </div>
              </div>
              <button onClick={() => remove(provider.id)}>Remove</button>
            </article>
          ))}
          {providers.length === 0 && <p className="empty-state">No private Git providers configured.</p>}
        </div>
      </div>
      <div>
        <form className="inline-form compact" onSubmit={submit}>
          <input placeholder="credential name" value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })} required />
          <select value={form.provider} onChange={(e) => {
            const provider = e.target.value;
            const host = provider === 'github' ? 'github.com' : provider === 'gitlab' ? 'gitlab.com' : provider === 'azure_devops' ? 'dev.azure.com' : form.host;
            setForm({ ...form, provider, host });
          }}>
            <option value="github">GitHub</option>
            <option value="gitlab">GitLab</option>
            <option value="azure_devops">Azure DevOps</option>
            <option value="generic">Generic HTTPS Git</option>
          </select>
          <input placeholder="host" value={form.host} onChange={(e) => setForm({ ...form, host: e.target.value })} required />
          <input placeholder="username (optional for known providers)" value={form.username} onChange={(e) => setForm({ ...form, username: e.target.value })} />
          <input type="password" placeholder="personal access token" value={form.token} onChange={(e) => setForm({ ...form, token: e.target.value })} required />
          <small className="field-hint">Tokens are encrypted at rest with GF_CREDENTIALS_ENCRYPTION_KEY and are never returned by the API.</small>
          <button>Store provider credential</button>
        </form>
        {message && <p className="operation-message">{message}</p>}
      </div>
    </section>
  );
}


function JobsManager({ session, projects, setProjects }: { session: Session; projects: Project[]; setProjects: (projects: Project[]) => void }) {
  const [projectId, setProjectId] = useState('');
  const [sources, setSources] = useState<Source[]>([]);
  const [sourceId, setSourceId] = useState('');
  const [jobs, setJobs] = useState<IndexJob[]>([]);
  const [generations, setGenerations] = useState<IndexGeneration[]>([]);
  const [rollbackTarget, setRollbackTarget] = useState<IndexGeneration | null>(null);
  const [rollbackConfirm, setRollbackConfirm] = useState('');
  const [rollbackReason, setRollbackReason] = useState('');
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
        const [items, generationItems] = await Promise.all([
          listIndexJobs(session.token, project.id),
          listIndexGenerations(session.token, project.id),
        ]);
        if (!cancelled) {
          setJobs(items);
          setGenerations(generationItems);
        }
      } catch {
        if (!cancelled) setMessage('job polling failed');
      }
    }
    load();
    const interval = window.setInterval(load, 1000);
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

  async function refreshOperations() {
    if (!project) return;
    const [nextJobs, nextGenerations, nextProjects] = await Promise.all([
      listIndexJobs(session.token, project.id),
      listIndexGenerations(session.token, project.id),
      listProjects(session.token),
    ]);
    setJobs(nextJobs);
    setGenerations(nextGenerations);
    setProjects(nextProjects);
  }

  function beginRollback(generation: IndexGeneration) {
    setRollbackTarget(generation);
    setRollbackConfirm('');
    setRollbackReason('');
    setMessage('');
  }

  async function confirmRollback() {
    if (!project || !rollbackTarget) return;
    if (rollbackConfirm.trim() !== String(rollbackTarget.generation)) {
      setMessage('type the exact generation number to confirm rollback');
      return;
    }
    try {
      await rollbackIndexGeneration(session.token, project.id, rollbackTarget.generation, rollbackReason.trim() || undefined);
      setRollbackTarget(null);
      setRollbackConfirm('');
      setRollbackReason('');
      await refreshOperations();
      setMessage(`rolled back to generation ${rollbackTarget.generation}`);
    } catch (error) {
      setMessage(error instanceof Error ? error.message : 'rollback failed');
    }
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
        <span>live · 1s</span>
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
      <div className="generation-block">
        <div className="panel-heading compact-heading"><h3>Index generations</h3><span>{generations.length} recorded</span></div>
        {generations.map((generation) => (
          <article className={`generation-row ${generation.is_active ? 'active' : ''}`} key={generation.generation}>
            <div>
              <strong>generation {generation.generation}</strong>
              <div className="job-meta">
                <span>{generation.status}</span>
                <span>{generation.embedding_model}</span>
                <span>published {generation.published_at ? generation.published_at.slice(0, 10) : 'never'}</span>
                {generation.is_active && <span>active</span>}
              </div>
              {generation.error && <p className="job-error">{generation.error}</p>}
            </div>
            {!generation.is_active && generation.status === 'archived' && (
              <button className="danger-button" onClick={() => beginRollback(generation)}>Rollback</button>
            )}
          </article>
        ))}
        {generations.length === 0 && <p className="empty-state">No generations recorded yet.</p>}
        {rollbackTarget && (
          <div className="rollback-confirm">
            <strong>Confirm rollback to generation {rollbackTarget.generation}</strong>
            <p>This switches the active search generation. Active indexing jobs must be cancelled first. Type <code>{rollbackTarget.generation}</code> to confirm.</p>
            <div className="rollback-actions">
              <input value={rollbackConfirm} onChange={(event) => setRollbackConfirm(event.target.value)} placeholder={`type ${rollbackTarget.generation}`} />
              <input value={rollbackReason} onChange={(event) => setRollbackReason(event.target.value)} placeholder="rollback reason for audit" />
              <button className="danger-button" onClick={confirmRollback} disabled={rollbackConfirm.trim() !== String(rollbackTarget.generation)}>Confirm rollback</button>
              <button onClick={() => setRollbackTarget(null)}>Cancel</button>
            </div>
          </div>
        )}
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

const tokenScopeOptions = [
  ['search:read', 'search:read — semantic and vault search'],
  ['code:read', 'code:read — code search and file inspection'],
  ['index:trigger', 'index:trigger — enqueue indexing jobs'],
  ['vault:write', 'vault:write — create and update managed notes'],
  ['vault:delete', 'vault:delete — delete managed notes'],
  ['project:write', 'project:write — manage projects and sources'],
  ['tokens:manage', 'tokens:manage — manage API tokens'],
] as const;

function TokensUsersManager({ session, projects }: { session: Session; projects: Project[] }) {
  const [tokens, setTokens] = useState<ApiToken[]>([]);
  const [users, setUsers] = useState<TenantUser[]>([]);
  const [tokenForm, setTokenForm] = useState({ name: 'mcp-client', scopes: ['search:read', 'code:read'], projectId: '', days: '30' });
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
    if (tokenForm.scopes.length === 0) {
      setMessage('select at least one scope');
      return;
    }
    const projectIds = tokenForm.projectId ? [tokenForm.projectId] : undefined;
    const created = await createApiToken(session.token, {
      name: tokenForm.name,
      scopes: tokenForm.scopes,
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
          <select
            className="scope-select"
            multiple
            required
            value={tokenForm.scopes}
            onChange={(e) => setTokenForm({ ...tokenForm, scopes: Array.from(e.target.selectedOptions, (option) => option.value) })}
          >
            {tokenScopeOptions.map(([scope, label]) => <option key={scope} value={scope}>{label}</option>)}
          </select>
          <small className="field-hint">Hold Ctrl/Cmd to select multiple scopes.</small>
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


function AuditViewer({ session }: { session: Session }) {
  const [events, setEvents] = useState<AuditEvent[]>([]);
  useEffect(() => { listAuditEvents(session.token).then(setEvents).catch(() => setEvents([])); }, [session.token]);
  return (
    <section>
      <div className="panel-heading"><h2>Mutation audit trail</h2><span>{events.length} recent events</span></div>
      <div className="token-list">
        {events.map((event) => (
          <article className="token-row" key={event.id}>
            <div>
              <strong>{event.action} · {event.resource_type}</strong>
              <div className="job-meta">
                <span>{new Date(event.created_at).toLocaleString()}</span>
                <span>project {event.project_id?.slice(0, 8) ?? 'tenant'}</span>
                <span>actor {event.actor_user_id?.slice(0, 8) ?? 'system'}</span>
              </div>
              <pre className="audit-summary">{JSON.stringify(event.summary)}</pre>
            </div>
          </article>
        ))}
        {events.length === 0 && <p className="empty-state">No mutation events recorded yet.</p>}
      </div>
    </section>
  );
}

const navItems = ['Overview', 'Projects', 'Sources', 'Providers', 'Index jobs', 'API tokens', 'Search lab', 'Audit'];

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
          <div className="session-chip">private server</div>
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
          {active === 'Audit' ? (
            <AuditViewer session={session} />
          ) : active === 'Search lab' ? (
            <SearchLab session={session} projects={projects} />
          ) : active === 'API tokens' ? (
            <TokensUsersManager session={session} projects={projects} />
          ) : active === 'Index jobs' ? (
            <JobsManager session={session} projects={projects} setProjects={setProjects} />
          ) : active === 'Overview' ? (
            <OverviewDashboard session={session} projects={projects} status={status} />
          ) : active === 'Projects' ? (
            <ProjectsManager session={session} projects={projects} setProjects={setProjects} status={status} />
          ) : active === 'Sources' ? (
            <SourcesManager session={session} projects={projects} />
          ) : active === 'Providers' ? (
            <ProvidersManager session={session} />
          ) : (
            <div className="empty-state large">{active} module scaffolded. Data wiring arrives in the next implementation step.</div>
          )}
        </section>
      </section>
    </main>
  );
}

export default App;
