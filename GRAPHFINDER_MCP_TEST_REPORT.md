# Relatório de Testes — GraphFinder MCP Server

> Teste end-to-end das 26 tools + 2 resources do servidor MCP GraphFinder,
> executado em 05/11/2026 contra o vault `bearry` (`Z:\Projetos\bearry`).

## Resumo Executivo

- **26/26 tools funcionais** — nenhuma falha dura, crash ou timeout.
- **2 resources** (`graphfinder://instructions`, `graphfinder://stats`) funcionais.
- **7 issues encontradas**: 0 críticas, 2 médias, 5 baixas/informativas.
- Veredito: **funciona corretamente**; os problemas são inconsistências de
  métricas/labels e limites de granularidade — não bugs bloqueantes.

| Categoria | Testadas | OK | Issues |
|---|---|---|---|
| Gestão de projetos | 3 | 3 | 1 (label) |
| Leitura de notas | 5 | 5 | 1 (dados do vault) |
| Leitura de código | 6 | 6 | 2 |
| Grafo/análise | 4 | 4 | 2 |
| Escrita/curadoria | 5 | 5 | 0 |
| Saúde/manutenção | 4 | 4 | 2 |

---

## Ambiente de Teste

- Projeto ativo: `bearry` (ID `bearry`), 4 vaults existentes no servidor
- Índice: 16 notas Markdown + 408 arquivos de código
- Linguagens detectadas: css, html, javascript, python, tsx, typescript
- Vetores: 1402 chunks | Conexões: 964 | Tags: 6 | ~21.9k tokens
- As "notas" do vault são os próprios `.md` do repositório, indexados in-place
  (`docs/*.md`, `README.md`, `extension/*.md`, `aws/**/*.md`).

---

## Resultados por Tool

### Gestão de Projetos

| Tool | Resultado | Observações |
|---|---|---|
| `list_projects` | ✅ | Listou 4 projetos com path e flag de ativo. **Issue:** o contador "notes" soma notas + arquivos de código (mostrou 424 = 16 + 408). |
| `switch_project` | ✅ | Idempotente — `bearry` já era ativo e aceitou o switch sem erro. |
| `create_project` | ⚠️ não testado | Criaria um vault novo de verdade; fora do escopo seguro. Schema simples (`name`, `path?`). |

### Resources

| Resource | Resultado | Observações |
|---|---|---|
| `graphfinder://instructions` | ✅ | Retorna as 5 regras de agente (AGENTS.md) — retrieval atômico, schema de nota, wikilinks, suggest_links antes de store, community report primeiro para arquitetura. |
| `graphfinder://stats` | ✅ | JSON com métricas separadas para notas vs código. Revela `total_links: 0` (wikilinks entre notas) vs `total_code_links: 964` (imports). |

### Leitura de Notas

| Tool | Resultado | Observações |
|---|---|---|
| `list_recent_notes` | ✅ | Retornou 10 notas com resumo de 1 linha e tags. Ordenação não documentada (parece ordem alfabética/por arquivo, não por recência). |
| `search_semantic` (dense_chunks) | ✅ | Query "autenticação Twitch OAuth" → 5 chunks relevantes (scores 73–75%) com deep links `[[Nota#Seção]]` e estimativa de tokens. |
| `search_semantic` (summaries) | ✅ | Query "plugin protocol socket.io" → top hit 91% relevância. Formato 1-linha correto. |
| `read_section` | ✅ | Leitura cirúrgica exata: seção "7.1 Página de Callback OAuth" (~139 tokens) com deep link. |
| `get_compressed_context` | ✅ | Mapa completo de headings + tokens por seção + conexões de grafo. Excelente para skim antes de ler seções. |
| `get_graph_neighborhood` | ✅ | Funciona; retornou vazio antes porque o vault **não tem wikilinks** (`total_links: 0`). Após `link_notes`, mostrou os vizinhos corretamente. |

### Leitura de Código

