import { invoke } from '@tauri-apps/api/core';
import type {
  ChunkHit,
  CompressedContext,
  GraphData,
  LinkSuggestion,
  Neighborhood,
  NoteRecord,
  NoteSummary,
  ProjectInfo,
  SearchHit,
} from '../types';

const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

const MOCK_NOTES: NoteRecord[] = [
  {
    id: 'architecture',
    path: 'Architecture.md',
    title: 'System Architecture',
    summary: 'Divided into three pillars: SQLite Storage, FastEmbed Semantic Engine, and MCP Server.',
    content: `# System Architecture\n\nGraphFinder is built upon three high-density architectural pillars designed for data sovereignty and surgical token efficiency:\n\n## 1. Local Storage and SQLite\n- **Plain Markdown (\`.md\`) Files**: Stored directly in your local vault directory. 100% human-readable with standard wikilinks [[Link]].\n- **Local SQLite Engine**: Stores nodes, relational edges, and vector embeddings in WAL mode for concurrent, low-latency queries.\n\n## 2. Rust Semantic Engine\n- Powered by **FastEmbed** and ONNX Runtime for offline embeddings with zero cloud dependencies.\n- Implements graph traversal algorithms (k-hop expansion) to extract focused neighborhoods around any query or note.\n- Learn more in [[Welcome]].\n\n## 3. Model Context Protocol (MCP) Server\n- Exposes tools like \`search_semantic\`, \`get_compressed_context\`, and \`store_context\` via stdio for AI coding assistants.\n- See integration workflows in the [[Usage Guide]].\n\nTags: #architecture #rust #sqlite #embeddings #mcp`,
    updated_at: Math.floor(Date.now() / 1000) - 3600,
    tags: ['architecture', 'rust', 'sqlite', 'embeddings', 'mcp'],
    outgoing_links: ['Welcome', 'Usage Guide'],
    backlinks: ['Welcome', 'AI Concepts and RAG'],
  },
  {
    id: 'welcome',
    path: 'Welcome.md',
    title: 'Welcome to GraphFinder',
    summary: 'Overview of the neural graph knowledge vault and local semantic search engine.',
    content: `# Welcome to GraphFinder\n\nGraphFinder is a local-first knowledge management system powered by an interactive graph and local vector search.\n\n## What Makes GraphFinder Unique?\n- **Local Semantic Retrieval**: Discover notes by conceptual meaning, not just exact keywords.\n- **Bidirectional Graph**: Connect ideas naturally through [[Architecture]].\n- **Zero Telemetry**: All AI vector processing runs locally on your machine with zero cloud dependencies.\n\n## Getting Started\n1. Explore connections in the interactive graph canvas.\n2. Create new notes using the [[Note Title]] syntax to link concepts.\n3. Follow the [[Usage Guide]] to configure the embedded MCP server for your AI coding assistants.\n4. Read [[AI Concepts and RAG]] to understand our atomic chunking model.\n\nTags: #welcome #onboarding #tutorial`,
    updated_at: Math.floor(Date.now() / 1000) - 7200,
    tags: ['welcome', 'onboarding', 'tutorial'],
    outgoing_links: ['Architecture', 'Usage Guide', 'AI Concepts and RAG'],
    backlinks: ['Architecture', 'Usage Guide'],
  },
  {
    id: 'usage-guide',
    path: 'Usage Guide.md',
    title: 'Usage Guide',
    summary: 'Practical workflows to maximize productivity and token efficiency in GraphFinder.',
    content: `# Usage Guide\n\nPractical workflows to maximize productivity and token efficiency in GraphFinder:\n\n## 1. Creating and Connecting Notes\n- To connect notes, use the standard wikilink syntax \`[[Note Title]]\`.\n- Example: Link directly to [[Architecture]] or return to [[Welcome]].\n- The interactive 2D graph updates in real time as you type.\n\n## 2. Tags and Categorization\n- Add contextual hashtags such as \`#research\`, \`#project\`, or \`#architecture\` to your notes.\n- The graph canvas groups and visualizes nodes by color based on their dominant tag clusters.\n\n## 3. Operating via MCP\n- AI agents query your knowledge vault using surgical atomic tools (\`search_semantic\`).\n- Agents receive precise deep links like \`[[Note#Section]]\` consuming ~50 tokens rather than whole files.\n- Check our RAG foundations in [[AI Concepts and RAG]].\n\nTags: #guide #tutorial #productivity #mcp`,
    updated_at: Math.floor(Date.now() / 1000) - 10800,
    tags: ['guide', 'tutorial', 'productivity', 'mcp'],
    outgoing_links: ['Architecture', 'Welcome', 'AI Concepts and RAG'],
    backlinks: ['Architecture', 'Welcome'],
  },
  {
    id: 'ai-concepts-and-rag',
    path: 'AI Concepts and RAG.md',
    title: 'AI Concepts and Atomic RAG',
    summary: 'Foundations of atomic Retrieval-Augmented Generation, semantic chunking, and context conservation.',
    content: `# AI Concepts and Atomic RAG\n\nHow GraphFinder transforms standard Markdown into a high-density, token-efficient knowledge base for LLMs:\n\n## 1. Atomic Chunking by Headers\n- Rather than vectorizing entire documents or arbitrary character windows, GraphFinder splits notes strictly across \`##\` and \`###\` headers.\n- Each chunk averages 40 to 150 tokens, creating fine-grained vectors with clear semantic boundaries.\n\n## 2. Deep Linking and Context Economy\n- When an AI agent performs a semantic query, the MCP server returns only the exact matching subsection with a \`[[Note#Section]]\` deep link.\n- This preserves context window budget and eliminates prompt degradation.\n- Review technical implementation in [[Architecture]].\n\nTags: #ai #rag #embeddings #architecture`,
    updated_at: Math.floor(Date.now() / 1000) - 14400,
    tags: ['ai', 'rag', 'embeddings', 'architecture'],
    outgoing_links: ['Architecture'],
    backlinks: ['Welcome', 'Usage Guide'],
  },
];

