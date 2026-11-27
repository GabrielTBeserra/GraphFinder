import client0 from './client/overview';
import client1 from './client/install';
import client2 from './client/quickstart';
import client3 from './client/desktop';
import client4 from './client/notes';
import client5 from './client/search';
import client6 from './client/code';
import client7 from './client/analytics';
import client8 from './client/cli';
import client9 from './client/mcp';
import client10 from './client/tools';
import client11 from './client/agents';
import client12 from './client/projects';
import client13 from './client/config';
import client14 from './client/architecture';
import client15 from './client/security';
import client16 from './client/development';
import client17 from './client/faq';

export type ManualSection = { id: string; label: string; html: string };

export const clientSections: ManualSection[] = [
  { id: 'overview', label: "Overview", html: client0 },
  { id: 'install', label: "Installation", html: client1 },
  { id: 'quickstart', label: "Quick start", html: client2 },
  { id: 'desktop', label: "Desktop app", html: client3 },
  { id: 'notes', label: "Writing notes", html: client4 },
  { id: 'search', label: "Search engine", html: client5 },
  { id: 'code', label: "Code indexing", html: client6 },
  { id: 'analytics', label: "Graph analytics", html: client7 },
  { id: 'cli', label: "CLI reference", html: client8 },
  { id: 'mcp', label: "MCP server setup", html: client9 },
  { id: 'tools', label: "MCP tools", html: client10 },
  { id: 'agents', label: "AI agents & skills", html: client11 },
  { id: 'projects', label: "Projects & vaults", html: client12 },
  { id: 'config', label: "Configuration", html: client13 },
  { id: 'architecture', label: "Architecture", html: client14 },
  { id: 'security', label: "Security", html: client15 },
  { id: 'development', label: "Development", html: client16 },
  { id: 'faq', label: "Troubleshooting & FAQ", html: client17 },
];
