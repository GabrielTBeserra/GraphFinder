import React, { Suspense, useState, useEffect, useCallback, useRef } from 'react';
import { listen } from '@tauri-apps/api/event';
import {
  fetchGraph,
  fetchNotes,
  fetchNote,
  saveNote,
  deleteNote,
  searchChunks,
  searchCode,
  listCodeFiles,
  syncVault,
  fetchProjects,
  switchProject,
  createProject,
  fetchEngineStatus,
  markClosePromptReady,
  resolveClosePrompt,
  type CloseBehavior,
} from './services/api';
import type {
  ChunkHit,
  CodeChunkHit,
  CodeFileRecord,
  GraphData,
  NoteRecord,
  NoteSummary,
  ProjectInfo,
  SyncProgress,
} from './types';
const GraphView = React.lazy(() =>
  import('./components/GraphView').then((m) => ({ default: m.GraphView }))
);
import { Sidebar } from './components/Sidebar';
import { OmnisearchModal } from './components/OmnisearchModal';
import { ConfirmDialog } from './components/ConfirmDialog';
import { ClosePromptDialog } from './components/ClosePromptDialog';
import { ActivityBar } from './components/ActivityBar';
import { DocTabs, type DocTab } from './components/DocTabs';
import { BottomPanel, StatusBar } from './components/StudioPanels';

const NoteEditorLazy = React.lazy(() =>
  import('./components/NoteEditor').then((m) => ({ default: m.NoteEditor }))
);
const CodeInspectorLazy = React.lazy(() =>
  import('./components/CodeInspector').then((m) => ({ default: m.CodeInspector }))
);
const McpModal = React.lazy(() =>
  import('./components/McpModal').then((m) => ({ default: m.McpModal }))
);
const NewProjectModal = React.lazy(() =>
  import('./components/NewProjectModal').then((m) => ({ default: m.NewProjectModal }))
);
const ReleaseModal = React.lazy(() =>
  import('./components/ReleaseModal').then((m) => ({ default: m.ReleaseModal }))
);
const SettingsModal = React.lazy(() =>
  import('./components/SettingsModal').then((m) => ({ default: m.SettingsModal }))
);
import { checkForUpdates, type ReleaseInfo } from './services/updater';
import {
  CheckCircle,
  WarningCircle,
} from '@phosphor-icons/react';

interface Toast {
  id: string;
  message: string;
  type?: 'success' | 'info' | 'error';
}

const formatSyncEta = (seconds: number) =>
  seconds >= 60
    ? `${Math.floor(seconds / 60)}m ${String(seconds % 60).padStart(2, '0')}s`
    : `${seconds}s`;

const getSyncStatus = (progress: SyncProgress) => {
  switch (progress.phase) {
    case 'scanning':
      return `Scanning files · ${progress.completed} found`;
    case 'indexing':
      return `Indexing ${progress.completed}/${progress.total} · ${progress.percent ?? 0}%`;
    case 'finalizing':
      return 'Finalizing index · 99%';
    case 'complete':
      return 'Synchronization complete · 100%';
  }
};

