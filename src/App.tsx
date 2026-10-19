import React, { useState, useEffect, useCallback } from 'react';
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

  const handleSelectNote = useCallback(async (id: string) => {
    setSelectedNoteId(id);
    setIsEditorOpen(true);
    try {
      const note = await fetchNote(id);
      setActiveNote(note);
    } catch (err) {
      console.error('Erro ao buscar nota:', err);
    }
  }, []);

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

      if (!selectedNoteId && nData.length > 0) {
        handleSelectNote(nData[0].id);
      }
    } catch (err) {
      console.error('Erro ao carregar dados do GraphFinder:', err);
    }
  }, [selectedNoteId, handleSelectNote]);

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
    return () => {
      isCancelled = true;
    };
  }, []);

  const handleNavigateNote = async (titleOrId: string) => {
    const found = notes.find(
      (n) => n.id === titleOrId || n.title.toLowerCase() === titleOrId.toLowerCase()
    );
    if (found) {
      handleSelectNote(found.id);
    } else {
      setSelectedNoteId(null);
      setActiveNote({
        id: '',
        path: `${titleOrId}.md`,
        title: titleOrId,
        summary: '',
        content: `# ${titleOrId}\n\nConexao criada a partir do grafo.`,
        updated_at: Math.floor(Date.now() / 1000),
        tags: [],
        outgoing_links: [],
        backlinks: [],
      });
      setIsEditorOpen(true);
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
      showToast(`Nota "${title}" salva com sucesso!`);
      await loadData();
      await loadProjects();
    } catch (err) {
      console.error('Erro ao salvar nota:', err);
    }
  };

  const handleSwitchProject = async (projectId: string) => {
    try {
      const project = await switchProject(projectId);
      setActiveProject(project);
      setSelectedNoteId(null);
      setActiveNote(null);
      showToast(`Cofre alternado para: ${project.name}`);
      await loadData();
      await loadProjects();
    } catch (err) {
      console.error('Erro ao alternar projeto:', err);
    }
  };

  const handleCreateProject = async (name: string) => {
    try {
      const project = await createProject(name);
      setActiveProject(project);
      setSelectedNoteId(null);
      setActiveNote(null);
      showToast(`Novo vault "${project.name}" criado com sucesso!`);
      await loadData();
      await loadProjects();
    } catch (err) {
      console.error('Erro ao criar projeto:', err);
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
      console.error('Erro na busca semantica por secoes:', err);
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
      showToast(`Sincronizacao concluida (${count} notas reindexadas)`);
    } catch (err) {
      console.error('Erro ao sincronizar:', err);
    } finally {
      setIsSyncing(false);
    }
  };

  return (
    <div className="flex flex-col w-screen h-screen overflow-hidden bg-[#090a0f] text-zinc-100 select-none">
      {/* Top Application Header */}
      <header className="h-14 px-4 flex items-center justify-between border-b border-zinc-800/80 bg-zinc-950/80 backdrop-blur-md shrink-0 z-40">
        <div className="flex items-center gap-3">
          <button
            type="button"
            onClick={() => setIsSidebarOpen(!isSidebarOpen)}
            className="flex items-center justify-center w-8 h-8 rounded-md text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800/80 transition-colors cursor-pointer"
            title={isSidebarOpen ? 'Ocultar Painel Lateral' : 'Exibir Painel Lateral'}
          >
            <SidebarSimple size={18} weight="regular" />
          </button>

          <div className="flex items-center gap-2.5">
            <div className="flex items-center justify-center w-7 h-7 rounded-md bg-zinc-900 border border-zinc-800 overflow-hidden">
              <img
                src="/favicon.svg"
                alt="GraphFinder"
                className="w-5 h-5 object-contain"
              />
            </div>
            <div className="flex flex-col">
              <div className="flex items-center gap-1.5 leading-none">
                <span className="text-sm font-bold tracking-tight text-zinc-100">
                  GraphFinder
                </span>
                <span className="text-[10px] font-mono px-1.5 py-0.2 rounded bg-zinc-800 text-zinc-400 border border-zinc-700/50">
                  v0.2.0
                </span>
              </div>
              <span className="text-[10px] text-zinc-400 mt-0.5 font-medium">
                Neural Knowledge Graph & Semantic Vault
              </span>
            </div>
          </div>
        </div>

        <div className="flex items-center">
          <ProjectSwitcher
            projects={projects}
            activeProject={activeProject}
            onSwitchProject={handleSwitchProject}
            onOpenNewProjectModal={() => setIsNewProjectModalOpen(true)}
          />
        </div>

        <div className="flex items-center gap-2">
          <button
            type="button"
            onClick={handleSync}
            disabled={isSyncing}
            className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium bg-zinc-900 hover:bg-zinc-800 border border-zinc-800 hover:border-zinc-700 text-zinc-300 transition-all cursor-pointer disabled:opacity-50"
            title="Sincronizar arquivos .md do Vault com chunking semantico"
          >
            <ArrowClockwise
              size={13}
              weight="bold"
              className={isSyncing ? 'animate-spin text-indigo-400' : 'text-zinc-400'}
            />
            <span>{isSyncing ? 'Indexando...' : 'Sincronizar Vault'}</span>
          </button>

          <button
            type="button"
            onClick={() => setIsMcpModalOpen(true)}
            className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium bg-zinc-900 hover:bg-zinc-800 border border-zinc-800 hover:border-zinc-700 text-zinc-300 transition-all cursor-pointer"
            title="Copiar Regras do Agente, Skill de Curadoria e Configuracao MCP para seu Editor"
          >
            <Robot size={14} weight="regular" className="text-zinc-400" />
            <span>Conectar IA & MCP</span>
          </button>
        </div>
      </header>

      {/* Main Studio Body */}
      <div className="flex-1 flex overflow-hidden relative">
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

        <main className="flex-1 relative h-full overflow-hidden">
          <GraphView
            graphData={graphData}
            selectedNodeId={selectedNoteId}
            onSelectNode={handleSelectNote}
            onRefresh={loadData}
          />

          {isEditorOpen && (
            <NoteEditor
              key={selectedNoteId || 'new'}
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
              className="flex items-center gap-2 px-3.5 py-2 rounded-lg bg-zinc-900/95 border border-zinc-800 text-xs font-medium text-zinc-200 shadow-xl backdrop-blur-md pointer-events-auto animate-in fade-in slide-in-from-bottom-2 duration-200"
            >
              <CheckCircle size={15} weight="fill" className="text-emerald-400 shrink-0" />
              <span>{toast.message}</span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};

export default App;
