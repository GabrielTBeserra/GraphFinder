import { useState } from 'react';
import { getUpdateStatus, Session, UpdateStatus } from './api';

function formatDate(value?: string | null) {
  if (!value) return '—';
  return new Intl.DateTimeFormat(undefined, { dateStyle: 'medium', timeStyle: 'short' }).format(new Date(value));
}

const SEVERITY_LABELS: Record<string, string> = {
  security: 'Security update',
  major: 'Major release',
  minor: 'Minor release',
  patch: 'Patch release',
};

export function UpdatesPanel({ session, status, onStatus }: {
  session: Session;
  status: UpdateStatus | null;
  onStatus: (status: UpdateStatus) => void;
}) {
  const [checking, setChecking] = useState(false);
  const [error, setError] = useState('');

  async function recheck() {
    setChecking(true);
    setError('');
    try {
      onStatus(await getUpdateStatus(session.token, true));
    } catch {
      setError('update check failed — control plane unreachable');
    } finally {
      setChecking(false);
    }
  }

  if (!status) {
    return <div className="empty-state large">Contacting release channel…</div>;
  }

  const severityLabel = status.severity ? SEVERITY_LABELS[status.severity] ?? status.severity : null;

  return (
    <section className="updates-workspace">
      <div className="panel-heading">
        <h2>Release center</h2>
        <span>channel {status.channel}</span>
      </div>

      <div className="update-version-grid">
        <article className="update-version-card">
          <span>Current deployment</span>
          <strong>v{status.current_version}</strong>
          <em>graphfinder-server</em>
        </article>
        <article className={`update-version-card ${status.has_update ? 'accent' : ''}`}>
          <span>Latest available</span>
          <strong>{status.latest_version ? `v${status.latest_version}` : '—'}</strong>
          <em>{status.published_at ? formatDate(status.published_at) : 'unpublished'}</em>
        </article>
        <article className="update-version-card">
          <span>Status</span>
          <strong className={status.has_update ? 'update-strong' : ''}>
            {status.check_failed ? 'check failed' : status.has_update ? 'update available' : 'up to date'}
          </strong>
          <em>checked {formatDate(status.checked_at)}</em>
        </article>
      </div>

      {status.has_update && (
        <div className={`update-banner-inline ${status.severity === 'security' ? 'danger' : ''}`}>
          <div>
            <strong>{severityLabel ?? 'Update available'} — {status.tag_name}</strong>
            <p>
              {status.severity === 'security'
                ? 'This release patches a security issue. Plan the upgrade as soon as possible.'
                : 'A new GraphFinder release is ready. Review the notes before upgrading the deployment.'}
            </p>
          </div>
          {status.release_url && (
            <a href={status.release_url} target="_blank" rel="noopener noreferrer">Open release</a>
          )}
        </div>
      )}

      {status.check_failed && (
        <p className="operation-message">github release check failed — showing last known data</p>
      )}
      {error && <p className="operation-message">{error}</p>}

      <div className="update-actions">
        <button onClick={recheck} disabled={checking}>
          {checking ? 'Checking…' : 'Check again'}
        </button>
        {status.release_url && (
          <a className="update-link" href={status.release_url} target="_blank" rel="noopener noreferrer">
            All releases on GitHub
          </a>
        )}
      </div>

      {status.release_notes && (
        <article className="release-notes">
          <div className="panel-heading compact-heading">
            <h3>{status.name ?? status.tag_name} — release notes</h3>
            <span>{formatDate(status.published_at)}</span>
          </div>
          <pre>{status.release_notes}</pre>
        </article>
      )}
    </section>
  );
}
