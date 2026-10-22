import React, { useState, useEffect } from 'react';
import * as Dialog from '@radix-ui/react-dialog';
import * as Tabs from '@radix-ui/react-tabs';
import {
  Robot,
  Copy,
  Check,
  X,
  TerminalWindow,
  Cpu,
  BookOpen,
  FileText,
  CheckCircle,
  Link,
  ShieldCheck,
  WarningCircle,
  Lightning,
} from '@phosphor-icons/react';
import {
  AGENT_RULE_CONTENT,
  SKILL_CONTENT,
  MCP_CONFIG_SNIPPET,
  MCP_CURSOR_CONFIG_SNIPPET,
  MCP_CLAUDE_CONFIG_SNIPPET,
} from '../services/aiConfig';
import { fetchMcpConfig, type McpConfigInfo } from '../services/api';

interface McpModalProps {
  isOpen: boolean;
  onClose: () => void;
}

type ClientTab = 'cursor' | 'claude' | 'antigravity' | 'url';

export const McpModal: React.FC<McpModalProps> = ({ isOpen, onClose }) => {
  const [copiedType, setCopiedType] = useState<string | null>(null);
  const [mcpInfo, setMcpInfo] = useState<McpConfigInfo | null>(null);
  const [activeClient, setActiveClient] = useState<ClientTab>('cursor');
  const [isServerHealthy, setIsServerHealthy] = useState<boolean | null>(null);

  const mcpUrl = 'http://127.0.0.1:45123/mcp';

  useEffect(() => {
    if (isOpen) {
      fetchMcpConfig()
        .then((info) => setMcpInfo(info))
        .catch((err) => console.error('Error detecting MCP config:', err));

      // Health ping
      fetch('http://127.0.0.1:45123/health')
        .then((res) => setIsServerHealthy(res.ok))
        .catch(() => setIsServerHealthy(false));
    }
  }, [isOpen]);

  const currentConfigJson = mcpInfo?.config_json || MCP_CONFIG_SNIPPET;

  const getClientSnippet = (client: ClientTab) => {
    switch (client) {
      case 'cursor':
        return MCP_CURSOR_CONFIG_SNIPPET;
      case 'claude':
        return MCP_CLAUDE_CONFIG_SNIPPET;
      case 'antigravity':
        return currentConfigJson;
      case 'url':
        return mcpUrl;
    }
  };

  const copyToClipboard = (text: string, type: string) => {
    navigator.clipboard.writeText(text);
    setCopiedType(type);
    setTimeout(() => setCopiedType(null), 2500);
  };

  return (
    <Dialog.Root open={isOpen} onOpenChange={(open) => !open && onClose()}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 z-50 bg-black/80 backdrop-blur-md" />
        <Dialog.Content className="fixed left-1/2 top-1/2 z-50 w-full max-w-2xl max-h-[92vh] -translate-x-1/2 -translate-y-1/2 rounded-2xl bg-[#0f1117] border border-[#232838] p-6 shadow-2xl flex flex-col focus:outline-none overflow-hidden text-[#f1f5f9]">
          {/* Header */}
          <div className="flex items-start justify-between gap-4 pb-4 border-b border-[#1f2433]">
            <div className="flex items-center gap-3">
              <div className="flex items-center justify-center w-9 h-9 rounded-xl bg-indigo-500/10 border border-indigo-500/20 text-indigo-400">
                <Robot size={20} weight="duotone" />
              </div>
              <div>
                <Dialog.Title className="text-base font-semibold text-[#f8fafc] flex items-center gap-2">
                  <span>AI Agent Hub: MCP, Rules &amp; Skills</span>
                  <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-medium bg-emerald-500/10 border border-emerald-500/25 text-emerald-400 font-mono">
                    <span className="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse" />
                    Streamable HTTP Only
                  </span>
                </Dialog.Title>
                <Dialog.Description className="text-xs text-[#64748b] mt-0.5">
                  Universal protocol for Cursor, Claude, Antigravity, and AI Agents with zero-latency SQLite shared memory.
                </Dialog.Description>
              </div>
            </div>

            <Dialog.Close asChild>
              <button
                type="button"
                className="rounded-lg p-1.5 text-[#64748b] hover:text-[#f8fafc] hover:bg-[#1a1f2e] transition-colors cursor-pointer"
                title="Close"
              >
                <X size={16} />
              </button>
            </Dialog.Close>
          </div>

          {/* Quick Copy Action Bar */}
          <div className="grid grid-cols-3 gap-2 py-3">
            <button
              type="button"
              onClick={() => copyToClipboard(AGENT_RULE_CONTENT, 'rule-quick')}
              className="flex items-center justify-center gap-2 px-3 py-2 text-xs font-medium rounded-xl bg-[#141824] border border-[#202638] hover:bg-[#1c2233] hover:border-indigo-500/40 text-[#cbd5e1] transition-all cursor-pointer shadow-sm"
            >
              {copiedType === 'rule-quick' ? (
                <>
                  <Check size={14} weight="bold" className="text-emerald-400" />
                  <span className="text-emerald-400">Rule Copied!</span>
                </>
              ) : (
                <>
                  <FileText size={14} className="text-indigo-400" />
                  <span>Copy AGENTS.md</span>
                </>
              )}
            </button>

            <button
              type="button"
              onClick={() => copyToClipboard(SKILL_CONTENT, 'skill-quick')}
              className="flex items-center justify-center gap-2 px-3 py-2 text-xs font-medium rounded-xl bg-[#141824] border border-[#202638] hover:bg-[#1c2233] hover:border-indigo-500/40 text-[#cbd5e1] transition-all cursor-pointer shadow-sm"
            >
              {copiedType === 'skill-quick' ? (
                <>
                  <Check size={14} weight="bold" className="text-emerald-400" />
                  <span className="text-emerald-400">Skill Copied!</span>
                </>
              ) : (
                <>
                  <BookOpen size={14} className="text-purple-400" />
                  <span>Copy SKILL.md</span>
                </>
              )}
            </button>

            <button
              type="button"
              onClick={() => copyToClipboard(currentConfigJson, 'mcp-quick')}
              className="flex items-center justify-center gap-2 px-3 py-2 text-xs font-medium rounded-xl bg-[#141824] border border-[#202638] hover:bg-[#1c2233] hover:border-indigo-500/40 text-[#cbd5e1] transition-all cursor-pointer shadow-sm"
            >
              {copiedType === 'mcp-quick' ? (
                <>
                  <Check size={14} weight="bold" className="text-emerald-400" />
                  <span className="text-emerald-400">Config Copied!</span>
                </>
              ) : (
                <>
                  <TerminalWindow size={14} className="text-emerald-400" />
                  <span>Copy MCP JSON</span>
                </>
              )}
            </button>
          </div>

          {/* Radix Tabs Section */}
          <Tabs.Root defaultValue="mcp" className="flex flex-col flex-1 min-h-0">
            <Tabs.List className="flex items-center gap-1.5 border-b border-[#1f2433] pb-2">
              <Tabs.Trigger
                value="mcp"
                className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium text-[#64748b] hover:text-[#cbd5e1] data-[state=active]:bg-[#1a1f2e] data-[state=active]:text-[#f8fafc] data-[state=active]:border data-[state=active]:border-indigo-500/30 transition-all cursor-pointer outline-none"
              >
                <Cpu size={14} className="text-indigo-400" />
                <span>MCP Server (/mcp)</span>
              </Tabs.Trigger>

              <Tabs.Trigger
                value="rule"
                className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium text-[#64748b] hover:text-[#cbd5e1] data-[state=active]:bg-[#1a1f2e] data-[state=active]:text-[#f8fafc] data-[state=active]:border data-[state=active]:border-indigo-500/30 transition-all cursor-pointer outline-none"
              >
                <FileText size={14} className="text-blue-400" />
                <span>Agent Rules (AGENTS.md)</span>
              </Tabs.Trigger>

              <Tabs.Trigger
                value="skill"
                className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium text-[#64748b] hover:text-[#cbd5e1] data-[state=active]:bg-[#1a1f2e] data-[state=active]:text-[#f8fafc] data-[state=active]:border data-[state=active]:border-indigo-500/30 transition-all cursor-pointer outline-none"
              >
                <BookOpen size={14} className="text-purple-400" />
                <span>Curator Skill (SKILL.md)</span>
              </Tabs.Trigger>
            </Tabs.List>

            {/* Tab 1: MCP Server (Default) */}
            <Tabs.Content value="mcp" className="flex-1 overflow-y-auto space-y-3 pt-3 outline-none pr-1">
              {/* Server Status Header */}
              <div className="flex items-center justify-between p-3 rounded-xl bg-[#141824] border border-[#202638]">
                <div className="flex items-center gap-2.5">
                  <div className="w-2.5 h-2.5 rounded-full bg-emerald-400 shadow-[0_0_10px_rgba(52,211,153,0.6)] animate-pulse" />
                  <div>
                    <div className="text-xs font-semibold text-[#f8fafc] flex items-center gap-2">
                      <span>Streamable HTTP Server Online</span>
                      <span className="font-mono text-[11px] text-emerald-400">PORT 45123</span>
                      {isServerHealthy !== null && (
                        <span
                          className={`text-[9.5px] px-1.5 py-0.5 rounded font-mono font-bold ${
                            isServerHealthy
                              ? 'text-emerald-400 bg-emerald-500/10 border border-emerald-500/20'
                              : 'text-amber-400 bg-amber-500/10 border border-amber-500/20'
                          }`}
                        >
                          {isServerHealthy ? 'HEALTHY' : 'CONNECTING'}
                        </span>
                      )}
                    </div>
                    <div className="text-[11px] text-[#94a3b8] font-mono mt-0.5">
                      {mcpUrl}
                    </div>
                  </div>
                </div>

                <button
                  type="button"
                  onClick={() => copyToClipboard(mcpUrl, 'url-copy')}
                  className="inline-flex items-center gap-1 px-2.5 py-1 text-xs rounded-lg bg-[#1c2233] hover:bg-[#252c42] border border-[#2b334a] text-[#cbd5e1] transition-colors cursor-pointer"
                >
                  {copiedType === 'url-copy' ? (
                    <>
                      <Check size={12} weight="bold" className="text-emerald-400" />
                      <span className="text-emerald-400 font-medium">Copied!</span>
                    </>
                  ) : (
                    <>
                      <Link size={12} />
                      <span>Copy URL</span>
                    </>
                  )}
                </button>
              </div>

              {/* Stdio Prohibition Notice */}
              <div className="rounded-xl bg-amber-500/10 border border-amber-500/20 p-3 text-xs text-amber-200/90 space-y-1">
                <div className="flex items-center gap-1.5 font-semibold text-amber-300">
                  <WarningCircle size={15} weight="bold" />
                  <span>Stdio Discontinued &amp; Prohibited</span>
                </div>
                <p className="text-[11px] text-amber-200/80 leading-relaxed">
                  To ensure maximum performance, STDIO transport has been <strong>discontinued and prohibited</strong>. GraphFinder operates exclusively via <strong>Streamable HTTP</strong> on the <code>/mcp</code> endpoint. This eliminates SQLite file locks, shares the ONNX embedding cache, and reflects agent writes on the visual screen graph in real time!
                </p>
              </div>

              {/* Client Configuration Selector */}
              <div>
                <div className="flex items-center justify-between text-xs text-[#64748b] mb-1.5">
                  <span className="font-medium text-[#cbd5e1] flex items-center gap-1.5">
                    <Lightning size={13} className="text-indigo-400" />
                    Select your editor / client:
                  </span>
                  <button
                    type="button"
                    onClick={() => copyToClipboard(getClientSnippet(activeClient), 'mcp-config-tab')}
                    className="flex items-center gap-1 px-2.5 py-1 rounded-lg bg-[#141824] border border-[#202638] text-[#cbd5e1] hover:text-[#f8fafc] hover:bg-[#1c2233] transition-colors cursor-pointer text-xs"
                  >
                    {copiedType === 'mcp-config-tab' ? (
                      <>
                        <Check size={12} weight="bold" className="text-emerald-400" />
                        <span className="text-emerald-400 font-medium">Copied!</span>
                      </>
                    ) : (
                      <>
                        <Copy size={12} />
                        <span>Copy Configuration</span>
                      </>
                    )}
                  </button>
                </div>

                <div className="flex items-center gap-1 p-1 rounded-xl bg-[#090b10] border border-[#1f2433] mb-2">
                  <button
                    type="button"
                    onClick={() => setActiveClient('cursor')}
                    className={`flex-1 py-1.5 px-2.5 rounded-lg text-xs font-medium transition-all cursor-pointer ${
                      activeClient === 'cursor'
                        ? 'bg-indigo-600/30 text-indigo-300 border border-indigo-500/40 font-semibold shadow-sm'
                        : 'text-[#94a3b8] hover:text-[#f8fafc] hover:bg-[#141824]'
                    }`}
                  >
                    Cursor (.cursor/mcp.json)
                  </button>
                  <button
                    type="button"
                    onClick={() => setActiveClient('claude')}
                    className={`flex-1 py-1.5 px-2.5 rounded-lg text-xs font-medium transition-all cursor-pointer ${
                      activeClient === 'claude'
                        ? 'bg-indigo-600/30 text-indigo-300 border border-indigo-500/40 font-semibold shadow-sm'
                        : 'text-[#94a3b8] hover:text-[#f8fafc] hover:bg-[#141824]'
                    }`}
                  >
                    Claude Desktop
                  </button>
                  <button
                    type="button"
                    onClick={() => setActiveClient('antigravity')}
                    className={`flex-1 py-1.5 px-2.5 rounded-lg text-xs font-medium transition-all cursor-pointer ${
                      activeClient === 'antigravity'
                        ? 'bg-indigo-600/30 text-indigo-300 border border-indigo-500/40 font-semibold shadow-sm'
                        : 'text-[#94a3b8] hover:text-[#f8fafc] hover:bg-[#141824]'
                    }`}
                  >
                    Antigravity / VS Code
                  </button>
                </div>

                <pre className="font-mono text-[11px] leading-relaxed p-3.5 rounded-xl bg-[#08090d] border border-[#1f2433] text-indigo-300 overflow-x-auto max-h-40">
                  {getClientSnippet(activeClient)}
                </pre>
              </div>

              {/* Atomic Tools Capability List */}
              <div className="space-y-1.5 pt-1">
                <div className="flex items-center gap-1.5 text-xs font-semibold text-[#cbd5e1]">
                  <ShieldCheck size={14} className="text-emerald-400" />
                  <span>Available Atomic MCP Tools via Streamable HTTP:</span>
                </div>
                <ul className="text-[11.5px] text-[#94a3b8] space-y-1 pl-4 list-disc leading-relaxed">
                  <li><code>search_semantic(query)</code>: Zero-allocation vector search with <code>[[Note#Section]]</code> deep links.</li>
                  <li><code>get_compressed_context(note_id)</code>: Structural outline and token estimates (~60-100 tokens).</li>
                  <li><code>suggest_links(text)</code>: Real-time semantic auto-linker for drafted paragraphs.</li>
                  <li><code>store_context(title, content, tags)</code>: Saves note with auto-chunking &amp; live graph canvas neon shockwave.</li>
                  <li><code>get_graph_neighborhood(note_id)</code>: Topological adjacency without loading full files.</li>
                  <li><code>link_notes(from, to)</code>: Bidirectional relationship creation in real time.</li>
                </ul>
              </div>
            </Tabs.Content>

            {/* Tab 2: Agent Rules */}
            <Tabs.Content value="rule" className="flex-1 overflow-y-auto space-y-3 pt-3 outline-none pr-1">
              <div className="rounded-xl bg-[#141824] border border-[#202638] p-3 text-xs text-[#cbd5e1]">
                <div className="flex items-center gap-1.5 font-semibold text-[#f8fafc] mb-1">
                  <CheckCircle size={14} className="text-emerald-400" />
                  <span>How to use in your editor:</span>
                </div>
                <p className="text-[11px] text-[#94a3b8] leading-relaxed">
                  Paste this into <code>AGENTS.md</code> or <code>.cursorrules</code> at your workspace root, or in the custom instructions of Cursor, Windsurf, Claude Desktop, or Antigravity.
                </p>
              </div>

              <div className="flex items-center justify-between text-xs text-[#64748b]">
                <span className="font-medium text-[#cbd5e1]">Preview of AGENTS.md:</span>
                <button
                  type="button"
                  onClick={() => copyToClipboard(AGENT_RULE_CONTENT, 'rule-tab')}
                  className="flex items-center gap-1 px-2.5 py-1 rounded-lg bg-[#141824] border border-[#202638] text-[#cbd5e1] hover:text-[#f8fafc] hover:bg-[#1c2233] transition-colors cursor-pointer text-xs"
                >
                  {copiedType === 'rule-tab' ? (
                    <>
                      <Check size={12} weight="bold" className="text-emerald-400" />
                      <span className="text-emerald-400 font-medium">Copied!</span>
                    </>
                  ) : (
                    <>
                      <Copy size={12} />
                      <span>Copy Text</span>
                    </>
                  )}
                </button>
              </div>

              <pre className="font-mono text-[11px] leading-relaxed p-3.5 rounded-xl bg-[#08090d] border border-[#1f2433] text-[#cbd5e1] overflow-x-auto max-h-56">
                {AGENT_RULE_CONTENT}
              </pre>
            </Tabs.Content>

            {/* Tab 3: Skill */}
            <Tabs.Content value="skill" className="flex-1 overflow-y-auto space-y-3 pt-3 outline-none pr-1">
              <div className="rounded-xl bg-[#141824] border border-[#202638] p-3 text-xs text-[#cbd5e1]">
                <div className="flex items-center gap-1.5 font-semibold text-[#f8fafc] mb-1">
                  <CheckCircle size={14} className="text-purple-400" />
                  <span>How to install this skill:</span>
                </div>
                <p className="text-[11px] text-[#94a3b8] leading-relaxed">
                  Save this into <code>.agents/skills/graphfinder-curator/SKILL.md</code>. Antigravity and compatible agents will automatically discover and load it on demand.
                </p>
              </div>

              <div className="flex items-center justify-between text-xs text-[#64748b]">
                <span className="font-medium text-[#cbd5e1]">Preview of SKILL.md:</span>
                <button
                  type="button"
                  onClick={() => copyToClipboard(SKILL_CONTENT, 'skill-tab')}
                  className="flex items-center gap-1 px-2.5 py-1 rounded-lg bg-[#141824] border border-[#202638] text-[#cbd5e1] hover:text-[#f8fafc] hover:bg-[#1c2233] transition-colors cursor-pointer text-xs"
                >
                  {copiedType === 'skill-tab' ? (
                    <>
                      <Check size={12} weight="bold" className="text-emerald-400" />
                      <span className="text-emerald-400 font-medium">Copied!</span>
                    </>
                  ) : (
                    <>
                      <Copy size={12} />
                      <span>Copy Skill</span>
                    </>
                  )}
                </button>
              </div>

              <pre className="font-mono text-[11px] leading-relaxed p-3.5 rounded-xl bg-[#08090d] border border-[#1f2433] text-purple-300 overflow-x-auto max-h-56">
                {SKILL_CONTENT}
              </pre>
            </Tabs.Content>
          </Tabs.Root>

          {/* Footer */}
          <div className="flex items-center justify-end pt-3 border-t border-[#1f2433] mt-3">
            <button
              type="button"
              onClick={onClose}
              className="px-4 py-1.5 text-xs font-semibold rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white transition-colors cursor-pointer shadow-sm"
            >
              Done
            </button>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
};