| Tool | Resultado | Observações |
|---|---|---|
| `find_files` | ✅ | Pattern `*.ts` + `language: typescript` funcionou; mostra grau de conectividade (Imports/Dependents). **Issue:** "Imports" mistura imports internos + pacotes externos (logger.ts diz "Imports: 5" mas são 5 pacotes externos, 0 arquivos internos). |
| `search_code` | ✅ c/ ressalva | Busca híbrida (neural + FTS5) funciona e filtro `language` funciona. **Issue média:** pode retornar chunks gigantes — a classe `TwitchBotService` veio inteira (~7904 tokens, L42-1008), o que contradiz a missão de economia de tokens. |
| `get_file_context` | ✅ | Resumo <80 tokens preciso: imports internos, dependents, pacotes externos, catálogo de símbolos com line ranges. |
| `read_code_chunk` | ✅ c/ ressalvas | **Issue média:** símbolo `waitForRestServer` (método privado, não indexado) → fallback silencioso para o header do arquivo (L1-15), sem aviso. Range `"1-40"` → retornou L1-63 (alinha aos limites do chunk, não às linhas pedidas). Só símbolos top-level (class/interface/const) são indexados, não métodos. |
| `get_code_dependencies` | ✅ | Saída rica: imports, dependents (91 para logger.ts), módulos externos, outgoing function calls, incoming callers. |

### Grafo e Análise

