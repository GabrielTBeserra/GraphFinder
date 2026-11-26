import { FormEvent, useEffect, useMemo, useState } from 'react';
import {
  createProject,
  createSource,
  deleteSource,
  listProjects,
  listSources,
  loadSession,
  login,
  logout,
  Project,
  saveSession,
  Session,
  Source,
  updateProject,
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
                    <span>branch {source.branch ?? 'default'} · include {source.include_patterns.length} · exclude {source.exclude_patterns.length}</span>
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
          {active === 'Projects' || active === 'Overview' || active === 'Sources' ? (
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
