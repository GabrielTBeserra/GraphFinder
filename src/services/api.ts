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
    id: 'arquitetura',
    path: 'Arquitetura.md',
    title: 'Arquitetura do Sistema',
    summary: 'Divisão em três pilares: Armazenamento Físico SQLite, Motor Semântico FastEmbed e Protocolo MCP.',
    content: `# Arquitetura do Sistema\n\nA arquitetura do GraphFinder é dividida em três pilares principais de alta densidade semântica:\n\n## 1. Armazenamento Físico e SQLite\n- **Arquivos Markdown (\`.md\`)**: Localizados no diretório local do vault. Legíveis por humanos, compatíveis com Obsidian.\n- **SQLite Local**: Guarda a tabela de nós, arestas (links bidirecionais extraídos do markdown) e vetores de embedding calculados localmente com suporte a WAL mode.\n\n## 2. Motor Semântico em Rust\n- Utiliza **FastEmbed** para calcular representações vetoriais semânticas via ONNX Runtime local sem enviar dados para a nuvem.\n- Expõe algoritmos de grafo (k-hop expansion) para extrair subgrafos relevantes ao redor de uma pesquisa ou nota.\n- Veja mais em [[Boas-vindas]].\n\n## 3. Protocolo MCP (Model Context Protocol)\n- Permite que ferramentas de IA usem comandos como \`store_context\` e \`search_semantic\` diretamente do seu editor.\n- Saiba como interagir em [[Guia de Uso]].\n\nTags: #arquitetura #rust #sqlite #embeddings #mcp\n\n- Conexão Semântica: [[Boas-vindas#O que torna o GraphFinder especial?]]\n- Conexão Semântica: [[Guia de Uso#Usando via MCP]]`,
    updated_at: Math.floor(Date.now() / 1000) - 3600,
    tags: ['arquitetura', 'rust', 'sqlite', 'embeddings', 'mcp'],
    outgoing_links: ['Boas-vindas', 'Guia de Uso'],
    backlinks: ['Boas-vindas', 'Conceitos de IA e RAG'],
  },
  {
    id: 'boas-vindas',
    path: 'Boas-vindas.md',
    title: 'Boas-vindas ao GraphFinder',
    summary: 'Visão geral do cofre de conhecimento em grafo e motor de busca semântica local.',
    content: `# Boas-vindas ao GraphFinder\n\nO GraphFinder é um sistema local-first de gerenciamento de conhecimento baseado em grafos e busca vetorial.\n\n## O que torna o GraphFinder especial?\n- **Busca Semântica Local**: Encontre notas por significado conceitual, não apenas palavras-chave exatas.\n- **Grafo Bidirecional**: Conecte ideias naturalmente através de [[Arquitetura]].\n- **Zero Telemetria**: Todo o processamento de IA roda localmente na sua máquina.\n\n## Primeiros Passos\n1. Explore as conexões no grafo interativo.\n2. Crie novas notas e use a sintaxe [[Nome da Nota]] para conectar conceitos.\n3. Configure o servidor MCP para integrar seus assistentes de IA.\n\nTags: #inicio #boasvindas #tutorial\n\n- Conexão Semântica: [[Arquitetura#1. Armazenamento Físico e SQLite]]`,
    updated_at: Math.floor(Date.now() / 1000) - 7200,
    tags: ['inicio', 'boasvindas', 'tutorial'],
    outgoing_links: ['Arquitetura'],
    backlinks: ['Arquitetura', 'Guia de Uso'],
  },
  {
    id: 'guia-de-uso',
    path: 'Guia de Uso.md',
    title: 'Guia de Uso e Operação',
    summary: 'Manual prático para criar notas, navegar no grafo e integrar com assistentes de IA via MCP.',
    content: `# Guia de Uso e Operação\n\nAprenda a extrair o máximo do GraphFinder com fluxos de trabalho otimizados para economia de tokens.\n\n## 1. Criando e Conectando Notas\nPara ligar uma ideia a outra, digite \`[[Nome da Nota]]\` em qualquer lugar do texto.\nO grafo atualizará automaticamente em tempo real.\n\n## 2. Usando via MCP com Agentes de IA\nSeu agente de IA pode consultar o cofre utilizando as ferramentas MCP registradas.\nAs ferramentas retornam deep links no formato \`[[Nota#Seção]]\` para poupar tokens de contexto.\n\n## 3. Auto-Linker Semântico\nAo editar uma nota, clique em **Auto-Linker** para que a IA analise a afinidade do texto com outras notas já existentes no cofre.\n\nTags: #guia #tutorial #mcp #ia\n\n- Conexão Semântica: [[Arquitetura#3. Protocolo MCP]]`,
    updated_at: Math.floor(Date.now() / 1000) - 10800,
    tags: ['guia', 'tutorial', 'mcp', 'ia'],
    outgoing_links: ['Arquitetura'],
    backlinks: ['Arquitetura', 'Boas-vindas'],
  },
  {
    id: 'conceitos-de-ia-e-rag',
    path: 'Conceitos de IA e RAG.md',
    title: 'Conceitos de IA e RAG Atômico',
    summary: 'Fundamentos de Retrieval-Augmented Generation atômico, chunking semântico e economia de contexto.',
    content: `# Conceitos de IA e RAG Atômico\n\nComo o GraphFinder transforma markdown convencional em uma base de conhecimento hiper-eficiente para LLMs.\n\n## 1. Chunking Atômico por Cabeçalhos\nAo invés de vetorizar documentos inteiros, o GraphFinder divide o arquivo por cabeçalhos \`##\` e \`###\` gerando vetores independentes de 40 a 150 tokens.\n\n## 2. Deep Linking Inteligente\nQuando o agente de IA faz uma busca semântica, ele recebe apenas a subseção exata necessária acompanhada do deep link \`[[Nota#Seção]]\`.\n\nTags: #ia #rag #embeddings #arquitetura`,
    updated_at: Math.floor(Date.now() / 1000) - 14400,
    tags: ['ia', 'rag', 'embeddings', 'arquitetura'],
    outgoing_links: ['Arquitetura'],
    backlinks: [],
  },
];