export async function fetchGraph(): Promise<GraphData> {
  if (isTauri) {
    return await invoke<GraphData>('get_graph');
  }
  return {
    nodes: [
      { id: 'architecture', title: 'System Architecture', tags: ['architecture', 'rust', 'sqlite'], degree: 4 },
      { id: 'welcome', title: 'Welcome to GraphFinder', tags: ['welcome', 'onboarding'], degree: 3 },
      { id: 'usage-guide', title: 'Usage Guide', tags: ['guide', 'tutorial', 'mcp'], degree: 3 },
      { id: 'ai-concepts-and-rag', title: 'AI Concepts and Atomic RAG', tags: ['ai', 'rag', 'architecture'], degree: 2 },
      { id: 'fastembed-local', title: 'FastEmbed ONNX Engine', tags: ['rust', 'embeddings'], degree: 2 },
      { id: 'sqlite-wal', title: 'SQLite WAL Storage', tags: ['sqlite', 'architecture'], degree: 2 },
    ],
    links: [
      { source: 'architecture', target: 'welcome', label: 'context' },
      { source: 'architecture', target: 'usage-guide', label: 'instructions' },
      { source: 'ai-concepts-and-rag', target: 'architecture', label: 'implements' },
      { source: 'usage-guide', target: 'welcome', label: 'onboarding' },
      { source: 'fastembed-local', target: 'architecture', label: 'engine' },
      { source: 'sqlite-wal', target: 'architecture', label: 'persistence' },
      { source: 'ai-concepts-and-rag', target: 'fastembed-local', label: 'vectorizes' },
    ],
  };
}

export async function fetchNotes(): Promise<NoteSummary[]> {
  if (isTauri) {
    return await invoke<NoteSummary[]>('list_notes');
  }
  return MOCK_NOTES.map((n) => ({
    id: n.id,
    path: n.path,
    title: n.title,
    summary: n.summary,
    updated_at: n.updated_at,
    tags: n.tags,
  }));
}

export async function fetchNote(id: string): Promise<NoteRecord | null> {
  if (isTauri) {
    return await invoke<NoteRecord | null>('get_note', { id });
  }
  return MOCK_NOTES.find((n) => n.id === id || n.title.toLowerCase() === id.toLowerCase()) || null;
}

export async function saveNote(
  title: string,
  content: string,
  filename?: string
): Promise<NoteRecord> {
  if (isTauri) {
    return await invoke<NoteRecord>('save_note', { title, content, filename });
  }
  const id = title.toLowerCase().replace(/[^a-z0-9]+/g, '-');
  const record: NoteRecord = {
    id,
    path: filename || `${title}.md`,
    title,
    summary: content.slice(0, 100),
    content,
    updated_at: Math.floor(Date.now() / 1000),
    tags: ['nova-nota'],
    outgoing_links: [],
    backlinks: [],
  };
  const existing = MOCK_NOTES.findIndex((n) => n.id === id);
  if (existing >= 0) {
    MOCK_NOTES[existing] = record;
  } else {
    MOCK_NOTES.unshift(record);
  }
  return record;
}

export async function searchSemantic(query: string, limit: number = 10): Promise<SearchHit[]> {
  if (isTauri) {
    return await invoke<SearchHit[]>('search_semantic', { query, limit });
  }
  const q = query.toLowerCase();
  return MOCK_NOTES.filter(
    (n) => n.title.toLowerCase().includes(q) || n.content.toLowerCase().includes(q)
  ).map((n) => ({
    id: n.id,
    title: n.title,
    snippet: n.summary,
    score: 0.88,
    tags: n.tags,
  }));
}