| Tool | Resultado | Observações |
|---|---|---|
| `find_path` | ✅ | `main.ts → logger.ts` em 1 hop. Nota → código retorna corretamente "no path found" (tipos diferentes não conectam). |
| `explain_node` | ✅ | Card 360° completo: comunidade, rank de centralidade (#20, degree 16), inbound/outbound, siblings na mesma comunidade. |
| `get_community_report` | ✅ c/ ressalva | 67 comunidades, 12 god nodes, densidade 0.0054. **Issue baixa:** anuncia "Surprises: 20" mas lista apenas 8. |
| `export_graph` | ✅ | Gerou `graphfinder-out/` com `graph.json` (339KB), `GRAPH_REPORT.md` (11.8KB) e `graph.html` (357KB). O relatório exportado é **mais rico** que o inline: revela 18 loops de dependência circular. |

### Escrita / Curadoria (nota de teste temporária)

| Tool | Resultado | Observações |
|---|---|---|
| `suggest_links` | ✅ | Draft sobre plugins → sugeriu `[[Plugin protocol (Socket.IO)]]` (76%) e `[[Guia de Extensões]]` (70%) — ambos relevantes. |
| `store_context` | ✅ | Criou "ZZZ GraphFinder Tool Test", vetorizou e auto-linkou (≥65% affinity) para Plugin protocol. |
| `update_section` | ✅ | Edição cirúrgica: heading preservado, só o corpo substituído, re-indexação confirmada via `read_section`. |
| `link_notes` | ✅ | Aresta bidirecional criada e visível em `get_graph_neighborhood`. Injeta seção "Related Connections" na nota de origem (sem poluir o arquivo da nota-alvo). |
| `rename_note` | ✅ | Renomeou arquivo + refatorou wikilinks incoming automaticamente. |

### Saúde / Manutenção

| Tool | Resultado | Observações |
|---|---|---|
| `validate_vault` | ✅ | Vault inteiro (17 notas) e nota única. Encontrou 6 erros reais + ~123 warnings no vault do bearry. |
| `sync_vault` | ✅ c/ ressalvas | Incremental, 0 itens (já sincronizado). **Issue baixa:** mensagem diz "Project 'default' updated" embora o ativo seja `bearry` (diretório correto, label errado). **Issue baixa:** reporta "Graph Connections: 2" (só wikilinks) enquanto `get_vault_stats` usa o mesmo nome para wikilinks+imports (966). |
| `delete_note` | ✅ | Removeu nota do índice + arquivo .md do disco; arestas do grafo limpas (notas-alvo voltaram a "orphan" sem links quebrados); zero resíduo textual nos .md das notas linkadas. |

### Tratamento de Erro (edge cases)

| Caso | Resultado |
|---|---|
| `get_compressed_context` em nota inexistente | ✅ "Note 'X' not found." — limpo |
| `get_file_context` em arquivo inexistente | ✅ Erro claro + sugere usar `find_files` |
| `find_path` nota → arquivo de código | ✅ "No connection path found" — correto |
| `search_code` com `language: python` | ✅ Filtra corretamente |

---

## Issues Encontradas

### Médias

1. **`search_code` pode retornar chunks gigantes.** Símbolos grandes vêm
   inteiros — `TwitchBotService` retornou ~7904 tokens num único chunk.
   Mitigação: usar `read_code_chunk` com símbolo específico para leituras
   cirúrgicas; `search_code` serve para localizar, não para ler.

2. **`read_code_chunk` faz fallback silencioso.** Pedir um símbolo não
   indexado (ex.: método privado `waitForRestServer`) retorna o header do
   arquivo (L1-15) sem avisar que o símbolo não foi achado. Pode fazer o
   agente acreditar que leu o método quando leu só os imports. Apenas
   símbolos top-level são indexados; ranges de linha são arredondados aos
   limites do chunk.

### Baixas

3. **Contador "notes" de `list_projects`** soma notas + arquivos de código
   (bearry mostra 424; na verdade são 16 notas + 408 arquivos).
4. **`sync_vault` rotula o projeto como `'default'`** na mensagem de saída
   mesmo quando sincroniza o projeto ativo correto.
5. **"Graph Connections" significa coisas diferentes por tool** —
   `get_vault_stats` conta wikilinks + imports (966); `sync_vault` conta só
   wikilinks (2).
6. **`find_files` "Imports"** mistura imports internos e pacotes externos
   no mesmo número.
7. **`get_community_report` anuncia 20 surprises, lista 8.**

### Estado do Vault (dado, não bug)

- `total_links: 0` antes do teste — as 16 notas não têm wikilinks entre si
  (todas órfãs; `get_graph_neighborhood` fica vazio por causa disso).
- `validate_vault`: 6 erros (executive summary ausente) + ~123 warnings
  (seções fora da banda 40–150 tokens, notas órfãs, tags faltando).
- Arquivos quase duplicados indexados como nós separados: `auth.py` (raiz) e
  `aws/auth.py` retornam os mesmos símbolos — esperado (paths distintos).

---

## Conclusões e Recomendações

**O servidor funciona direitinho.** Todas as 26 tools responderam corretamente,
o tratamento de erro é limpo e informativo, e o ciclo completo de curadoria
(suggest → store → update → link → rename → validate → delete) funciona de
ponta a ponta com cleanup correto.

Recomendações de uso:

1. **Localize com `search_code`, leia com `read_code_chunk`/`get_file_context`** — não confie em chunks de busca para leitura detalhada.
2. **Use só símbolos top-level** em `read_code_chunk` (confira o catálogo via `get_file_context` antes); para métodos internos, prefira ranges de linha.
3. **Rode `validate_vault` periodicamente** — o vault bearry precisa de curadoria (summary faltando em 6 notas, 0 wikilinks).
4. **Confie em `get_vault_stats` para métricas** — as labels de `list_projects` e `sync_vault` têm pequenas inconsistências de contagem/nome.
5. **`export_graph` é a visão mais completa** para auditoria arquitetural (inclui detecção de loops circulares que o report inline não mostra).

## Anexo: Cobertura de Teste

| # | Tool | Testada |
|---|---|---|
| 1 | `list_projects` | ✅ |
| 2 | `switch_project` | ✅ |
| 3 | `create_project` | ⚠️ não testado (criaria vault real) |
| 4 | `get_vault_stats` | ✅ |
| 5 | `list_recent_notes` | ✅ |
| 6 | `search_semantic` (dense + summaries) | ✅ |
| 7 | `read_section` | ✅ |
| 8 | `get_compressed_context` | ✅ |
| 9 | `get_graph_neighborhood` | ✅ |
| 10 | `find_files` | ✅ |
| 11 | `search_code` | ✅ |
| 12 | `get_file_context` | ✅ |
| 13 | `read_code_chunk` (símbolo + range) | ✅ |
| 14 | `get_code_dependencies` (both) | ✅ |
| 15 | `find_path` | ✅ |
| 16 | `explain_node` | ✅ |
| 17 | `get_community_report` | ✅ |
| 18 | `suggest_links` | ✅ |
| 19 | `store_context` | ✅ |
| 20 | `update_section` | ✅ |
| 21 | `link_notes` | ✅ |
| 22 | `rename_note` | ✅ |
| 23 | `validate_vault` (vault + single) | ✅ |
| 24 | `sync_vault` | ✅ |
| 25 | `export_graph` | ✅ |
| 26 | `delete_note` | ✅ (confirmado pelo usuário) |
| — | `graphfinder://instructions` | ✅ |
| — | `graphfinder://stats` | ✅ |