export async function fetchGraph(): Promise<GraphData> {
  if (isTauri) {
    return await invoke<GraphData>('get_graph');
  }
  return {
    nodes: [
      { id: 'arquitetura', title: 'Arquitetura do Sistema', tags: ['arquitetura', 'rust', 'sqlite'], degree: 4 },
      { id: 'boas-vindas', title: 'Boas-vindas ao GraphFinder', tags: ['inicio', 'boasvindas'], degree: 3 },
      { id: 'guia-de-uso', title: 'Guia de Uso e Operação', tags: ['guia', 'tutorial', 'mcp'], degree: 3 },
      { id: 'conceitos-de-ia-e-rag', title: 'Conceitos de IA e RAG Atômico', tags: ['ia', 'rag', 'arquitetura'], degree: 2 },
      { id: 'fastembed-local', title: 'Motor FastEmbed ONNX', tags: ['rust', 'embeddings'], degree: 2 },
      { id: 'sqlite-wal', title: 'Armazenamento SQLite WAL', tags: ['sqlite', 'arquitetura'], degree: 2 },
    ],
    links: [
      { source: 'arquitetura', target: 'boas-vindas', label: 'contexto' },
      { source: 'arquitetura', target: 'guia-de-uso', label: 'instruções' },
      { source: 'conceitos-de-ia-e-rag', target: 'arquitetura', label: 'implementa' },
      { source: 'guia-de-uso', target: 'boas-vindas', label: 'onboarding' },
      { source: 'fastembed-local', target: 'arquitetura', label: 'motor' },
      { source: 'sqlite-wal', target: 'arquitetura', label: 'persistencia' },
      { source: 'conceitos-de-ia-e-rag', target: 'fastembed-local', label: 'vetoriza' },
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
      note_id: 'arquitetura',
      note_title: 'Arquitetura do Sistema',
      heading: '2. Motor Semântico em Rust',
      deep_link: '[[Arquitetura#2. Motor Semântico em Rust]]',
      content: 'Utiliza FastEmbed para calcular representações vetoriais semânticas via ONNX Runtime local sem enviar dados para a nuvem.',
      token_estimate: 48,
      score: 0.94,
    },
    {
      chunk_id: 'chk-2',
      note_id: 'conceitos-de-ia-e-rag',
      note_title: 'Conceitos de IA e RAG Atômico',
      heading: '1. Chunking Atômico por Cabeçalhos',
      deep_link: '[[Conceitos de IA e RAG Atômico#1. Chunking Atômico]]',
      content: 'Ao invés de vetorizar documentos inteiros, divide o arquivo por cabeçalhos gerando vetores independentes de 40 a 150 tokens.',
      token_estimate: 56,
      score: 0.89,
    },
    {
      chunk_id: 'chk-3',
      note_id: 'guia-de-uso',
      note_title: 'Guia de Uso e Operação',
      heading: '2. Usando via MCP com Agentes de IA',
      deep_link: '[[Guia de Uso#2. Usando via MCP com Agentes de IA]]',
      content: 'As ferramentas retornam deep links no formato [[Nota#Seção]] para poupar tokens de contexto na janela de atenção da LLM.',
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
      target_note_id: 'arquitetura',
      target_title: 'Arquitetura do Sistema',
      heading: '1. Armazenamento Físico e SQLite',
      deep_link: '[[Arquitetura#1. Armazenamento Físico e SQLite]]',
      similarity_score: 0.89,
      preview: 'SQLite Local: Guarda a tabela de nós, arestas e vetores de embedding calculados localmente.',
    },
    {
      target_note_id: 'conceitos-de-ia-e-rag',
      target_title: 'Conceitos de IA e RAG Atômico',
      heading: 'Chunking Atômico',
      deep_link: '[[Conceitos de IA e RAG Atômico#Chunking Atômico]]',
      similarity_score: 0.82,
      preview: 'Segmentação atômica por seções para conservação de tokens.',
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
      "command": "graphfinder",
      "args": ["--mcp"],
      "env": {
        "GRAPHFINDER_VAULT": "./vault",
        "GRAPHFINDER_DB": "./index.db"
      }
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

export async function createProject(name: string): Promise<ProjectInfo> {
  if (isTauri) {
    return await invoke<ProjectInfo>('create_project', { name });
  }
  return {
    id: name.toLowerCase(),
    name,
    path: `./vaults/${name.toLowerCase()}`,
    db_path: `./vaults/${name.toLowerCase()}/index.db`,
    note_count: 1,
    is_active: true,
  };
}

