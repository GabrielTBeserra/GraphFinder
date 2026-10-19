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
  Folder,
} from '@phosphor-icons/react';
import { AGENT_RULE_CONTENT, SKILL_CONTENT, MCP_CONFIG_SNIPPET, MCP_STDIO_CONFIG_SNIPPET } from '../services/aiConfig';
import { fetchMcpConfig, type McpConfigInfo } from '../services/api';

interface McpModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export const McpModal: React.FC<McpModalProps> = ({ isOpen, onClose }) => {
  const [copiedType, setCopiedType] = useState<string | null>(null);
  const [mcpInfo, setMcpInfo] = useState<McpConfigInfo | null>(null);
  const [mcpMode, setMcpMode] = useState<'sse' | 'stdio'>('sse');

  useEffect(() => {
    if (isOpen) {
      fetchMcpConfig()
        .then((info) => setMcpInfo(info))
        .catch((err) => console.error('Error detecting MCP paths:', err));
    }
  }, [isOpen]);

  const sseConfigJson = mcpInfo?.config_json || MCP_CONFIG_SNIPPET;
  const stdioConfigJson = mcpInfo
    ? `{
  "mcpServers": {
    "graphfinder": {
      "command": "${mcpInfo.executable_path.replace(/\\/g, '\\\\')}",
      "args": ["--mcp"],
      "env": {
        "GRAPHFINDER_VAULT": "${mcpInfo.vault_path.replace(/\\/g, '\\\\')}",
        "GRAPHFINDER_DB": "${mcpInfo.db_path.replace(/\\/g, '\\\\')}"
      }
    }
  }
}`
    : MCP_STDIO_CONFIG_SNIPPET;

  const currentConfigJson = mcpMode === 'sse' ? sseConfigJson : stdioConfigJson;

  const copyToClipboard = (text: string, type: string) => {
    navigator.clipboard.writeText(text);
    setCopiedType(type);
    setTimeout(() => setCopiedType(null), 2500);
  };

