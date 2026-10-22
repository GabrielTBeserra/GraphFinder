export interface GraphNode {
  id: string;
  title: string;
  tags: string[];
  degree: number;
  node_type?: 'note' | 'code' | 'unresolved';
  path?: string;
  x?: number;
  y?: number;
  vx?: number;
  vy?: number;
  color?: string;
}

export interface GraphLink {
  source: string | GraphNode;
  target: string | GraphNode;
  label?: string;
}

export interface GraphData {
  nodes: GraphNode[];
  links: GraphLink[];
}

export interface ProjectInfo {
  id: string;
  name: string;
  path: string;
  db_path: string;
  note_count: number;
  is_active: boolean;
}

export interface NoteRecord {
  id: string;
  path: string;
  title: string;
  summary: string;
  content: string;
  updated_at: number;
  tags: string[];
  outgoing_links: string[];
  backlinks: string[];
}

export interface NoteSummary {
  id: string;
  path: string;
  title: string;
  summary: string;
  updated_at: number;
  tags: string[];
}

export interface SearchHit {
  id: string;
  title: string;
  snippet: string;
  score: number;
  tags: string[];
}

export interface ChunkHit {
  chunk_id: string;
  note_id: string;
  note_title: string;
  heading: string | null;
  deep_link: string;
  content: string;
  token_estimate: number;
  score: number;
}

export interface LinkSuggestion {
  target_note_id: string;
  target_title: string;
  heading: string | null;
  deep_link: string;
  similarity_score: number;
  preview: string;
}

export interface SectionSummary {
  heading: string;
  token_estimate: number;
  snippet: string;
}

export interface CompressedContext {
  note_id: string;
  title: string;
  summary: string;
  tags: string[];
  token_estimate: number;
  connected_notes: string[];
  sections: SectionSummary[];
}

export interface Neighborhood {
  center: NoteRecord | null;
  nodes: GraphNode[];
  links: GraphLink[];
}

export interface VaultStats {
  total_notes: number;
  total_chunks: number;
  total_links: number;
  total_tags: number;
  total_tokens: number;
  total_code_files?: number;
  total_code_chunks?: number;
  total_code_links?: number;
  languages?: string[];
}

export interface CodeFileRecord {
  id: string;
  path: string;
  name: string;
  extension: string;
  language: string;
  size_bytes: number;
  summary: string;
  updated_at: number;
  imports_count: number;
  dependents_count: number;
}

export interface CodeChunkHit {
  chunk_id: string;
  file_id: string;
  file_name: string;
  language: string;
  symbol_name: string | null;
  start_line: number;
  end_line: number;
  deep_link: string;
  content: string;
  token_estimate: number;
  score: number;
}

export interface SymbolInfo {
  name: string;
  start_line: number;
  end_line: number;
  token_estimate: number;
}

export interface FileContext {
  file_id: string;
  name: string;
  language: string;
  size_bytes: number;
  summary: string;
  token_estimate: number;
  imported_files: string[];
  dependent_files: string[];
  external_packages: string[];
  symbols: SymbolInfo[];
}

export interface CodeDependencies {
  file_id: string;
  imports: string[];
  imported_by: string[];
  external: string[];
}