export const App: React.FC = () => {
  const [graphData, setGraphData] = useState<GraphData>({ nodes: [], links: [] });
  const [notes, setNotes] = useState<NoteSummary[]>([]);
  const [codeFiles, setCodeFiles] = useState<CodeFileRecord[]>([]);
  const [selectedNoteId, setSelectedNoteId] = useState<string | null>(null);
  const [activeNote, setActiveNote] = useState<NoteRecord | null>(null);
  const [selectedCodePath, setSelectedCodePath] = useState<string | null>(null);
  const [selectedCodeSymbol, setSelectedCodeSymbol] = useState<string | null>(null);
  const [isCodeInspectorOpen, setIsCodeInspectorOpen] = useState(false);
  const [chunkResults, setChunkResults] = useState<ChunkHit[]>([]);
  const [codeChunkResults, setCodeChunkResults] = useState<CodeChunkHit[]>([]);
  const [isSearching, setIsSearching] = useState(false);
  const [isSyncing, setIsSyncing] = useState(false);
  const [syncProgress, setSyncProgress] = useState<SyncProgress | null>(null);
  const [isEditorOpen, setIsEditorOpen] = useState(false);
  const [isOmnisearchOpen, setIsOmnisearchOpen] = useState(false);
  const [isMcpModalOpen, setIsMcpModalOpen] = useState(false);
  const [isNewProjectModalOpen, setIsNewProjectModalOpen] = useState(false);
  const [pendingDelete, setPendingDelete] = useState<{ id: string; title: string } | null>(null);
  const [openTabs, setOpenTabs] = useState<DocTab[]>([]);
  const [panelOpen, setPanelOpen] = useState(false);
  const [isSidebarOpen, setIsSidebarOpen] = useState<boolean>(() => {
    try {
      return localStorage.getItem('gf_sidebar') !== '0';
    } catch {
      return true;
    }
  });
  const [projects, setProjects] = useState<ProjectInfo[]>([]);
  const [activeProject, setActiveProject] = useState<ProjectInfo | null>(null);
  const loadGenRef = useRef<number>(0);
  const [toasts, setToasts] = useState<Toast[]>([]);
  const [glowingNode, setGlowingNode] = useState<{ id: string; timestamp: number } | null>(null);
  const [engineReady, setEngineReady] = useState<boolean>(false);
  const [engineStatusMessage, setEngineStatusMessage] = useState<string>(
    'Initializing neural embedding engine...'
  );
  const [engineError, setEngineError] = useState<boolean>(false);
  const [releaseInfo, setReleaseInfo] = useState<ReleaseInfo | null>(null);
  const [isReleaseModalOpen, setIsReleaseModalOpen] = useState(false);
  const [isSettingsModalOpen, setIsSettingsModalOpen] = useState(false);
  const [isClosePromptOpen, setIsClosePromptOpen] = useState(false);
  const [theme, setTheme] = useState<'dark' | 'light'>(() => {
    try {
      const saved = localStorage.getItem('gf_theme');
      return saved === 'light' ? 'light' : 'dark';
    } catch {
      return 'dark';
    }
  });

  const toggleTheme = useCallback(() => {
    setTheme((prev) => {
      const next = prev === 'dark' ? 'light' : 'dark';
      try {
        localStorage.setItem('gf_theme', next);
      } catch {
        // ignore
      }
      return next;
    });
  }, []);

  useEffect(() => {
    try {
      localStorage.setItem('gf_sidebar', isSidebarOpen ? '1' : '0');
    } catch {
      // ignore
    }
  }, [isSidebarOpen]);

  useEffect(() => {
    if (theme === 'light') {
      document.documentElement.classList.add('light');
      document.documentElement.classList.remove('dark');
      document.documentElement.setAttribute('data-theme', 'light');
    } else {
      document.documentElement.classList.add('dark');
      document.documentElement.classList.remove('light');
      document.documentElement.setAttribute('data-theme', 'dark');
    }
  }, [theme]);

  const showToast = useCallback(
    (message: string, type: 'success' | 'info' | 'error' = 'success') => {
      const id = Math.random().toString(36).substring(2, 9);
      setToasts((prev) => [...prev, { id, message, type }]);
      setTimeout(() => {
        setToasts((prev) => prev.filter((t) => t.id !== id));
      }, 3200);
    },
    []
  );

  useEffect(() => {
    checkForUpdates()
      .then((info) => setReleaseInfo(info))
      .catch((err) => {
        console.warn('[Updater] Initial check error:', err);
      });
  }, []);

  const handleSelectNote = useCallback(async (idOrTitle: string) => {
    const clean = idOrTitle.replace(/^unresolved:/, '').split('#')[0].trim();
    setSelectedNoteId(clean);
    setIsEditorOpen(true);
    // Mutually exclusive doc panels: a note tab replaces the code inspector
    // (otherwise activeTabKey stays pinned on `code:` and tabs look dead).
    setSelectedCodePath(null);
    setSelectedCodeSymbol(null);
    setIsCodeInspectorOpen(false);
    const provisionalKey = `note:${clean}`;
    setOpenTabs((prev) => {
      if (prev.some((t) => t.key === provisionalKey)) return prev;
      return [...prev, { key: provisionalKey, type: 'note' as const, label: clean.replace(/\.md$/, '') }].slice(-8);
    });
    // Once the backend resolves the real note id, canonicalize the tab key so
    // the same document can't exist under two different keys (title vs id).
    const canonicalize = (noteId: string, label: string) => {
      const canonKey = `note:${noteId}`;
      setOpenTabs((prev) => {
        if (canonKey === provisionalKey) return prev;
        if (prev.some((t) => t.key === canonKey)) {
          return prev.filter((t) => t.key !== provisionalKey);
        }
        return prev.map((t) =>
          t.key === provisionalKey ? { key: canonKey, type: 'note' as const, label } : t
        );
      });
    };
    try {
      const note = await fetchNote(clean);
      if (note) {
        setActiveNote(note);
        setSelectedNoteId(note.id);
        canonicalize(note.id, (note.title || note.id).replace(/\.md$/, ''));
        try {
          localStorage.setItem('gf_last_note', note.id);
        } catch {
          // ignore
        }
        return;
      }
      // If not found directly by id, search in notes list
      const found = notes.find(
        (n) =>
          n.id === clean ||
          n.id.replace(/\.md$/, '') === clean ||
          n.title.toLowerCase() === clean.toLowerCase()
      );
      if (found) {
        const foundNote = await fetchNote(found.id);
        if (foundNote) {
          setActiveNote(foundNote);
          setSelectedNoteId(foundNote.id);
          canonicalize(foundNote.id, (foundNote.title || foundNote.id).replace(/\.md$/, ''));
          return;
        }
      }
      // Unresolved note from graph: create draft. Keep selectedNoteId on the
      // provisional key so the tab stays highlighted while editing.
      setSelectedNoteId(clean);
      setActiveNote({
        id: '',
        path: clean.endsWith('.md') ? clean : `${clean}.md`,
        title: clean,
        summary: '',
        content: `# ${clean}\n\nConnection created from knowledge graph.`,
        updated_at: Math.floor(Date.now() / 1000),
        tags: [],
        outgoing_links: [],
        backlinks: [],
      });
    } catch (err) {
      console.error('Error fetching note:', err);
    }
  }, [notes]);

  const handleSelectCodeFile = useCallback((path: string, symbol?: string) => {
    setSelectedCodePath(path);
    setSelectedNoteId(path);
    // Mutually exclusive doc panels: a code tab replaces the note editor.
    setIsEditorOpen(false);
    setOpenTabs((prev) => {
      const key = `code:${path}`;
      if (prev.some((t) => t.key === key)) return prev;
      const label = path.split('/').pop() || path;
      return [...prev, { key, type: 'code' as const, label }].slice(-8);
    });
    setSelectedCodeSymbol(symbol || null);
    setIsCodeInspectorOpen(true);
  }, []);

  const handleRevealNode = useCallback((id: string) => {
    setGlowingNode({ id, timestamp: Date.now() });
  }, []);

  const handleClearSelection = useCallback(() => {
    setSelectedNoteId(null);
    setSelectedCodePath(null);
    setSelectedCodeSymbol(null);
  }, []);

  const activeTabKey = selectedCodePath
    ? `code:${selectedCodePath}`
    : selectedNoteId && isEditorOpen
      ? `note:${selectedNoteId}`
      : null;

  const handleSelectTab = useCallback(
    (tab: DocTab) => {
      const id = tab.key.slice(5);
      if (tab.type === 'note') handleSelectNote(id);
      else handleSelectCodeFile(id);
    },
    [handleSelectNote, handleSelectCodeFile]
  );

  const handleCloseTab = useCallback(
    (tab: DocTab) => {
      setOpenTabs((prev) => prev.filter((t) => t.key !== tab.key));
      if (tab.key === activeTabKey) {
        setIsEditorOpen(false);
        setIsCodeInspectorOpen(false);
        handleClearSelection();
      }
    },
    [activeTabKey, handleClearSelection]
  );

  const handleShowGraphTab = useCallback(() => {
    setIsEditorOpen(false);
    setIsCodeInspectorOpen(false);
    handleClearSelection();
  }, [handleClearSelection]);

  useEffect(() => {
    if (chunkResults.length > 0 || codeChunkResults.length > 0) setPanelOpen(true);
  }, [chunkResults.length, codeChunkResults.length]);

  const handleNavigateNote = useCallback((titleOrId: string) => {
    handleSelectNote(titleOrId);
  }, [handleSelectNote]);

  const loadProjects = useCallback(async () => {
    try {
      const list = await fetchProjects();
      setProjects(list);
      const active = list.find((p) => p.is_active) || list[0] || null;
      setActiveProject(active);
    } catch (err) {
      console.error('Error listing projects:', err);
    }
  }, []);

  const loadData = useCallback(async () => {
    const gen = ++loadGenRef.current;
    try {
      const [gRes, nRes, cRes] = await Promise.allSettled([
        fetchGraph(),
        fetchNotes(),
        listCodeFiles(),
      ]);

      if (gen !== loadGenRef.current) return;

      if (gRes.status === 'fulfilled') setGraphData(gRes.value);
      if (nRes.status === 'fulfilled') setNotes(nRes.value);
      if (cRes.status === 'fulfilled') setCodeFiles(cRes.value);
    } catch (err) {
      console.error('Error loading GraphFinder data:', err);
    }
  }, []);

  useEffect(() => {
    let isCancelled = false;

    async function initApp() {
      try {
        // Check engine status asynchronously without blocking UI data fetch
        fetchEngineStatus()
          .then((engineInfo) => {
            if (!isCancelled && engineInfo.is_ready) {
              setEngineReady(true);
            }
          })
          .catch(() => {});

        // 1. Fetch projects in parallel and update active project as soon as ready
        const projectsPromise = fetchProjects()
          .then((pList) => {
            if (!isCancelled) {
              setProjects(pList);
              const active = pList.find((p) => p.is_active) || pList[0] || null;
              setActiveProject(active);
            }
            return pList;
          })
          .catch((e) => {
            console.warn('fetchProjects failed:', e);
            return [];
          });

        // 2. Fetch notes in parallel and set initial active note
        const notesPromise = fetchNotes()
          .then(async (nData) => {
            if (!isCancelled) {
              setNotes(nData);
              if (nData.length > 0) {
                let initialId = nData[0].id;
                try {
                  const last = localStorage.getItem('gf_last_note');
                  if (last && nData.some((n) => n.id === last)) initialId = last;
                } catch {
                  // ignore
                }
                setSelectedNoteId(initialId);
                setIsEditorOpen(false);
                try {
                  const firstNote = await fetchNote(initialId);
                  if (!isCancelled && firstNote) {
                    setActiveNote(firstNote);
                  }
                } catch (e) {
                  console.warn('fetchNote failed:', e);
                }
              }
            }
            return nData;
          })
          .catch((e) => {
            console.warn('fetchNotes failed:', e);
            return [];
          });

        // 3. Fetch graph topology in parallel
        const graphPromise = fetchGraph()
          .then((gData) => {
            if (!isCancelled) {
              setGraphData(gData);
            }
            return gData;
          })
          .catch((e) => {
            console.warn('fetchGraph failed:', e);
            return { nodes: [], links: [] };
          });

        // 4. Fetch code files in parallel
        const codeFilesPromise = listCodeFiles()
          .then((cFiles) => {
            if (!isCancelled) {
              setCodeFiles(cFiles);
            }
            return cFiles;
          })
          .catch((e) => {
            console.warn('listCodeFiles failed:', e);
            return [];
          });

        await Promise.allSettled([
          projectsPromise,
          notesPromise,
          graphPromise,
          codeFilesPromise,
        ]);
      } catch (err) {
        console.error('Initialization error:', err);
      }
    }

    initApp();

    // Fallback polling for engine readiness in case event listener missed the event
    const statusInterval = setInterval(async () => {
      if (isCancelled) return;
      try {
        const info = await fetchEngineStatus();
        if (info.is_ready) {
          setEngineReady(true);
          setEngineError(false);
          clearInterval(statusInterval);
        }
      } catch {
        // ignore
      }
    }, 600);

    // Hard fallback timeout: always show UI after 4 seconds even if engine is still warming up
    const fallbackTimeout = setTimeout(() => {
      if (!isCancelled) {
        setEngineReady(true);
      }
    }, 4000);

    let unlistenSync: (() => void) | undefined;
    let unlistenSyncProgress: (() => void) | undefined;
    let unlistenStructure: (() => void) | undefined;
    let unlistenMcp: (() => void) | undefined;
    let unlistenEngine: (() => void) | undefined;
    let syncCompleteTimeout: ReturnType<typeof setTimeout> | undefined;

    if (typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) {
      listen<SyncProgress>('vault-sync-progress', (event) => {
        const progress = event.payload;
        setSyncProgress(progress);
        if (progress.phase === 'complete') {
          if (syncCompleteTimeout) clearTimeout(syncCompleteTimeout);
          syncCompleteTimeout = setTimeout(() => {
            setSyncProgress((current) => (current?.phase === 'complete' ? null : current));
          }, 1600);
        }
      }).then((unlisten) => {
        unlistenSyncProgress = unlisten;
      });

      // Structural stage finished (AST/imports/links saved): graph and file
      // lists are queryable now even though embeddings are still computing.
      listen<number>('vault-structure-ready', () => {
        loadData();
      }).then((unlisten) => {
        unlistenStructure = unlisten;
      });

      listen<number>('vault-sync-complete', (event) => {
        loadData();
        loadProjects();
        if (event.payload > 0) {
          showToast(`${event.payload} files synchronized in background`, 'info');
        }
      }).then((unlisten) => {
        unlistenSync = unlisten;
      });

      listen<string>('mcp-node-updated', (event) => {
        const nodeId = event.payload;
        setGlowingNode({ id: nodeId, timestamp: Date.now() });
        loadData();
        showToast(`Vault updated via MCP: [[${nodeId}]]`, 'info');
      }).then((unlisten) => {
        unlistenMcp = unlisten;
      });

      listen<{ status: string; message: string }>('engine-status', (event) => {
        const p = event.payload;
        if (p.status === 'ready') {
          setEngineReady(true);
          setEngineError(false);
          loadData();
        } else if (p.status === 'loading') {
          setEngineStatusMessage(p.message || 'Loading embedding model...');
        } else if (p.status === 'error') {
          setEngineError(true);
          setEngineStatusMessage(p.message || 'Error initializing neural model.');
        }
      }).then((unlisten) => {
        unlistenEngine = unlisten;
      });
    }

    return () => {
      isCancelled = true;
      clearInterval(statusInterval);
      clearTimeout(fallbackTimeout);
      if (syncCompleteTimeout) clearTimeout(syncCompleteTimeout);
      if (unlistenSync) unlistenSync();
      if (unlistenSyncProgress) unlistenSyncProgress();
      if (unlistenStructure) unlistenStructure();
      if (unlistenMcp) unlistenMcp();
      if (unlistenEngine) unlistenEngine();
    };
  }, []);

  // Global Keyboard Shortcuts
  useEffect(() => {
    const handleGlobalKeyDown = (e: KeyboardEvent) => {
      // Ctrl+K / Cmd+K -> Toggle Omnisearch
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k') {
        e.preventDefault();
        setIsOmnisearchOpen((prev) => !prev);
      }
      // Ctrl+B / Cmd+B -> Toggle Sidebar
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'b') {
        e.preventDefault();
        setIsSidebarOpen((prev) => !prev);
      }
    };
    window.addEventListener('keydown', handleGlobalKeyDown);
    return () => window.removeEventListener('keydown', handleGlobalKeyDown);
  }, []);

  // First-close prompt: the backend only asks (instead of quitting) once this
  // listener is registered and acknowledged via markClosePromptReady().
  useEffect(() => {
    if (!('__TAURI_INTERNALS__' in window)) return;
    let disposed = false;
    let unlisten: (() => void) | undefined;
    listen('close-requested', () => setIsClosePromptOpen(true)).then((fn) => {
      if (disposed) {
        fn();
        return;
      }
      unlisten = fn;
      markClosePromptReady().catch((err) =>
        console.error('Failed to enable close prompt:', err)
      );
    });
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);

  const handleClosePromptChoice = async (behavior: CloseBehavior) => {
    setIsClosePromptOpen(false);
    try {
      await resolveClosePrompt(behavior);
    } catch (err) {
      console.error('Failed to save close behavior:', err);
      showToast('Could not save the close preference', 'error');
    }
  };

  const handleNewNote = () => {
    setSelectedNoteId(null);
    setActiveNote(null);
    setIsEditorOpen(true);
  };

  const handleSaveNote = async (title: string, content: string, filename?: string) => {
    try {
      const saved = await saveNote(title, content, filename);
      setActiveNote(saved);
      setSelectedNoteId(saved.id);
      setGlowingNode({ id: saved.id, timestamp: Date.now() });
      showToast(`Note "${title}" saved successfully!`);
      await loadData();
      await loadProjects();
    } catch (err) {
      console.error('Error saving note:', err);
    }
  };

  const handleDeleteNote = (idOrTitle: string, title?: string) => {
    setPendingDelete({ id: idOrTitle, title: title || idOrTitle });
  };

  const confirmDeleteNote = async () => {
    if (!pendingDelete) return;
    const { id: idOrTitle, title } = pendingDelete;
    setPendingDelete(null);
    try {
      await deleteNote(idOrTitle);
      showToast(`Note "${title || idOrTitle}" deleted successfully`);
      if (selectedNoteId === idOrTitle || activeNote?.id === idOrTitle) {
        setIsEditorOpen(false);
        setActiveNote(null);
        setSelectedNoteId(null);
      }
      await loadData();
      await loadProjects();
    } catch (err) {
      console.error('Error deleting note:', err);
      showToast(`Error deleting note: ${err}`, 'info');
    }
  };

  const handleSwitchProject = async (projectId: string) => {
    try {
      loadGenRef.current += 1;
      const optimistic = projects.find((p) => p.id.toLowerCase() === projectId.toLowerCase());
      if (optimistic) {
        setActiveProject({ ...optimistic, is_active: true });
      }
      setSelectedNoteId(null);
      setActiveNote(null);
      setSelectedCodePath(null);
      setSelectedCodeSymbol(null);
      setIsEditorOpen(false);
      setIsCodeInspectorOpen(false);
      setChunkResults([]);
      setCodeChunkResults([]);
      setGraphData({ nodes: [], links: [] });
      setNotes([]);
      setCodeFiles([]);

      const project = await switchProject(projectId);
      setActiveProject(project);
      setProjects((prev) =>
        prev.map((p) => ({
          ...p,
          is_active: p.id.toLowerCase() === project.id.toLowerCase(),
        }))
      );
      showToast(`Active project: ${project.name}`);
      await Promise.allSettled([loadData(), loadProjects()]);
    } catch (err) {
      console.error('Error switching project:', err);
      showToast(`Error switching project: ${err}`, 'info');
    }
  };

  const handleCreateProject = async (name: string, path?: string) => {
    try {
      loadGenRef.current += 1;
      setSelectedNoteId(null);
      setActiveNote(null);
      setSelectedCodePath(null);
      setSelectedCodeSymbol(null);
      setIsEditorOpen(false);
      setIsCodeInspectorOpen(false);
      setChunkResults([]);
      setCodeChunkResults([]);
      setGraphData({ nodes: [], links: [] });
      setNotes([]);
      setCodeFiles([]);

      const project = await createProject(name, path);
      setActiveProject(project);
      showToast(`New project "${project.name}" created successfully!`);
      await Promise.allSettled([loadData(), loadProjects()]);
    } catch (err) {
      console.error('Error creating project:', err);
      throw err;
    }
  };

  const handleSearchSemantic = async (query: string) => {
    if (!query.trim()) return;
    setIsSearching(true);
    try {
      const chunks = await searchChunks(query, 8);
      setChunkResults(chunks);
    } catch (err) {
      console.error('Error during semantic search:', err);
    } finally {
      setIsSearching(false);
    }
  };

  const handleSearchCode = async (query: string) => {
    if (!query.trim()) return;
    setIsSearching(true);
    try {
      const chunks = await searchCode(query, undefined, 8);
      setCodeChunkResults(chunks);
    } catch (err) {
      console.error('Error during code semantic search:', err);
    } finally {
      setIsSearching(false);
    }
  };

  const handleSync = async () => {
    setIsSyncing(true);
    try {
      const count = await syncVault();
      await loadData();
      await loadProjects();
      showToast(`Sync complete (${count} files processed)`);
    } catch (err) {
      console.error('Error during sync:', err);
      setSyncProgress(null);
      showToast(`Sync failed: ${String(err)}`, 'error');
    } finally {
      setIsSyncing(false);
    }
  };

  // Non-blocking boot: UI renders immediately from local SQLite database
  return (
    <div className="flex flex-col w-screen h-dvh overflow-hidden select-none transition-colors bg-canvas text-ink">
      {syncProgress && (
        <div
          className="shrink-0 border-b border-line bg-surface px-4 py-2"
          role="status"
          aria-live="polite"
        >
          <div className="flex items-center gap-3">
            <span className="gf-eyebrow !text-accent-strong">Sync</span>
            <span
              className="min-w-0 flex-1 truncate text-[11px] font-mono text-muted"
              title={syncProgress.current_file ?? undefined}
            >
              {syncProgress.current_file
                ? `${getSyncStatus(syncProgress)} · ${syncProgress.current_file}`
                : getSyncStatus(syncProgress)}
            </span>
            {syncProgress.phase === 'indexing' && (
              <span className="hidden shrink-0 text-[11px] font-mono text-faint sm:inline tabular-nums">
                {syncProgress.eta_seconds === null
                  ? 'Estimating time'
                  : `~${formatSyncEta(syncProgress.eta_seconds)} left`}
              </span>
            )}
            <div
              className="h-1 w-20 shrink-0 overflow-hidden rounded-full bg-line-strong sm:w-28"
              role="progressbar"
              aria-label="Vault synchronization progress"
              aria-valuemin={0}
              aria-valuemax={100}
              aria-valuenow={syncProgress.percent ?? undefined}
              aria-valuetext={getSyncStatus(syncProgress)}
            >
              <div
                className={`h-full rounded-full bg-accent transition-[width] duration-300 ${
                  syncProgress.percent === null ? 'w-1/3 animate-pulse' : ''
                }`}
                style={
                  syncProgress.percent === null
                    ? undefined
                    : { width: `${syncProgress.percent}%` }
                }
              />
            </div>
          </div>
        </div>
      )}

      {/* Main Studio Body */}
      <div className="flex-1 flex overflow-hidden relative bg-surface">
        <ActivityBar
          sidebarOpen={isSidebarOpen}
          isSyncing={isSyncing}
          hasUpdate={releaseInfo?.hasUpdate}
          theme={theme}
          onToggleTheme={toggleTheme}
          onToggleSidebar={() => setIsSidebarOpen(!isSidebarOpen)}
          onSearch={() => setIsOmnisearchOpen(true)}
          onNewNote={handleNewNote}
          onSync={handleSync}
          onOpenMcp={() => setIsMcpModalOpen(true)}
          onOpenReleases={() => setIsReleaseModalOpen(true)}
          onOpenSettings={() => setIsSettingsModalOpen(true)}
        />
        <Sidebar
          projectId={activeProject?.id || 'default'}
          projects={projects}
          activeProject={activeProject}
          onSwitchProject={handleSwitchProject}
          onOpenNewProjectModal={() => setIsNewProjectModalOpen(true)}
          notes={notes}
          codeFiles={codeFiles}
          selectedNoteId={selectedNoteId}
          selectedCodePath={selectedCodePath}
          onSelectNote={handleSelectNote}
          onSelectCodeFile={handleSelectCodeFile}
          onNewNote={handleNewNote}
          onDeleteNote={handleDeleteNote}
          onSearchSemantic={handleSearchSemantic}
          onSearchCode={handleSearchCode}
          onRevealInGraph={(id) => setGlowingNode({ id, timestamp: Date.now() })}
          chunkResults={[]}
          codeChunkResults={[]}
          isSearching={isSearching}
          isCollapsed={!isSidebarOpen}
          theme={theme}
          appVersion={releaseInfo?.currentVersion || '0.15.0'}
          hasUpdate={releaseInfo?.hasUpdate}
          onOpenReleases={() => setIsReleaseModalOpen(true)}
        />

        <div className="flex-1 flex flex-col min-w-0">
          <DocTabs
            tabs={openTabs}
            activeKey={activeTabKey}
            onSelect={handleSelectTab}
            onClose={handleCloseTab}
            onShowGraph={handleShowGraphTab}
          />
        <main className="flex-1 relative overflow-hidden bg-canvas">
          <Suspense
            fallback={
              <div className="absolute inset-0 flex items-center justify-center">
                <span className="gf-eyebrow">Loading graph…</span>
              </div>
            }
          >
          <GraphView
            projectId={activeProject?.id || 'default'}
            graphData={graphData}
            selectedNodeId={selectedNoteId}
            onSelectNode={handleSelectNote}
            onSelectCode={handleSelectCodeFile}
            onClearSelection={handleClearSelection}
            onRefresh={loadData}
            glowingNode={glowingNode}
            theme={theme}
          />
          </Suspense>

          {isEditorOpen && (
            <Suspense fallback={null}>
            <NoteEditorLazy
              note={activeNote}
              onSave={handleSaveNote}
              onDelete={handleDeleteNote}
              onClose={() => setIsEditorOpen(false)}
              onNavigateNote={handleNavigateNote}
              availableNotes={notes}
              theme={theme}
            />
            </Suspense>
          )}

          {isCodeInspectorOpen && selectedCodePath && (
            <Suspense fallback={null}>
            <CodeInspectorLazy
              fileId={selectedCodePath}
              initialSymbol={selectedCodeSymbol}
              onClose={() => setIsCodeInspectorOpen(false)}
              onNavigateFile={(fileId) => {
                setSelectedCodePath(fileId);
                setSelectedCodeSymbol(null);
              }}
              theme={theme}
            />
            </Suspense>
          )}
        </main>
        <BottomPanel
          open={panelOpen}
          chunkResults={chunkResults}
          codeChunkResults={codeChunkResults}
          isSearching={isSearching}
          onSelectNote={handleSelectNote}
          onSelectCode={handleSelectCodeFile}
          onClose={() => setPanelOpen(false)}
        />
        </div>
      </div>

      <StatusBar
        project={activeProject}
        noteCount={notes.length}
        fileCount={codeFiles.length}
        linkCount={graphData.links.length}
        engineReady={engineReady}
        engineError={engineError}
        engineStatusMessage={engineStatusMessage}
        syncProgress={syncProgress}
        onOpenMcp={() => setIsMcpModalOpen(true)}
      />

      {/* Dynamic Modals */}
      <OmnisearchModal
        isOpen={isOmnisearchOpen}
        onClose={() => setIsOmnisearchOpen(false)}
        notes={notes}
        codeFiles={codeFiles}
        onSelectNote={handleSelectNote}
        onSelectCodeFile={handleSelectCodeFile}
        onRevealNode={handleRevealNode}
        onNewNote={handleNewNote}
        onSync={handleSync}
        onToggleTheme={toggleTheme}
        theme={theme}
      />
      <Suspense fallback={null}>
        {isMcpModalOpen && (
          <McpModal
            isOpen={isMcpModalOpen}
            onClose={() => setIsMcpModalOpen(false)}
            theme={theme}
          />
        )}
        {isNewProjectModalOpen && (
          <NewProjectModal
            isOpen={isNewProjectModalOpen}
            onClose={() => setIsNewProjectModalOpen(false)}
            onCreate={handleCreateProject}
          />
        )}
        {isReleaseModalOpen && (
          <ReleaseModal
            isOpen={isReleaseModalOpen}
            onClose={() => setIsReleaseModalOpen(false)}
            releaseInfo={releaseInfo}
            onUpdateReleaseInfo={(info) => setReleaseInfo(info)}
            theme={theme}
          />
        )}
        {isSettingsModalOpen && (
          <SettingsModal
            isOpen={isSettingsModalOpen}
            onClose={() => setIsSettingsModalOpen(false)}
          />
        )}
      </Suspense>

      <ConfirmDialog
        open={pendingDelete !== null}
        title="Delete note?"
        description={`"${pendingDelete?.title ?? ''}" will be permanently removed. This cannot be undone.`}
        onConfirm={confirmDeleteNote}
        onCancel={() => setPendingDelete(null)}
      />

      <ClosePromptDialog
        open={isClosePromptOpen}
        onChoose={handleClosePromptChoice}
        onCancel={() => setIsClosePromptOpen(false)}
      />

      {/* Toast Notifications */}
      {toasts.length > 0 && (
        <div className="fixed bottom-4 right-4 z-50 flex flex-col gap-2 pointer-events-none">
          {toasts.map((toast) => (
            <div
              key={toast.id}
              className="flex items-center gap-2 px-3 py-2 rounded-md border border-line bg-surface text-ink text-xs font-medium shadow-2xl pointer-events-auto animate-in fade-in slide-in-from-bottom-2 duration-200"
            >
              {toast.type === 'error' ? (
                <WarningCircle size={14} weight="fill" className="shrink-0 text-bad" />
              ) : (
                <CheckCircle size={14} weight="fill" className="shrink-0 text-good" />
              )}
              <span>{toast.message}</span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};

export default App;
