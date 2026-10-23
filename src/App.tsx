import React, { useState, useEffect, useCallback } from 'react';
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
} from './services/api';
import type {
  ChunkHit,
  CodeChunkHit,
  CodeFileRecord,
  GraphData,
  NoteRecord,
  NoteSummary,
  ProjectInfo,
} from './types';
import { GraphView } from './components/GraphView';
import { NoteEditor } from './components/NoteEditor';
import { CodeInspector } from './components/CodeInspector';
import { Sidebar } from './components/Sidebar';
import { McpModal } from './components/McpModal';
import { ProjectSwitcher } from './components/ProjectSwitcher';
import { NewProjectModal } from './components/NewProjectModal';
import { OmnisearchModal } from './components/OmnisearchModal';
import {
  ArrowClockwise,
  SidebarSimple,
  CheckCircle,
  Plus,
  MagnifyingGlass,
  Sun,
  Moon,
} from '@phosphor-icons/react';

interface Toast {
  id: string;
  message: string;
  type?: 'success' | 'info';
}

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
  const [isEditorOpen, setIsEditorOpen] = useState(false);
  const [isOmnisearchOpen, setIsOmnisearchOpen] = useState(false);
  const [isMcpModalOpen, setIsMcpModalOpen] = useState(false);
  const [isNewProjectModalOpen, setIsNewProjectModalOpen] = useState(false);
  const [isSidebarOpen, setIsSidebarOpen] = useState(true);
  const [projects, setProjects] = useState<ProjectInfo[]>([]);
  const [activeProject, setActiveProject] = useState<ProjectInfo | null>(null);
  const [toasts, setToasts] = useState<Toast[]>([]);
  const [glowingNode, setGlowingNode] = useState<{ id: string; timestamp: number } | null>(null);
  const [engineReady, setEngineReady] = useState<boolean>(false);
  const [engineStatusMessage, setEngineStatusMessage] = useState<string>(
    'Initializing neural embedding engine...'
  );
  const [engineError, setEngineError] = useState<boolean>(false);
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

  const showToast = useCallback((message: string, type: 'success' | 'info' = 'success') => {
    const id = Math.random().toString(36).substring(2, 9);
    setToasts((prev) => [...prev, { id, message, type }]);
    setTimeout(() => {
      setToasts((prev) => prev.filter((t) => t.id !== id));
    }, 3200);
  }, []);

  const handleSelectNote = useCallback(async (idOrTitle: string) => {
    const clean = idOrTitle.replace(/^unresolved:/, '').split('#')[0].trim();
    setSelectedNoteId(clean);
    setIsEditorOpen(true);
    try {
      const note = await fetchNote(clean);
      if (note) {
        setActiveNote(note);
        setSelectedNoteId(note.id);
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
          return;
        }
      }
      // Unresolved note from graph: create draft
      setSelectedNoteId(null);
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
    setSelectedCodeSymbol(symbol || null);
    setIsCodeInspectorOpen(true);
  }, []);

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
    try {
      let gData: GraphData = { nodes: [], links: [] };
      let nData: NoteSummary[] = [];
      let cFiles: CodeFileRecord[] = [];

      try { gData = await fetchGraph(); } catch (e) { console.warn('fetchGraph failed in loadData:', e); }
      try { nData = await fetchNotes(); } catch (e) { console.warn('fetchNotes failed in loadData:', e); }
      try { cFiles = await listCodeFiles(); } catch (e) { console.warn('listCodeFiles failed in loadData:', e); }

      setGraphData(gData);
      setNotes(nData);
      setCodeFiles(cFiles);
    } catch (err) {
      console.error('Error loading GraphFinder data:', err);
    }
  }, []);

  useEffect(() => {
    let isCancelled = false;

    async function initApp() {
      try {
        const engineInfo = await fetchEngineStatus();
        if (engineInfo.is_ready) {
          setEngineReady(true);
        }

        // Fetch data independently so one failure never blocks the others
        let gData: GraphData = { nodes: [], links: [] };
        let nData: NoteSummary[] = [];
        let cFiles: CodeFileRecord[] = [];
        let pList: ProjectInfo[] = [];

        try { gData = await fetchGraph(); } catch (e) { console.warn('fetchGraph failed:', e); }
        try { nData = await fetchNotes(); } catch (e) { console.warn('fetchNotes failed:', e); }
        try { cFiles = await listCodeFiles(); } catch (e) { console.warn('listCodeFiles failed:', e); }
        try { pList = await fetchProjects(); } catch (e) { console.warn('fetchProjects failed:', e); }

        if (isCancelled) return;
        setGraphData(gData);
        setNotes(nData);
        setCodeFiles(cFiles);
        setProjects(pList);

        const active = pList.find((p) => p.is_active) || pList[0] || null;
        setActiveProject(active);

        if (nData.length > 0) {
          setSelectedNoteId(nData[0].id);
          setIsEditorOpen(false);
          try {
            const firstNote = await fetchNote(nData[0].id);
            if (!isCancelled && firstNote) {
              setActiveNote(firstNote);
            }
          } catch (e) {
            console.warn('fetchNote failed:', e);
          }
        }
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
    let unlistenMcp: (() => void) | undefined;
    let unlistenEngine: (() => void) | undefined;

    if (typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) {
      listen<number>('vault-sync-complete', (event) => {
        loadData();
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
      if (unlistenSync) unlistenSync();
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

  const handleDeleteNote = async (idOrTitle: string, title?: string) => {
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
      const project = await switchProject(projectId);
      setActiveProject(project);
      setSelectedNoteId(null);
      setActiveNote(null);
      setSelectedCodePath(null);
      setIsCodeInspectorOpen(false);
      showToast(`Active project: ${project.name}`);
      await Promise.allSettled([loadData(), loadProjects()]);
    } catch (err) {
      console.error('Error switching project:', err);
      showToast(`Error switching project: ${err}`, 'info');
    }
  };

  const handleCreateProject = async (name: string, path?: string) => {
    try {
      const project = await createProject(name, path);
      setActiveProject(project);
      setSelectedNoteId(null);
      setActiveNote(null);
      setSelectedCodePath(null);
      setIsCodeInspectorOpen(false);
      showToast(`New project "${project.name}" created successfully!`);
      await loadData();
      await loadProjects();
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
    } finally {
      setIsSyncing(false);
    }
  };

  // Non-blocking boot: UI renders immediately from local SQLite database
  const isLight = theme === 'light';

  return (
    <div
      className={`flex flex-col w-screen h-screen overflow-hidden select-none transition-colors ${
        isLight ? 'bg-slate-50 text-slate-800' : 'bg-[#090a0f] text-[#f1f5f9]'
      }`}
    >
      {/* Titlebar / Minimalist Header */}
      <header
        className={`h-12 px-3.5 flex items-center justify-between border-b shrink-0 z-40 transition-colors ${
          isLight ? 'border-slate-200 bg-white shadow-2xs' : 'border-[#181c26] bg-[#0c0e14]'
        }`}
      >
        {/* Left Section: Brand & Integrated Project Switcher */}
        <div className="flex items-center gap-2.5">
          <button
            type="button"
            onClick={() => setIsSidebarOpen(!isSidebarOpen)}
            className={`flex items-center justify-center w-7 h-7 rounded-lg transition-colors cursor-pointer ${
              isLight
                ? 'text-slate-500 hover:text-slate-900 hover:bg-slate-100'
                : 'text-[#64748b] hover:text-[#f1f5f9] hover:bg-[#161a24]'
            }`}
            title={isSidebarOpen ? 'Collapse Sidebar (Ctrl+B)' : 'Expand Sidebar (Ctrl+B)'}
          >
            <SidebarSimple size={16} weight="regular" />
          </button>

          <div className="flex items-center gap-2 mr-1">
            <span
              className={`text-[13px] font-bold tracking-[0.2px] ${
                isLight ? 'text-slate-900' : 'text-[#f8fafc]'
              }`}
            >
              GraphFinder
            </span>
          </div>

          <div className={`w-[1px] h-4 mx-0.5 ${isLight ? 'bg-slate-200' : 'bg-[#1f2433]'}`} />

          {/* Project Switcher */}
          <ProjectSwitcher
            projects={projects}
            activeProject={activeProject}
            onSwitchProject={handleSwitchProject}
            onOpenNewProjectModal={() => setIsNewProjectModalOpen(true)}
            theme={theme}
          />
        </div>

        {/* Center Section: Global Omnisearch Bar */}
        <div className="flex items-center">
          <button
            type="button"
            onClick={() => setIsOmnisearchOpen(true)}
            className={`flex items-center justify-between w-64 md:w-80 h-7 px-2.5 rounded-lg border text-xs transition-all cursor-pointer shadow-xs ${
              isLight
                ? 'bg-slate-50 hover:bg-slate-100 border-slate-200 hover:border-slate-300 text-slate-500'
                : 'bg-[#12151f] hover:bg-[#161b28] border-[#1f2536] hover:border-[#2f384f] text-[#64748b] hover:text-[#94a3b8]'
            }`}
            title="Search entire project (Ctrl+K)"
          >
            <div className="flex items-center gap-2 truncate">
              <MagnifyingGlass
                size={13}
                className={`${isLight ? 'text-slate-400' : 'text-[#64748b]'} shrink-0`}
              />
              <span className="truncate">Search notes or code...</span>
            </div>
            <kbd
              className={`hidden sm:inline-flex items-center gap-0.5 px-1.5 py-0.2 rounded border text-[9px] font-mono shrink-0 ${
                isLight
                  ? 'bg-white border-slate-200 text-slate-500 shadow-2xs'
                  : 'bg-[#1a2030] border-[#252e44] text-[#94a3b8]'
              }`}
            >
              Ctrl K
            </kbd>
          </button>
        </div>

        {/* Right Section: Primary Action & Tools */}
        <div className="flex items-center gap-2">
          {/* Non-blocking Neural Engine Status Indicator */}
          {!engineReady && (
            <div
              className={`hidden sm:flex items-center gap-1.5 px-2.5 py-1 rounded-full border text-[10.5px] font-medium transition-all ${
                engineError
                  ? 'bg-rose-500/10 border-rose-500/30 text-rose-400'
                  : isLight
                  ? 'bg-indigo-50 border-indigo-200 text-indigo-600 animate-pulse'
                  : 'bg-indigo-950/40 border-indigo-800/40 text-indigo-400 animate-pulse'
              }`}
              title={engineStatusMessage}
            >
              <div
                className={`w-1.5 h-1.5 rounded-full ${
                  engineError ? 'bg-rose-500' : 'bg-indigo-500'
                }`}
              />
              <span className="truncate max-w-[130px]">
                {engineError ? 'Engine Error' : 'Neural Warming Up...'}
              </span>
            </div>
          )}

          {/* Theme Toggle Button */}
          <button
            type="button"
            onClick={toggleTheme}
            className={`inline-flex items-center justify-center w-7 h-7 rounded-lg border transition-all cursor-pointer ${
              isLight
                ? 'bg-slate-50 hover:bg-slate-100 border-slate-200 text-amber-500 hover:text-amber-600 shadow-2xs'
                : 'bg-[#131620] hover:bg-[#191d2c] border-[#202636] hover:border-[#2f384f] text-amber-300 hover:text-amber-200'
            }`}
            title={isLight ? 'Switch to dark theme' : 'Switch to light theme'}
          >
            {isLight ? <Moon size={13} weight="fill" /> : <Sun size={13} weight="fill" />}
          </button>

          {/* New Note Button */}
          <button
            type="button"
            onClick={handleNewNote}
            className="inline-flex items-center gap-1.5 h-7 px-2.5 rounded-lg text-xs font-semibold bg-indigo-600 hover:bg-indigo-500 text-white shadow-xs transition-all cursor-pointer"
            title="Create new Markdown note"
          >
            <Plus size={12} weight="bold" />
            <span className="text-[11.5px]">New Note</span>
          </button>

          {/* Sync Button */}
          <button
            type="button"
            onClick={handleSync}
            disabled={isSyncing}
            className={`inline-flex items-center gap-1.5 h-7 px-2.5 rounded-lg text-xs font-medium border transition-all cursor-pointer disabled:opacity-50 ${
              isLight
                ? 'bg-slate-50 hover:bg-slate-100 border-slate-200 hover:border-slate-300 text-slate-700 shadow-2xs'
                : 'bg-[#131620] hover:bg-[#191d2c] border-[#202636] hover:border-[#2f384f] text-[#cbd5e1]'
            }`}
            title="Synchronize notes and code vault"
          >
            <ArrowClockwise
              size={12}
              weight="bold"
              className={
                isSyncing
                  ? 'animate-spin text-indigo-500'
                  : isLight
                  ? 'text-slate-500'
                  : 'text-[#64748b]'
              }
            />
            <span className="text-[11px]">{isSyncing ? 'Sync...' : 'Sync'}</span>
          </button>

          {/* MCP Status & Settings */}
          <button
            type="button"
            onClick={() => setIsMcpModalOpen(true)}
            className={`inline-flex items-center gap-1.5 h-7 px-2.5 rounded-lg text-xs font-medium border transition-all cursor-pointer ${
              isLight
                ? 'bg-slate-50 hover:bg-slate-100 border-slate-200 hover:border-slate-300 text-slate-700 shadow-2xs'
                : 'bg-[#131620] hover:bg-[#191d2c] border-[#202636] hover:border-[#2f384f] text-[#cbd5e1]'
            }`}
            title="MCP Connection &amp; Agent Rules"
          >
            <span className="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse" />
            <span
              className={`text-[11px] font-mono ${
                isLight ? 'text-slate-600' : 'text-[#94a3b8]'
              }`}
            >
              MCP
            </span>
          </button>
        </div>
      </header>

      {/* Main Studio Body */}
      <div
        className={`flex-1 flex overflow-hidden relative ${
          isLight ? 'bg-slate-100' : 'bg-[#111318]'
        }`}
      >
        <Sidebar
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
          chunkResults={chunkResults}
          codeChunkResults={codeChunkResults}
          isSearching={isSearching}
          isCollapsed={!isSidebarOpen}
          theme={theme}
        />

        <main
          className={`flex-1 relative h-full overflow-hidden ${
            isLight ? 'bg-[#f8fafc]' : 'bg-[#090a0f]'
          }`}
        >
          <GraphView
            graphData={graphData}
            selectedNodeId={selectedNoteId}
            onSelectNode={handleSelectNote}
            onSelectCode={handleSelectCodeFile}
            onRefresh={loadData}
            glowingNode={glowingNode}
            theme={theme}
          />

          {isEditorOpen && (
            <NoteEditor
              note={activeNote}
              onSave={handleSaveNote}
              onDelete={handleDeleteNote}
              onClose={() => setIsEditorOpen(false)}
              onNavigateNote={handleNavigateNote}
              availableNotes={notes}
              theme={theme}
            />
          )}

          {isCodeInspectorOpen && selectedCodePath && (
            <CodeInspector
              fileId={selectedCodePath}
              initialSymbol={selectedCodeSymbol}
              onClose={() => setIsCodeInspectorOpen(false)}
              onNavigateFile={(fileId) => {
                setSelectedCodePath(fileId);
                setSelectedCodeSymbol(null);
              }}
              theme={theme}
            />
          )}
        </main>
      </div>

      {/* Dynamic Modals */}
      <OmnisearchModal
        isOpen={isOmnisearchOpen}
        onClose={() => setIsOmnisearchOpen(false)}
        notes={notes}
        codeFiles={codeFiles}
        onSelectNote={handleSelectNote}
        onSelectCodeFile={handleSelectCodeFile}
        theme={theme}
      />
      <McpModal isOpen={isMcpModalOpen} onClose={() => setIsMcpModalOpen(false)} />
      <NewProjectModal
        isOpen={isNewProjectModalOpen}
        onClose={() => setIsNewProjectModalOpen(false)}
        onCreate={handleCreateProject}
      />

      {/* Toast Notifications */}
      {toasts.length > 0 && (
        <div className="fixed bottom-4 right-4 z-50 flex flex-col gap-2 pointer-events-none">
          {toasts.map((toast) => (
            <div
              key={toast.id}
              className={`flex items-center gap-2 px-3 py-1.5 rounded-lg border text-xs font-medium shadow-2xl backdrop-blur-md pointer-events-auto animate-in fade-in slide-in-from-bottom-2 duration-200 ${
                isLight
                  ? 'bg-white/95 border-slate-200 text-slate-800'
                  : 'bg-[#14171f] border-[#262933] text-[#f1f5f9]'
              }`}
            >
              <CheckCircle size={14} weight="fill" className="text-[#10b981] shrink-0" />
              <span>{toast.message}</span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};

export default App;
