import { useCallback, useEffect, useState } from 'react';
import {
  getProjectGraph,
  Project,
  ProjectGraph,
  ProjectGraphNode,
  Session,
} from './api';
import { GraphCanvas } from './GraphCanvas';

export function GraphViewer({ session, projects }: { session: Session; projects: Project[] }) {
  const [projectId, setProjectId] = useState('');
  const [graph, setGraph] = useState<ProjectGraph | null>(null);
  const [selected, setSelected] = useState<ProjectGraphNode | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');

  const activeProject = projects.find((p) => p.id === projectId) ?? projects[0];

  useEffect(() => {
    if (!activeProject) return;
    let cancelled = false;
    setLoading(true);
    setError('');
    setSelected(null);
    getProjectGraph(session.token, activeProject.id)
      .then((data) => {
        if (!cancelled) setGraph(data);
      })
      .catch(() => {
        if (!cancelled) {
          setGraph(null);
          setError('graph unavailable — the project may not have a published generation yet');
        }
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [session.token, activeProject?.id]);

  const handleSelect = useCallback((node: ProjectGraphNode | null) => {
    setSelected(node);
  }, []);

  if (projects.length === 0) {
    return (
      <div className="empty-state large">
        No projects yet — create a project and publish a generation to explore its knowledge graph.
      </div>
    );
  }

  return (
    <section className="graph-workspace">
      <div className="graph-toolbar">
        <div className="panel-heading compact-heading">
          <h2>Project knowledge graph</h2>
          {graph && (
            <span>
              {graph.nodes.length} nodes · {graph.edges.length} edges · {graph.community_count} communities · gen {graph.generation} · {graph.edge_source}
              {graph.truncated ? ` · top of ${graph.total_documents}` : ''}
            </span>
          )}
        </div>
        <div className="graph-toolbar-actions">
          {graph && (
            <button
              className="graph-export"
              onClick={() => {
                const payload = JSON.stringify(
                  {
                    project: activeProject?.slug,
                    generation: graph.generation,
                    exported_at: new Date().toISOString(),
                    nodes: graph.nodes,
                    edges: graph.edges,
                    analytics: graph.analytics,
                  },
                  null,
                  2,
                );
                const url = URL.createObjectURL(new Blob([payload], { type: 'application/json' }));
                const anchor = document.createElement('a');
                anchor.href = url;
                anchor.download = `graph-${activeProject?.slug ?? 'project'}-gen${graph.generation}.json`;
                anchor.click();
                URL.revokeObjectURL(url);
              }}
            >
              Export JSON
            </button>
          )}
          <select value={activeProject?.id ?? ''} onChange={(event) => setProjectId(event.target.value)}>
            {projects.map((project) => (
              <option key={project.id} value={project.id}>
                {project.name}
              </option>
            ))}
          </select>
        </div>
      </div>

      {error && <p className="operation-message">{error}</p>}
      {loading && <div className="empty-state large">Reconstructing document topology…</div>}

      {!loading && graph && graph.nodes.length === 0 && (
        <div className="overview-empty">
          <span className="overview-empty-mark">GF</span>
          <h2>No graphable documents</h2>
          <p>This generation has no documents with wikilinks or imports yet. Index markdown notes or source code to grow the graph.</p>
        </div>
      )}

      {!loading && graph && graph.nodes.length > 0 && (
        <div className="graph-layout">
          <GraphCanvas nodes={graph.nodes} edges={graph.edges} selectedId={selected?.id} onSelectNode={handleSelect} />
          <aside className="graph-detail">
            {selected ? (
              <>
                <h3>{selected.title ?? selected.path.split('/').pop()}</h3>
                <dl>
                  <div><dt>Path</dt><dd>{selected.path}</dd></div>
                  <div><dt>Type</dt><dd>{selected.document_type}{selected.language ? ` · ${selected.language}` : ''}</dd></div>
                  <div><dt>Connections</dt><dd>{selected.degree}</dd></div>
                  {selected.is_god_node && (
                    <div><dt>Role</dt><dd className="god-badge">Architectural hub</dd></div>
                  )}
                  <div><dt>Community</dt><dd>cluster {selected.community + 1}</dd></div>
                  <div><dt>Size</dt><dd>{(selected.size_bytes / 1024).toFixed(1)} kb</dd></div>
                  {selected.tags.length > 0 && (
                    <div><dt>Tags</dt><dd>{selected.tags.map((tag) => `#${tag}`).join(' ')}</dd></div>
                  )}
                </dl>
                <button className="graph-clear" onClick={() => setSelected(null)}>Clear selection</button>
              </>
            ) : (
              <div className="graph-detail-empty">
                <h3>Topology inspector</h3>
                <p>Click a node to inspect its document, connections and tags. Drag to rearrange, scroll to zoom, drag the background to pan.</p>
                <div className="graph-stats">
                  <div><span>Documents</span><strong>{graph.total_documents}</strong></div>
                  <div><span>Resolved edges</span><strong>{graph.total_edges}</strong></div>
                  <div><span>Communities</span><strong>{graph.community_count}</strong></div>
                  <div><span>Cross-community coupling</span><strong className={graph.analytics.cross_community_edges > 0 ? 'warn-text' : ''}>{graph.analytics.cross_community_edges}</strong></div>
                  <div><span>Dependency cycles</span><strong className={graph.analytics.cycles.length > 0 ? 'danger-text' : ''}>{graph.analytics.cycles.length}</strong></div>
                  <div><span>Isolated documents</span><strong>{graph.analytics.isolated_count}</strong></div>
                </div>
                {graph.analytics.god_nodes.length > 0 && (
                  <div className="god-node-list">
                    <h4>Architectural hubs</h4>
                    {graph.analytics.god_nodes.slice(0, 5).map((god) => (
                      <button
                        key={god.id}
                        className="god-node-row"
                        onClick={() => {
                          const node = graph.nodes.find((candidate) => candidate.id === god.id);
                          if (node) setSelected(node);
                        }}
                      >
                        <strong>{god.title ?? god.path.split('/').pop()}</strong>
                        <span>{god.role}</span>
                        <em>{god.degree} connections</em>
                      </button>
                    ))}
                  </div>
                )}
                {graph.analytics.cycles.length > 0 && (
                  <div className="cycle-list">
                    <h4>Dependency cycles</h4>
                    {graph.analytics.cycles.slice(0, 4).map((cycle, index) => (
                      <div className="cycle-row" key={index} title={cycle.nodes.join(' → ')}>
                        {cycle.nodes.map((path) => path.split('/').pop()).join(' ↔ ')}
                      </div>
                    ))}
                  </div>
                )}
              </div>
            )}
          </aside>
        </div>
      )}
    </section>
  );
}