export async function searchChunks(query: string, limit: number = 8): Promise<ChunkHit[]> {
  if (isTauri) {
    return await invoke<ChunkHit[]>('search_chunks', { query, limit });
  }
  return [
    {
      chunk_id: 'chk-1',
      note_id: 'architecture',
      note_title: 'System Architecture',
      heading: '2. Rust Semantic Engine',
      deep_link: '[[Architecture#2. Rust Semantic Engine]]',
      content: 'Powered by FastEmbed and ONNX Runtime for offline embeddings with zero cloud dependencies.',
      token_estimate: 48,
      score: 0.94,
    },
    {
      chunk_id: 'chk-2',
      note_id: 'ai-concepts-and-rag',
      note_title: 'AI Concepts and Atomic RAG',
      heading: '1. Atomic Chunking by Headers',
      deep_link: '[[AI Concepts and RAG#1. Atomic Chunking by Headers]]',
      content: 'Rather than vectorizing entire documents, splits notes strictly across headers generating independent vectors of 40 to 150 tokens.',
      token_estimate: 56,
      score: 0.89,
    },
    {
      chunk_id: 'chk-3',
      note_id: 'usage-guide',
      note_title: 'Usage Guide',
      heading: '3. Operating via MCP',
      deep_link: '[[Usage Guide#3. Operating via MCP]]',
      content: 'Tools return deep links in [[Note#Section]] format to preserve context window tokens in LLM attention layers.',
      token_estimate: 52,
      score: 0.84,
    },
  ];
}

export async function suggestLinks(
  text: string,
  currentNote?: string,
  threshold: number = 0.6
): Promise<LinkSuggestion[]> {
  if (isTauri) {
    return await invoke<LinkSuggestion[]>('suggest_links', {
      text,
      currentNote,
      threshold,
      limit: 4,
    });
  }
  return [
    {
      target_note_id: 'architecture',
      target_title: 'System Architecture',
      heading: '1. Local Storage and SQLite',
      deep_link: '[[Architecture#1. Local Storage and SQLite]]',
      similarity_score: 0.89,
      preview: 'Local SQLite Engine: Stores nodes, relational edges, and vector embeddings in WAL mode.',
    },
    {
      target_note_id: 'ai-concepts-and-rag',
      target_title: 'AI Concepts and Atomic RAG',
      heading: 'Atomic Chunking',
      deep_link: '[[AI Concepts and RAG#Atomic Chunking]]',
      similarity_score: 0.82,
      preview: 'Fine-grained section segmentation for LLM token conservation.',
    },
  ];
}

export async function getCompressedContext(target: string): Promise<CompressedContext | null> {
  if (isTauri) {
    return await invoke<CompressedContext | null>('get_compressed_context', { target });
  }
  return null;
}

export async function fetchNeighborhood(target: string, depth: number = 1): Promise<Neighborhood> {
  if (isTauri) {
    return await invoke<Neighborhood>('get_neighborhood', { target, depth });
  }
  return { center: null, nodes: [], links: [] };
}

export async function linkNotes(from: string, to: string): Promise<NoteRecord> {
  if (isTauri) {
    return await invoke<NoteRecord>('link_notes', { from, to });
  }
  throw new Error('Tauri context not available');
}

export async function syncVault(): Promise<number> {
  if (isTauri) {
    return await invoke<number>('sync_vault');
  }
  return 0;
}

export interface McpConfigInfo {
  executable_path: string;
  vault_path: string;
  db_path: string;
  config_json: string;
}

export async function fetchMcpConfig(): Promise<McpConfigInfo> {
  if (isTauri) {
    return await invoke<McpConfigInfo>('get_mcp_config');
  }
  return {
    executable_path: 'graphfinder.exe',
    vault_path: './vault',
    db_path: './index.db',
    config_json: `{
  "mcpServers": {
    "graphfinder": {
      "type": "sse",
      "url": "http://127.0.0.1:45123/sse"
    }
  }
}`,
  };
}

export async function fetchProjects(): Promise<ProjectInfo[]> {
  if (isTauri) {
    return await invoke<ProjectInfo[]>('list_projects');
  }
  return [
    {
      id: 'default',
      name: 'Principal',
      path: './vault',
      db_path: './index.db',
      note_count: 4,
      is_active: true,
    },
  ];
}

export async function switchProject(projectId: string): Promise<ProjectInfo> {
  if (isTauri) {
    return await invoke<ProjectInfo>('switch_project', { projectId });
  }
  return {
    id: projectId,
    name: projectId,
    path: `./vaults/${projectId}`,
    db_path: `./vaults/${projectId}/index.db`,
    note_count: 0,
    is_active: true,
  };
}

export async function pickDirectory(): Promise<string | null> {
  if (isTauri) {
    return await invoke<string | null>('pick_directory');
  }
  return null;
}

export async function createProject(name: string, path?: string): Promise<ProjectInfo> {
  if (isTauri) {
    return await invoke<ProjectInfo>('create_project', { name, path: path || null });
  }
  const cleanPath = path || `./vaults/${name.toLowerCase()}`;
  return {
    id: name.toLowerCase(),
    name,
    path: cleanPath,
    db_path: `${cleanPath}/index.db`,
    note_count: 1,
    is_active: true,
  };
}

