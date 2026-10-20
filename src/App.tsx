import React, { useState, useEffect, useCallback } from 'react';
import { listen } from '@tauri-apps/api/event';
import {
  fetchGraph,
  fetchNotes,
  fetchNote,
  saveNote,
  searchChunks,
  syncVault,
  fetchProjects,
  switchProject,
  createProject,
} from './services/api';
import type { ChunkHit, GraphData, NoteRecord, NoteSummary, ProjectInfo } from './types';
import { GraphView } from './components/GraphView';
import { NoteEditor } from './components/NoteEditor';
import { Sidebar } from './components/Sidebar';
import { McpModal } from './components/McpModal';
import { ProjectSwitcher } from './components/ProjectSwitcher';
import { NewProjectModal } from './components/NewProjectModal';
import { ArrowClockwise, Robot, SidebarSimple, CheckCircle } from '@phosphor-icons/react';

interface Toast {
  id: string;
  message: string;
  type?: 'success' | 'info';
}

export const App: React.FC = () => {
  const [graphData, setGraphData] = useState<GraphData>({ nodes: [], links: [] });
  const [notes, setNotes] = useState<NoteSummary[]>([]);
  const [selectedNoteId, setSelectedNoteId] = useState<string | null>(null);
  const [activeNote, setActiveNote] = useState<NoteRecord | null>(null);
  const [chunkResults, setChunkResults] = useState<ChunkHit[]>([]);
  const [isSearching, setIsSearching] = useState(false);
  const [isSyncing, setIsSyncing] = useState(false);
  const [isEditorOpen, setIsEditorOpen] = useState(false);
  const [isMcpModalOpen, setIsMcpModalOpen] = useState(false);
  const [isNewProjectModalOpen, setIsNewProjectModalOpen] = useState(false);
  const [isSidebarOpen, setIsSidebarOpen] = useState(true);
  const [projects, setProjects] = useState<ProjectInfo[]>([]);
  const [activeProject, setActiveProject] = useState<ProjectInfo | null>(null);
  const [toasts, setToasts] = useState<Toast[]>([]);

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
        content: `# ${clean}\n\nConexao criada a partir do grafo.`,
        updated_at: Math.floor(Date.now() / 1000),
        tags: [],
        outgoing_links: [],
        backlinks: [],
      });
    } catch (err) {
      console.error('Erro ao buscar nota:', err);
    }
  }, [notes]);

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
      console.error('Erro ao listar projetos:', err);
    }
  }, []);

  const loadData = useCallback(async () => {
    try {
      const [gData, nData] = await Promise.all([fetchGraph(), fetchNotes()]);
      setGraphData(gData);
      setNotes(nData);
    } catch (err) {
      console.error('Erro ao carregar dados do GraphFinder:', err);
    }
  }, []);

  useEffect(() => {
    let isCancelled = false;

    async function initApp() {
      try {
        const [gData, nData, pList] = await Promise.all([
          fetchGraph(),
          fetchNotes(),
          fetchProjects(),
        ]);
        if (isCancelled) return;
        setGraphData(gData);
        setNotes(nData);
        setProjects(pList);

        const active = pList.find((p) => p.is_active) || pList[0] || null;
        setActiveProject(active);

        if (nData.length > 0) {
          setSelectedNoteId(nData[0].id);
          setIsEditorOpen(true);
          const firstNote = await fetchNote(nData[0].id);
          if (!isCancelled && firstNote) {
            setActiveNote(firstNote);
          }
        }
      } catch (err) {
        console.error('Erro na inicializacao:', err);
      }
    }

    initApp();

    let unlistenSync: (() => void) | undefined;
    if (typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) {
      listen<number>('vault-sync-complete', (event) => {
        loadData();
        if (event.payload > 0) {
          showToast(`${event.payload} nota(s) sincronizadas em segundo plano`, 'info');
        }
      }).then((unlisten) => {
        unlistenSync = unlisten;
      });
    }

    return () => {
      isCancelled = true;
      if (unlistenSync) unlistenSync();
    };
  }, []); // Run only once on mount

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
      showToast(`Note "${title}" saved successfully!`);
      await loadData();
      await loadProjects();
    } catch (err) {
      console.error('Error saving note:', err);
    }
  };

  const handleSwitchProject = async (projectId: string) => {
    try {
      const project = await switchProject(projectId);
      setActiveProject(project);
      setSelectedNoteId(null);
      setActiveNote(null);
      showToast(`Vault switched to: ${project.name}`);
      await loadData();
      await loadProjects();
    } catch (err) {
      console.error('Error switching project:', err);
    }
  };

  const handleCreateProject = async (name: string, path?: string) => {
    try {
      const project = await createProject(name, path);
      setActiveProject(project);
      setSelectedNoteId(null);
      setActiveNote(null);
      showToast(`New vault "${project.name}" created successfully!`);
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

  const handleSync = async () => {
    setIsSyncing(true);
    try {
      const count = await syncVault();
      await loadData();
      await loadProjects();
      showToast(`Sync completed (${count} notes reindexed)`);
    } catch (err) {
      console.error('Error during sync:', err);
    } finally {
      setIsSyncing(false);
    }
  };

  return (
    <div className="flex flex-col w-screen h-screen overflow-hidden bg-[#090a0f] text-[#f1f5f9] select-none">
      {/* Titlebar matching banner.svg lines 15-35 */}
      <header className="h-12 px-4 flex items-center justify-between border-b border-[#1f232d] bg-[#14171f] shrink-0 z-40">
        {/* Left Section: Window dots & Brand */}
        <div className="flex items-center gap-3">
          {/* Subtle Window Control Dots from banner.svg */}
          <div className="hidden sm:flex items-center gap-1.5 mr-1">
            <span className="w-2.5 h-2.5 rounded-full bg-[#323642]" />
            <span className="w-2.5 h-2.5 rounded-full bg-[#323642]" />
            <span className="w-2.5 h-2.5 rounded-full bg-[#323642]" />
          </div>

          <button
            type="button"
            onClick={() => setIsSidebarOpen(!isSidebarOpen)}
            className="flex items-center justify-center w-7 h-7 rounded-md text-[#64748b] hover:text-[#f1f5f9] hover:bg-[#1a1e28] transition-colors cursor-pointer"
            title={isSidebarOpen ? 'Hide Sidebar' : 'Show Sidebar'}
          >
            <SidebarSimple size={16} weight="regular" />
          </button>

          <div className="flex items-center gap-2">
            <span className="text-[13px] font-semibold tracking-[0.3px] text-[#f1f5f9]">
              GraphFinder
            </span>
            <span className="text-[10px] font-mono font-semibold px-1.5 py-0.5 rounded bg-[#1e222d] text-[#94a3b8] border border-[#2d3342]">
              v{typeof __APP_VERSION__ !== 'undefined' ? __APP_VERSION__ : '0.0.1'}
            </span>
          </div>
        </div>

        {/* Center Section: Capsule Vault Indicator from banner.svg */}
        <div className="flex items-center">
          <ProjectSwitcher
            projects={projects}
            activeProject={activeProject}
            onSwitchProject={handleSwitchProject}
            onOpenNewProjectModal={() => setIsNewProjectModalOpen(true)}
          />
        </div>

        {/* Right Section: Telemetry pill & Actions */}
        <div className="flex items-center gap-2.5">
          <button
            type="button"
            onClick={() => setIsMcpModalOpen(true)}
            className="hidden md:inline-flex items-center gap-1.5 px-2 py-0.5 rounded-full bg-[#13161f] border border-[#1e222d] hover:border-[#2d3342] text-[#94a3b8] hover:text-[#cbd5e1] font-mono text-[11px] transition-colors cursor-pointer"
            title="MCP Server HTTP/SSE escutando em http://127.0.0.1:45123"
          >
            <span className="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse"></span>
            <span>mcp: 127.0.0.1:45123</span>
          </button>

          <button
            type="button"
            onClick={handleSync}
            disabled={isSyncing}
            className="inline-flex items-center gap-1.5 h-7 px-2.5 rounded-md text-xs font-medium bg-[#13161f] hover:bg-[#1a1e28] border border-[#1e222d] hover:border-[#2d3342] text-[#cbd5e1] transition-all cursor-pointer disabled:opacity-50"
            title="Sync Vault Markdown files with semantic chunking"
          >
            <ArrowClockwise
              size={12}
              weight="bold"
              className={isSyncing ? 'animate-spin text-indigo-400' : 'text-[#64748b]'}
            />
            <span className="text-[11px]">{isSyncing ? 'Indexing...' : 'Sync Vault'}</span>
          </button>

          <button
            type="button"
            onClick={() => setIsMcpModalOpen(true)}
            className="inline-flex items-center gap-1.5 h-7 px-2.5 rounded-md text-xs font-medium bg-[#13161f] hover:bg-[#1a1e28] border border-[#1e222d] hover:border-[#2d3342] text-[#cbd5e1] transition-all cursor-pointer"
            title="Copy Agent Rules, Curator Skill, and MCP Configuration"
          >
            <Robot size={13} weight="regular" className="text-[#64748b]" />
            <span className="text-[11px]">Connect AI & MCP</span>
          </button>
        </div>
      </header>

      {/* Main Studio Body with border framing */}
      <div className="flex-1 flex overflow-hidden relative bg-[#111318]">
        <Sidebar
          notes={notes}
          selectedNoteId={selectedNoteId}
          onSelectNote={handleSelectNote}
          onNewNote={handleNewNote}
          onSearchSemantic={handleSearchSemantic}
          chunkResults={chunkResults}
          isSearching={isSearching}
          isCollapsed={!isSidebarOpen}
        />

        <main className="flex-1 relative h-full overflow-hidden bg-[#090a0f]">
          <GraphView
            graphData={graphData}
            selectedNodeId={selectedNoteId}
            onSelectNode={handleSelectNote}
            onRefresh={loadData}
          />

          {isEditorOpen && (
            <NoteEditor
              key={activeNote ? activeNote.id || activeNote.title : 'new-note'}
              note={activeNote}
              onSave={handleSaveNote}
              onClose={() => setIsEditorOpen(false)}
              onNavigateNote={handleNavigateNote}
            />
          )}
        </main>
      </div>

      {/* Dynamic Modals */}
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
              className="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-[#14171f] border border-[#262933] text-xs font-medium text-[#f1f5f9] shadow-2xl backdrop-blur-md pointer-events-auto animate-in fade-in slide-in-from-bottom-2 duration-200"
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
