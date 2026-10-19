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
import { AGENT_RULE_CONTENT, SKILL_CONTENT, MCP_CONFIG_SNIPPET } from '../services/aiConfig';
import { fetchMcpConfig, type McpConfigInfo } from '../services/api';

interface McpModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export const McpModal: React.FC<McpModalProps> = ({ isOpen, onClose }) => {
  const [copiedType, setCopiedType] = useState<string | null>(null);
  const [mcpInfo, setMcpInfo] = useState<McpConfigInfo | null>(null);

  useEffect(() => {
    if (isOpen) {
      fetchMcpConfig()
        .then((info) => setMcpInfo(info))
        .catch((err) => console.error('Erro ao detectar caminhos do MCP:', err));
    }
  }, [isOpen]);

  const currentConfigJson = mcpInfo?.config_json || MCP_CONFIG_SNIPPET;

  const copyToClipboard = (text: string, type: string) => {
    navigator.clipboard.writeText(text);
    setCopiedType(type);
    setTimeout(() => setCopiedType(null), 2500);
  };

  return (
    <Dialog.Root open={isOpen} onOpenChange={(open) => !open && onClose()}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 z-50 bg-black/75 backdrop-blur-sm animate-in fade-in-0 duration-200" />
        <Dialog.Content className="fixed left-1/2 top-1/2 z-50 w-full max-w-2xl max-h-[90vh] -translate-x-1/2 -translate-y-1/2 rounded-xl bg-[#111318] border border-[#262933] p-6 shadow-2xl flex flex-col animate-in fade-in-0 zoom-in-95 duration-200 focus:outline-none overflow-hidden text-[#f1f5f9]">
          {/* Header */}
          <div className="flex items-start justify-between gap-4 pb-4 border-b border-[#1f232d]">
            <div className="flex items-center gap-3">
              <div className="flex items-center justify-center w-8 h-8 rounded-lg bg-[#161922] border border-[#262a36] text-[#cbd5e1]">
                <Robot size={18} weight="regular" />
              </div>
              <div>
                <Dialog.Title className="text-sm font-semibold text-[#f8fafc]">
                  Integracao com IA: Regras, Skill & MCP
                </Dialog.Title>
                <Dialog.Description className="text-xs text-[#64748b] mt-0.5">
                  Regras de economia de tokens e configuracao prontas para seu editor ou agente.
                </Dialog.Description>
              </div>
            </div>

            <Dialog.Close asChild>
              <button
                type="button"
                className="rounded-md p-1.5 text-[#64748b] hover:text-[#f8fafc] hover:bg-[#1a1e28] transition-colors cursor-pointer"
                title="Fechar"
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
                  <span className="text-[#10b981]">Regra Copiada!</span>
                </>
              ) : (
                <>
                  <FileText size={14} className="text-[#64748b]" />
                  <span>Copiar AGENTS.md</span>
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
                  <span className="text-[#10b981]">Skill Copiada!</span>
                </>
              ) : (
                <>
                  <BookOpen size={14} className="text-[#64748b]" />
                  <span>Copiar SKILL.md</span>
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
                  <span className="text-[#10b981]">Config Copiada!</span>
                </>
              ) : (
                <>
                  <TerminalWindow size={14} className="text-[#64748b]" />
                  <span>Copiar Config JSON</span>
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
                <span>Regra do Agente (AGENTS.md)</span>
              </Tabs.Trigger>

              <Tabs.Trigger
                value="skill"
                className="flex items-center gap-1.5 px-3 py-1.5 rounded-md text-xs font-medium text-[#64748b] hover:text-[#cbd5e1] data-[state=active]:bg-[#1a1e28] data-[state=active]:text-[#f8fafc] data-[state=active]:border data-[state=active]:border-[#2c3242] transition-all cursor-pointer outline-none"
              >
                <BookOpen size={13} />
                <span>Skill de Curadoria (SKILL.md)</span>
              </Tabs.Trigger>

              <Tabs.Trigger
                value="mcp"
                className="flex items-center gap-1.5 px-3 py-1.5 rounded-md text-xs font-medium text-[#64748b] hover:text-[#cbd5e1] data-[state=active]:bg-[#1a1e28] data-[state=active]:text-[#f8fafc] data-[state=active]:border data-[state=active]:border-[#2c3242] transition-all cursor-pointer outline-none"
              >
                <Cpu size={13} />
                <span>Protocolo MCP & Tools</span>
              </Tabs.Trigger>
            </Tabs.List>

            {/* Tab 1: Agent Rule */}
            <Tabs.Content value="rule" className="flex-1 overflow-y-auto space-y-3 pt-3 outline-none">
              <div className="rounded-lg bg-[#161922] border border-[#262a36] p-3 text-xs text-[#cbd5e1]">
                <div className="flex items-center gap-1.5 font-semibold text-[#f8fafc] mb-1">
                  <CheckCircle size={14} className="text-[#10b981]" />
                  <span>Como usar no seu editor:</span>
                </div>
                <p className="text-[11px] text-[#94a3b8] leading-relaxed">
                  Cole este conteudo no arquivo <code>AGENTS.md</code> ou <code>.cursorrules</code> na raiz do projeto, ou nas instrucoes de sistema do Cursor, Windsurf, Claude Desktop ou Antigravity.
                </p>
              </div>

              <div className="flex items-center justify-between text-xs text-[#64748b]">
                <span className="font-medium text-[#cbd5e1]">Previa de AGENTS.md:</span>
                <button
                  type="button"
                  onClick={() => copyToClipboard(AGENT_RULE_CONTENT, 'rule-tab')}
                  className="flex items-center gap-1 px-2.5 py-1 rounded bg-[#13161f] border border-[#1e222d] text-[#cbd5e1] hover:text-[#f8fafc] hover:bg-[#1a1e28] transition-colors cursor-pointer text-xs"
                >
                  {copiedType === 'rule-tab' ? (
                    <>
                      <Check size={12} weight="bold" className="text-[#10b981]" />
                      <span className="text-[#10b981]">Copiado!</span>
                    </>
                  ) : (
                    <>
                      <Copy size={12} />
                      <span>Copiar Texto</span>
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
                  <span>Como instalar a Skill:</span>
                </div>
                <p className="text-[11px] text-[#94a3b8] leading-relaxed">
                  Salve o conteudo em <code>.agents/skills/graphfinder-curator/SKILL.md</code>. O Antigravity carregara a skill automaticamente sob demanda.
                </p>
              </div>

              <div className="flex items-center justify-between text-xs text-[#64748b]">
                <span className="font-medium text-[#cbd5e1]">Previa de SKILL.md:</span>
                <button
                  type="button"
                  onClick={() => copyToClipboard(SKILL_CONTENT, 'skill-tab')}
                  className="flex items-center gap-1 px-2.5 py-1 rounded bg-[#13161f] border border-[#1e222d] text-[#cbd5e1] hover:text-[#f8fafc] hover:bg-[#1a1e28] transition-colors cursor-pointer text-xs"
                >
                  {copiedType === 'skill-tab' ? (
                    <>
                      <Check size={12} weight="bold" className="text-[#10b981]" />
                      <span className="text-[#10b981]">Copiada!</span>
                    </>
                  ) : (
                    <>
                      <Copy size={12} />
                      <span>Copiar Skill</span>
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
              {mcpInfo && (
                <div className="rounded-lg bg-[#161922] border border-[#262a36] p-3 text-xs text-[#cbd5e1]">
                  <div className="flex items-center gap-1.5 font-semibold text-[#38bdf8] mb-1">
                    <Folder size={14} className="text-[#38bdf8]" />
                    <span>Caminhos detectados no sistema:</span>
                  </div>
                  <div className="font-mono text-[11px] space-y-0.5 text-[#94a3b8] break-all">
                    <div><strong>Executavel:</strong> {mcpInfo.executable_path}</div>
                    <div><strong>Cofre (Vault):</strong> {mcpInfo.vault_path}</div>
                    <div><strong>Banco de Dados:</strong> {mcpInfo.db_path}</div>
                  </div>
                </div>
              )}

              <div className="flex items-center justify-between text-xs text-[#64748b]">
                <span className="font-medium text-[#cbd5e1]">Configuracao JSON pronta para copiar:</span>
                <button
                  type="button"
                  onClick={() => copyToClipboard(currentConfigJson, 'mcp-tab')}
                  className="flex items-center gap-1 px-2.5 py-1 rounded bg-[#13161f] border border-[#1e222d] text-[#cbd5e1] hover:text-[#f8fafc] hover:bg-[#1a1e28] transition-colors cursor-pointer text-xs"
                >
                  {copiedType === 'mcp-tab' ? (
                    <>
                      <Check size={12} weight="bold" className="text-[#10b981]" />
                      <span className="text-[#10b981]">JSON Copiado!</span>
                    </>
                  ) : (
                    <>
                      <Copy size={12} />
                      <span>Copiar JSON</span>
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
                  <span>Ferramentas Atomicas Disponiveis:</span>
                </div>
                <ul className="text-xs text-[#94a3b8] space-y-1 pl-4 list-disc">
                  <li><code>search_semantic(query)</code>: Retorna trechos atomicos com deep links <code>[[Nota#Secao]]</code>.</li>
                  <li><code>get_compressed_context(note_id)</code>: Digest e mapa de secoes (~100 tokens).</li>
                  <li><code>suggest_links(text)</code>: Auto-linker semantico em tempo real para novos paragrafos.</li>
                  <li><code>store_context(title, content, tags)</code>: Registra nota com chunking e embeddings locais.</li>
                  <li><code>get_graph_neighborhood(note_id)</code>: Raio topologico de nos vizinhos.</li>
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
              Concluir
            </button>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
};
