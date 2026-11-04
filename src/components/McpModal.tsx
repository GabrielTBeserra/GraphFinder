import React, { useState, useEffect, useMemo } from 'react';
import * as Dialog from '@radix-ui/react-dialog';
import * as Tabs from '@radix-ui/react-tabs';
import {
  Robot,
  Copy,
  Check,
  X,
  Cpu,
  BookOpen,
  FileText,
  ShieldCheck,
  WarningCircle,
  Lightning,
  Wrench,
  Pulse,
  MagnifyingGlass,
  LockKey,
  Trash,
} from '@phosphor-icons/react';
import {
  AGENT_RULE_CONTENT,
  SKILL_CONTENT,
  MCP_CONFIG_SNIPPET,
  MCP_CURSOR_CONFIG_SNIPPET,
  MCP_CLAUDE_CONFIG_SNIPPET,
} from '../services/aiConfig';
import {
  fetchMcpConfig,
  fetchMcpTools,
  fetchMcpActivity,
  type McpConfigInfo,
  type McpToolInfo,
  type McpCallEntry,
} from '../services/api';

interface McpModalProps {
  isOpen: boolean;
  onClose: () => void;
  theme?: 'dark' | 'light';
}

type ClientTab = 'cursor' | 'claude' | 'universal';

const TOOL_GROUPS: [string, string[]][] = [
  ['Semantic Vault', ['search_semantic', 'get_compressed_context', 'read_section', 'store_context', 'update_section', 'suggest_links', 'link_notes', 'get_graph_neighborhood', 'list_recent_notes', 'rename_note', 'delete_note', 'get_vault_stats', 'validate_vault']],
  ['Code Intelligence', ['search_code', 'find_files', 'get_file_context', 'read_code_chunk', 'get_code_dependencies']],
  ['Topological Audit', ['get_community_report', 'explain_node', 'find_path', 'export_graph']],
  ['Projects & Sync', ['list_projects', 'switch_project', 'create_project', 'sync_vault']],
];

function groupOf(name: string): string {
  for (const [group, names] of TOOL_GROUPS) {
    if (names.includes(name)) return group;
  }
  return 'Other';
}

function formatTime(ts: number): string {
  const d = new Date(ts);
  return d.toLocaleTimeString(undefined, { hour12: false });
}

