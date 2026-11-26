import { FormEvent, useEffect, useMemo, useState } from 'react';
import { listProjects, loadSession, login, logout, Project, saveSession, Session } from './api';

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
          {active === 'Projects' || active === 'Overview' ? (
            <>
              <div className="panel-heading">
                <h2>Projects</h2>
                <span>{status}</span>
              </div>
              <div className="project-list">
                {projects.map((project) => (
                  <article key={project.id} className="project-row">
                    <div>
                      <strong>{project.name}</strong>
                      <span>{project.slug}</span>
                    </div>
                    <code>gen {project.active_generation}</code>
                  </article>
                ))}
                {projects.length === 0 && <p className="empty-state">No projects visible for this principal.</p>}
              </div>
            </>
          ) : (
            <div className="empty-state large">{active} module scaffolded. Data wiring arrives in the next implementation step.</div>
          )}
        </section>
      </section>
    </main>
  );
}

export default App;
