export interface GraphNode {
  id: string;
  title: string;
  tags: string[];
  degree: number;
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