export const McpModal: React.FC<McpModalProps> = ({ isOpen, onClose, theme = 'dark' }) => {
  const isLight = theme === 'light';
  const [copiedType, setCopiedType] = useState<string | null>(null);
  const [mcpInfo, setMcpInfo] = useState<McpConfigInfo | null>(null);
  const [activeClient, setActiveClient] = useState<ClientTab>('cursor');
  const [isServerHealthy, setIsServerHealthy] = useState<boolean | null>(null);
  const [tools, setTools] = useState<McpToolInfo[]>([]);
  const [toolFilter, setToolFilter] = useState('');
  const [activity, setActivity] = useState<McpCallEntry[]>([]);

  const mcpUrl = 'http://127.0.0.1:45123/mcp';

  useEffect(() => {
    if (!isOpen) return;
    fetchMcpConfig()
      .then((info) => setMcpInfo(info))
      .catch(() => {});
    fetch('http://127.0.0.1:45123/health')
      .then((res) => setIsServerHealthy(res.ok))
      .catch(() => setIsServerHealthy(false));
    fetchMcpTools().then(setTools);

    let cancelled = false;
    const poll = async () => {
      const calls = await fetchMcpActivity();
      if (!cancelled) setActivity(calls);
    };
    poll();
    const interval = setInterval(poll, 2500);
    return () => {
      cancelled = true;
      clearInterval(interval);
    };
  }, [isOpen]);

  const currentConfigJson = mcpInfo?.config_json || MCP_CONFIG_SNIPPET;

  const getClientSnippet = (client: ClientTab) => {
    switch (client) {
      case 'cursor':
        return MCP_CURSOR_CONFIG_SNIPPET;
      case 'claude':
        return MCP_CLAUDE_CONFIG_SNIPPET;
      case 'universal':
        return currentConfigJson;
    }
  };

  const copyToClipboard = (text: string, type: string) => {
    navigator.clipboard.writeText(text);
    setCopiedType(type);
    setTimeout(() => setCopiedType(null), 2500);
  };

  const filteredTools = useMemo(() => {
    const q = toolFilter.trim().toLowerCase();
    const list = q
      ? tools.filter((t) => t.name.includes(q) || (t.description || '').toLowerCase().includes(q))
      : tools;
    const grouped = new Map<string, McpToolInfo[]>();
    for (const t of list) {
      const g = groupOf(t.name);
      if (!grouped.has(g)) grouped.set(g, []);
      grouped.get(g)!.push(t);
    }
    return grouped;
  }, [tools, toolFilter]);

  const copyBtn = (text: string, type: string, label: string) => (
    <button
      type="button"
      onClick={() => copyToClipboard(text, type)}
      className="flex items-center justify-center gap-1.5 px-3 py-1.5 text-[11px] font-medium rounded-md bg-raise border border-line hover:border-line-strong hover:text-ink text-muted transition-colors cursor-pointer"
    >
      {copiedType === type ? (
        <>
          <Check size={12} weight="bold" className="text-good" />
          <span className="text-good">Copied</span>
        </>
      ) : (
        <>
          <Copy size={12} />
          <span>{label}</span>
        </>
      )}
    </button>
  );

  const tabCls =
    'flex items-center gap-1.5 px-2.5 py-1.5 rounded-md text-[11px] font-medium text-muted hover:text-ink data-[state=active]:bg-raise data-[state=active]:text-ink transition-colors cursor-pointer outline-none';

  return (
    <Dialog.Root open={isOpen} onOpenChange={(open) => !open && onClose()}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 z-50 bg-black/50 backdrop-blur-sm" />
        <Dialog.Content className="fixed left-1/2 top-1/2 z-50 w-full max-w-2xl max-h-[90vh] -translate-x-1/2 -translate-y-1/2 rounded-xl bg-surface border border-line p-5 flex flex-col focus:outline-none overflow-hidden text-ink">
          {/* Header */}
          <div className="flex items-start justify-between gap-4 pb-3 border-b border-line">
            <div className="flex items-center gap-3">
              <div className="flex items-center justify-center w-9 h-9 rounded-lg bg-accent/10 border border-accent/25 text-accent">
                <Robot size={19} weight="duotone" />
              </div>
              <div>
                <Dialog.Title className="text-sm font-semibold text-ink flex items-center gap-2">
                  <span>AI Agent Hub</span>
                  <span className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-[9.5px] font-mono bg-good/10 border border-good/25 text-good">
                    <span className="w-1.5 h-1.5 rounded-full bg-good" />
                    Streamable HTTP
                  </span>
                </Dialog.Title>
                <Dialog.Description className="text-[11px] text-muted mt-0.5">
                  MCP server, tools, and agent rules — shared SQLite memory across all clients.
                </Dialog.Description>
              </div>
            </div>
            <Dialog.Close asChild>
              <button
                type="button"
                className="rounded-md p-1.5 text-faint hover:text-ink hover:bg-raise transition-colors cursor-pointer"
                title="Close"
              >
                <X size={15} />
              </button>
            </Dialog.Close>
          </div>

          {/* Quick actions */}
          <div className="grid grid-cols-3 gap-2 py-3">
            {copyBtn(AGENT_RULE_CONTENT, 'rule-quick', 'Copy AGENTS.md')}
            {copyBtn(SKILL_CONTENT, 'skill-quick', 'Copy SKILL.md')}
            {copyBtn(currentConfigJson, 'mcp-quick', 'Copy MCP JSON')}
          </div>

          <Tabs.Root defaultValue="connect" className="flex flex-col flex-1 min-h-0">
            <Tabs.List className="flex items-center gap-1 border-b border-line pb-2">
              <Tabs.Trigger value="connect" className={tabCls}>
                <Cpu size={13} className="text-accent" />
                <span>Connect</span>
              </Tabs.Trigger>
              <Tabs.Trigger value="tools" className={tabCls}>
                <Wrench size={13} className="text-accent" />
                <span>Tools ({tools.length || 26})</span>
              </Tabs.Trigger>
              <Tabs.Trigger value="activity" className={tabCls}>
                <Pulse size={13} className="text-good" />
                <span>Activity</span>
              </Tabs.Trigger>
              <Tabs.Trigger value="rule" className={tabCls}>
                <FileText size={13} className="text-accent" />
                <span>Rules</span>
              </Tabs.Trigger>
              <Tabs.Trigger value="skill" className={tabCls}>
                <BookOpen size={13} className="text-accent" />
                <span>Skill</span>
              </Tabs.Trigger>
            </Tabs.List>

            {/* CONNECT */}
            <Tabs.Content value="connect" className="flex-1 overflow-y-auto space-y-3 pt-3 outline-none pr-1">
              <div className="flex items-center justify-between p-3 rounded-lg bg-raise border border-line">
                <div className="flex items-center gap-2.5">
                  <div className="w-2 h-2 rounded-full bg-good" />
                  <div>
                    <div className="text-xs font-semibold text-ink flex items-center gap-2">
                      <span>Server Online</span>
                      <span className="font-mono text-[10px] text-good">:45123</span>
                      {isServerHealthy !== null && (
                        <span
                          className={`text-[9px] px-1.5 py-0.5 rounded font-mono font-semibold ${
                            isServerHealthy
                              ? 'text-good bg-good/10 border border-good/25'
                              : 'text-warn bg-warn/10 border border-warn/25'
                          }`}
                        >
                          {isServerHealthy ? 'HEALTHY' : 'CONNECTING'}
                        </span>
                      )}
                    </div>
                    <div className="text-[10.5px] text-muted font-mono mt-0.5">{mcpUrl}</div>
                  </div>
                </div>
                {copyBtn(mcpUrl, 'url-copy', 'Copy URL')}
              </div>

              <div className="rounded-lg bg-warn/10 border border-warn/25 p-3 text-[11px] text-warn space-y-1">
                <div className="flex items-center gap-1.5 font-semibold">
                  <WarningCircle size={14} weight="bold" />
                  <span>HTTP transport only</span>
                </div>
                <p className="leading-relaxed opacity-90">
                  Stdio is discontinued. GraphFinder serves Streamable HTTP on <code>/mcp</code>, sharing the ONNX cache and reflecting writes in the graph live.
                </p>
              </div>

              <div>
                <div className="flex items-center justify-between text-[11px] text-muted mb-1.5">
                  <span className="font-medium text-ink flex items-center gap-1.5">
                    <Lightning size={12} className="text-accent" />
                    Client configuration:
                  </span>
                  {copyBtn(getClientSnippet(activeClient), 'mcp-config-tab', 'Copy Config')}
                </div>
                <div className="flex items-center gap-1 p-1 rounded-lg bg-canvas border border-line mb-2">
                  {(['cursor', 'claude', 'universal'] as ClientTab[]).map((c) => (
                    <button
                      key={c}
                      type="button"
                      onClick={() => setActiveClient(c)}
                      className={`flex-1 py-1.5 px-2 rounded-md text-[11px] font-medium transition-colors cursor-pointer ${
                        activeClient === c
                          ? 'bg-accent/15 text-accent border border-accent/30 font-semibold'
                          : 'text-muted hover:text-ink hover:bg-raise'
                      }`}
                    >
                      {c === 'cursor' ? 'Cursor' : c === 'claude' ? 'Claude Desktop' : 'Universal'}
                    </button>
                  ))}
                </div>
                <pre className="font-mono text-[10.5px] leading-relaxed p-3 rounded-lg bg-canvas border border-line text-accent overflow-x-auto max-h-40">
                  {getClientSnippet(activeClient)}
                </pre>
              </div>
            </Tabs.Content>

            {/* TOOLS */}
            <Tabs.Content value="tools" className="flex-1 overflow-y-auto space-y-3 pt-3 outline-none pr-1">
              <div className="relative">
                <MagnifyingGlass size={12} className="absolute left-2.5 top-1/2 -translate-y-1/2 text-faint" />
                <input
                  type="text"
                  value={toolFilter}
                  onChange={(e) => setToolFilter(e.target.value)}
                  placeholder={`Filter ${tools.length} tools...`}
                  className="w-full pl-8 pr-3 py-1.5 text-[11px] rounded-md bg-canvas border border-line text-ink placeholder-faint focus:outline-none focus:border-accent/50"
                />
              </div>
              <div className="flex items-center gap-3 text-[9.5px] font-mono text-faint px-1">
                <span className="flex items-center gap-1">
                  <LockKey size={10} className="text-good" /> RO = read-only
                </span>
                <span className="flex items-center gap-1">
                  <Trash size={10} className="text-bad" /> destructive
                </span>
              </div>
              {filteredTools.size === 0 && (
                <div className="py-10 text-center text-[11px] text-faint">
                  {tools.length === 0 ? 'Server unreachable — start the app to list tools.' : 'No tools match.'}
                </div>
              )}
              {Array.from(filteredTools.entries()).map(([group, items]) => (
                <div key={group}>
                  <div className="px-1 pt-1 pb-1.5 text-[9.5px] font-mono uppercase tracking-[0.12em] text-faint">
                    {group} · {items.length}
                  </div>
                  <div className="space-y-1">
                    {items.map((t) => (
                      <div
                        key={t.name}
                        className="flex items-start gap-2.5 px-2.5 py-1.5 rounded-md border border-line bg-raise/50 hover:bg-raise transition-colors"
                      >
                        <code className="text-[11px] font-mono font-semibold text-ink shrink-0 pt-px">
                          {t.name}
                        </code>
                        <div className="flex-1 min-w-0">
                          {t.description && (
                            <p className="text-[10.5px] text-muted leading-snug truncate" title={t.description}>
                              {t.description}
                            </p>
                          )}
                        </div>
                        <div className="flex items-center gap-1 shrink-0">
                          {t.annotations.readOnly && (
                            <span className="inline-flex items-center gap-0.5 px-1 py-0.5 rounded text-[8.5px] font-mono bg-good/10 border border-good/25 text-good">
                              <LockKey size={8} />
                              RO
                            </span>
                          )}
                          {t.annotations.destructive && (
                            <span className="inline-flex items-center gap-0.5 px-1 py-0.5 rounded text-[8.5px] font-mono bg-bad/10 border border-bad/25 text-bad">
                              <Trash size={8} />
                              DEL
                            </span>
                          )}
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              ))}
            </Tabs.Content>

            {/* ACTIVITY */}
            <Tabs.Content value="activity" className="flex-1 overflow-y-auto pt-3 outline-none pr-1">
              <div className="flex items-center justify-between mb-2 px-1">
                <span className="text-[11px] text-muted flex items-center gap-1.5">
                  <Pulse size={12} className="text-good" />
                  Live tool calls (last 60)
                </span>
                <span className="text-[9.5px] font-mono text-faint">polls every 2.5s</span>
              </div>
              {activity.length === 0 ? (
                <div className="py-10 text-center text-[11px] text-faint">
                  No tool calls recorded yet. Connect an MCP client and run a tool.
                </div>
              ) : (
                <div className="space-y-0.5">
                  {activity.map((c, i) => (
                    <div
                      key={`${c.ts_ms}-${i}`}
                      className="flex items-center gap-2.5 px-2.5 py-1.5 rounded-md text-[10.5px] font-mono hover:bg-raise transition-colors"
                    >
                      <span className="text-faint shrink-0">{formatTime(c.ts_ms)}</span>
                      <span
                        className={`px-1 py-0.5 rounded text-[8.5px] font-semibold shrink-0 ${
                          c.status < 400
                            ? 'bg-good/10 text-good'
                            : 'bg-bad/10 text-bad'
                        }`}
                      >
                        {c.status}
                      </span>
                      <span className="text-accent truncate">{c.tool || c.method}</span>
                      <span className="text-faint ml-auto shrink-0">{c.duration_ms}ms</span>
                    </div>
                  ))}
                </div>
              )}
            </Tabs.Content>

            {/* RULES */}
            <Tabs.Content value="rule" className="flex-1 overflow-y-auto space-y-3 pt-3 outline-none pr-1">
              <div className="rounded-lg bg-raise border border-line p-3 text-[11px] text-muted">
                <div className="flex items-center gap-1.5 font-semibold text-ink mb-1">
                  <ShieldCheck size={13} className="text-good" />
                  <span>How to use:</span>
                </div>
                <p className="leading-relaxed">
                  Paste into <code>AGENTS.md</code> or <code>.cursorrules</code> at your workspace root, or in your editor's custom instructions.
                </p>
              </div>
              <div className="flex items-center justify-between text-[11px]">
                <span className="font-medium text-ink">AGENTS.md</span>
                {copyBtn(AGENT_RULE_CONTENT, 'rule-tab', 'Copy Text')}
              </div>
              <pre className="font-mono text-[10.5px] leading-relaxed p-3 rounded-lg bg-canvas border border-line text-muted overflow-x-auto max-h-56">
                {AGENT_RULE_CONTENT}
              </pre>
            </Tabs.Content>

            {/* SKILL */}
            <Tabs.Content value="skill" className="flex-1 overflow-y-auto space-y-3 pt-3 outline-none pr-1">
              <div className="rounded-lg bg-raise border border-line p-3 text-[11px] text-muted">
                <div className="flex items-center gap-1.5 font-semibold text-ink mb-1">
                  <BookOpen size={13} className="text-accent" />
                  <span>How to install:</span>
                </div>
                <p className="leading-relaxed">
                  Save into <code>.agents/skills/graphfinder-curator/SKILL.md</code>. Compatible agents will discover and load it on demand.
                </p>
              </div>
              <div className="flex items-center justify-between text-[11px]">
                <span className="font-medium text-ink">SKILL.md</span>
                {copyBtn(SKILL_CONTENT, 'skill-tab', 'Copy Text')}
              </div>
              <pre className="font-mono text-[10.5px] leading-relaxed p-3 rounded-lg bg-canvas border border-line text-muted overflow-x-auto max-h-56">
                {SKILL_CONTENT}
              </pre>
            </Tabs.Content>
          </Tabs.Root>

          <div className="flex items-center justify-between pt-3 border-t border-line text-[9.5px] font-mono text-faint">
            <span>GraphFinder MCP · rmcp SDK</span>
            <span>{isLight ? 'light' : 'dark'} theme</span>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
};