  return (
    <Dialog.Root open={isOpen} onOpenChange={(open) => !open && onClose()}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 z-50 bg-black/75 backdrop-blur-sm" />
        <Dialog.Content className="fixed left-1/2 top-1/2 z-50 w-full max-w-2xl max-h-[90vh] -translate-x-1/2 -translate-y-1/2 rounded-xl bg-[#111318] border border-[#262933] p-6 shadow-2xl flex flex-col focus:outline-none overflow-hidden text-[#f1f5f9]">
          {/* Header */}
          <div className="flex items-start justify-between gap-4 pb-4 border-b border-[#1f232d]">
            <div className="flex items-center gap-3">
              <div className="flex items-center justify-center w-8 h-8 rounded-lg bg-[#161922] border border-[#262a36] text-[#cbd5e1]">
                <Robot size={18} weight="regular" />
              </div>
              <div>
                <Dialog.Title className="text-sm font-semibold text-[#f8fafc]">
                  AI Integration: Rules, Skill & MCP
                </Dialog.Title>
                <Dialog.Description className="text-xs text-[#64748b] mt-0.5">
                  Token economy rules and configuration ready for your editor or AI agent.
                </Dialog.Description>
              </div>
            </div>

            <Dialog.Close asChild>
              <button
                type="button"
                className="rounded-md p-1.5 text-[#64748b] hover:text-[#f8fafc] hover:bg-[#1a1e28] transition-colors cursor-pointer"
                title="Close"
              >
                <X size={15} />
              </button>
            </Dialog.Close>
          </div>

          {/* Quick Copy Row */}
          <div className="grid grid-cols-3 gap-2 py-3">
            <button
              type="button"
              onClick={() => copyToClipboard(AGENT_RULE_CONTENT, 'rule-quick')}
              className="flex items-center justify-center gap-2 px-3 py-2 text-xs font-medium rounded-lg bg-[#13161f] border border-[#1e222d] hover:bg-[#1a1e28] hover:border-[#2d3342] text-[#cbd5e1] transition-all cursor-pointer"
            >
              {copiedType === 'rule-quick' ? (
                <>
                  <Check size={14} weight="bold" className="text-[#10b981]" />
                  <span className="text-[#10b981]">Rule Copied!</span>
                </>
              ) : (
                <>
                  <FileText size={14} className="text-[#64748b]" />
                  <span>Copy AGENTS.md</span>
                </>
              )}
            </button>

            <button
              type="button"
              onClick={() => copyToClipboard(SKILL_CONTENT, 'skill-quick')}
              className="flex items-center justify-center gap-2 px-3 py-2 text-xs font-medium rounded-lg bg-[#13161f] border border-[#1e222d] hover:bg-[#1a1e28] hover:border-[#2d3342] text-[#cbd5e1] transition-all cursor-pointer"
            >
              {copiedType === 'skill-quick' ? (
                <>
                  <Check size={14} weight="bold" className="text-[#10b981]" />
                  <span className="text-[#10b981]">Skill Copied!</span>
                </>
              ) : (
                <>
                  <BookOpen size={14} className="text-[#64748b]" />
                  <span>Copy SKILL.md</span>
                </>
              )}
            </button>

            <button
              type="button"
              onClick={() => copyToClipboard(currentConfigJson, 'mcp-quick')}
              className="flex items-center justify-center gap-2 px-3 py-2 text-xs font-medium rounded-lg bg-[#13161f] border border-[#1e222d] hover:bg-[#1a1e28] hover:border-[#2d3342] text-[#cbd5e1] transition-all cursor-pointer"
            >
              {copiedType === 'mcp-quick' ? (
                <>
                  <Check size={14} weight="bold" className="text-[#10b981]" />
                  <span className="text-[#10b981]">Config Copied!</span>
                </>
              ) : (
                <>
                  <TerminalWindow size={14} className="text-[#64748b]" />
                  <span>Copy Config JSON</span>
                </>
              )}
            </button>
          </div>

          {/* Radix Tabs Section */}
          <Tabs.Root defaultValue="rule" className="flex flex-col flex-1 min-h-0">
            <Tabs.List className="flex items-center gap-1 border-b border-[#1f232d] pb-2">
              <Tabs.Trigger
                value="rule"
                className="flex items-center gap-1.5 px-3 py-1.5 rounded-md text-xs font-medium text-[#64748b] hover:text-[#cbd5e1] data-[state=active]:bg-[#1a1e28] data-[state=active]:text-[#f8fafc] data-[state=active]:border data-[state=active]:border-[#2c3242] transition-all cursor-pointer outline-none"
              >
                <FileText size={13} />
                <span>Agent Rule (AGENTS.md)</span>
              </Tabs.Trigger>

              <Tabs.Trigger
                value="skill"
                className="flex items-center gap-1.5 px-3 py-1.5 rounded-md text-xs font-medium text-[#64748b] hover:text-[#cbd5e1] data-[state=active]:bg-[#1a1e28] data-[state=active]:text-[#f8fafc] data-[state=active]:border data-[state=active]:border-[#2c3242] transition-all cursor-pointer outline-none"
              >
                <BookOpen size={13} />
                <span>Curator Skill (SKILL.md)</span>
              </Tabs.Trigger>

              <Tabs.Trigger
                value="mcp"
                className="flex items-center gap-1.5 px-3 py-1.5 rounded-md text-xs font-medium text-[#64748b] hover:text-[#cbd5e1] data-[state=active]:bg-[#1a1e28] data-[state=active]:text-[#f8fafc] data-[state=active]:border data-[state=active]:border-[#2c3242] transition-all cursor-pointer outline-none"
              >
                <Cpu size={13} />
                <span>MCP Protocol & Tools</span>
              </Tabs.Trigger>
            </Tabs.List>

            {/* Tab 1: Agent Rule */}
            <Tabs.Content value="rule" className="flex-1 overflow-y-auto space-y-3 pt-3 outline-none">
              <div className="rounded-lg bg-[#161922] border border-[#262a36] p-3 text-xs text-[#cbd5e1]">
                <div className="flex items-center gap-1.5 font-semibold text-[#f8fafc] mb-1">
                  <CheckCircle size={14} className="text-[#10b981]" />
                  <span>How to use in your editor:</span>
                </div>
                <p className="text-[11px] text-[#94a3b8] leading-relaxed">
                  Paste this content into <code>AGENTS.md</code> or <code>.cursorrules</code> at your workspace root, or into system instructions for Cursor, Windsurf, Claude Desktop, or Antigravity.
                </p>
              </div>

              <div className="flex items-center justify-between text-xs text-[#64748b]">
                <span className="font-medium text-[#cbd5e1]">Preview of AGENTS.md:</span>
                <button
                  type="button"
                  onClick={() => copyToClipboard(AGENT_RULE_CONTENT, 'rule-tab')}
                  className="flex items-center gap-1 px-2.5 py-1 rounded bg-[#13161f] border border-[#1e222d] text-[#cbd5e1] hover:text-[#f8fafc] hover:bg-[#1a1e28] transition-colors cursor-pointer text-xs"
                >
                  {copiedType === 'rule-tab' ? (
                    <>
                      <Check size={12} weight="bold" className="text-[#10b981]" />
                      <span className="text-[#10b981]">Copied!</span>
                    </>
                  ) : (
                    <>
                      <Copy size={12} />
                      <span>Copy Text</span>
                    </>
                  )}
                </button>
              </div>

              <pre className="font-mono text-[11px] leading-relaxed p-3 rounded-lg bg-[#0d0f14] border border-[#1f232d] text-[#cbd5e1] overflow-x-auto max-h-56">
                {AGENT_RULE_CONTENT}
              </pre>
            </Tabs.Content>

            {/* Tab 2: Skill */}
            <Tabs.Content value="skill" className="flex-1 overflow-y-auto space-y-3 pt-3 outline-none">
              <div className="rounded-lg bg-[#161922] border border-[#262a36] p-3 text-xs text-[#cbd5e1]">
                <div className="flex items-center gap-1.5 font-semibold text-[#f8fafc] mb-1">
                  <CheckCircle size={14} className="text-[#a78bfa]" />
                  <span>How to install this skill:</span>
                </div>
                <p className="text-[11px] text-[#94a3b8] leading-relaxed">
                  Save this content into <code>.agents/skills/graphfinder-curator/SKILL.md</code>. Antigravity will automatically load the skill on demand.
                </p>
              </div>

              <div className="flex items-center justify-between text-xs text-[#64748b]">
                <span className="font-medium text-[#cbd5e1]">Preview of SKILL.md:</span>
                <button
                  type="button"
                  onClick={() => copyToClipboard(SKILL_CONTENT, 'skill-tab')}
                  className="flex items-center gap-1 px-2.5 py-1 rounded bg-[#13161f] border border-[#1e222d] text-[#cbd5e1] hover:text-[#f8fafc] hover:bg-[#1a1e28] transition-colors cursor-pointer text-xs"
                >
                  {copiedType === 'skill-tab' ? (
                    <>
                      <Check size={12} weight="bold" className="text-[#10b981]" />
                      <span className="text-[#10b981]">Copied!</span>
                    </>
                  ) : (
                    <>
                      <Copy size={12} />
                      <span>Copy Skill</span>
                    </>
                  )}
                </button>
              </div>

              <pre className="font-mono text-[11px] leading-relaxed p-3 rounded-lg bg-[#0d0f14] border border-[#1f232d] text-[#a78bfa] overflow-x-auto max-h-56">
                {SKILL_CONTENT}
              </pre>
            </Tabs.Content>

            {/* Tab 3: MCP Config */}
            <Tabs.Content value="mcp" className="flex-1 overflow-y-auto space-y-3 pt-3 outline-none">
              {/* Transport Mode Switcher */}
              <div className="flex items-center gap-1.5 p-1 rounded-lg bg-[#0d0f14] border border-[#1f232d]">
                <button
                  type="button"
                  onClick={() => setMcpMode('sse')}
                  className={`flex-1 py-1.5 px-3 rounded-md text-xs font-medium transition-all cursor-pointer ${
                    mcpMode === 'sse'
                      ? 'bg-indigo-600/20 text-indigo-300 border border-indigo-500/30 font-semibold'
                      : 'text-[#94a3b8] hover:text-[#f8fafc] hover:bg-[#161922]'
                  }`}
                >
                  📡 SSE Local Server (Recomendado)
                </button>
                <button
                  type="button"
                  onClick={() => setMcpMode('stdio')}
                  className={`flex-1 py-1.5 px-3 rounded-md text-xs font-medium transition-all cursor-pointer ${
                    mcpMode === 'stdio'
                      ? 'bg-indigo-600/20 text-indigo-300 border border-indigo-500/30 font-semibold'
                      : 'text-[#94a3b8] hover:text-[#f8fafc] hover:bg-[#161922]'
                  }`}
                >
                  ⚙️ Stdio CLI (Fallback Headless)
                </button>
              </div>

              {mcpMode === 'sse' ? (
                <div className="rounded-lg bg-[#161922] border border-[#262a36] p-3 text-xs text-[#cbd5e1] space-y-1">
                  <div className="flex items-center gap-1.5 font-semibold text-[#10b981]">
                    <span className="w-2 h-2 rounded-full bg-emerald-500 animate-pulse"></span>
                    <span>Servidor MCP Embutido Ativo (Porta 45123)</span>
                  </div>
                  <p className="text-[11px] text-[#94a3b8] leading-relaxed">
                    O servidor sobe junto com o GraphFinder na sua máquina local. Ele compartilha o cache de embeddings, não causa bloqueios no SQLite e atualiza o grafo em tempo real na tela!
                  </p>
                  <div className="font-mono text-[11px] pt-1 text-[#38bdf8] flex items-center gap-2">
                    <span>SSE: <code>http://127.0.0.1:45123/sse</code></span>
                    <span>•</span>
                    <span>HTTP: <code>http://127.0.0.1:45123/mcp</code></span>
                  </div>
                </div>
              ) : (
                mcpInfo && (
                  <div className="rounded-lg bg-[#161922] border border-[#262a36] p-3 text-xs text-[#cbd5e1]">
                    <div className="flex items-center gap-1.5 font-semibold text-[#38bdf8] mb-1">
                      <Folder size={14} className="text-[#38bdf8]" />
                      <span>Detected System Paths:</span>
                    </div>
                    <div className="font-mono text-[11px] space-y-0.5 text-[#94a3b8] break-all">
                      <div><strong>Executable:</strong> {mcpInfo.executable_path}</div>
                      <div><strong>Vault Directory:</strong> {mcpInfo.vault_path}</div>
                      <div><strong>Database:</strong> {mcpInfo.db_path}</div>
                    </div>
                  </div>
                )
              )}

              <div className="flex items-center justify-between text-xs text-[#64748b]">
                <span className="font-medium text-[#cbd5e1]">
                  {mcpMode === 'sse' ? 'Configuração SSE para seu editor:' : 'Configuração Stdio para seu editor:'}
                </span>
                <button
                  type="button"
                  onClick={() => copyToClipboard(currentConfigJson, 'mcp-tab')}
                  className="flex items-center gap-1 px-2.5 py-1 rounded bg-[#13161f] border border-[#1e222d] text-[#cbd5e1] hover:text-[#f8fafc] hover:bg-[#1a1e28] transition-colors cursor-pointer text-xs"
                >
                  {copiedType === 'mcp-tab' ? (
                    <>
                      <Check size={12} weight="bold" className="text-[#10b981]" />
                      <span className="text-[#10b981]">JSON Copied!</span>
                    </>
                  ) : (
                    <>
                      <Copy size={12} />
                      <span>Copy JSON</span>
                    </>
                  )}
                </button>
              </div>

              <pre className="font-mono text-[11px] leading-relaxed p-3 rounded-lg bg-[#0d0f14] border border-[#1f232d] text-[#cbd5e1] overflow-x-auto max-h-40">
                {currentConfigJson}
              </pre>

              <div className="space-y-1.5 pt-1">
                <div className="flex items-center gap-1.5 text-xs font-semibold text-[#cbd5e1]">
                  <Cpu size={13} className="text-[#64748b]" />
                  <span>Available Atomic MCP Tools:</span>
                </div>
                <ul className="text-xs text-[#94a3b8] space-y-1 pl-4 list-disc">
                  <li><code>search_semantic(query)</code>: Returns atomic chunks with <code>[[Note#Section]]</code> deep links.</li>
                  <li><code>get_compressed_context(note_id)</code>: Structural digest and section map (~100 tokens).</li>
                  <li><code>suggest_links(text)</code>: Real-time semantic auto-linker for draft paragraphs.</li>
                  <li><code>store_context(title, content, tags)</code>: Stores note with local chunking and ONNX embeddings.</li>
                  <li><code>get_graph_neighborhood(note_id)</code>: Topological traversal radius of adjacent nodes.</li>
                </ul>
              </div>
            </Tabs.Content>
          </Tabs.Root>

          {/* Footer */}
          <div className="flex items-center justify-end pt-3 border-t border-[#1f232d] mt-3">
            <button
              type="button"
              onClick={onClose}
              className="px-4 py-1.5 text-xs font-semibold rounded-md bg-indigo-600 hover:bg-indigo-500 text-white transition-colors cursor-pointer"
            >
              Done
            </button>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
};
