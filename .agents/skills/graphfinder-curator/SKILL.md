---
name: graphfinder-curator
description: >-
  Guides authoring, knowledge curation, semantic linking, and high-performance MCP retrieval in GraphFinder.
  Use when creating or updating markdown notes in the vault, querying the knowledge graph, configuring MCP tools,
  or optimizing token efficiency for AI agents.
---

# GraphFinder Curator Skill

Esta skill guia agentes e editores no gerenciamento do cofre de conhecimento do **GraphFinder**, garantindo máxima economia de tokens, preservação do grafo semântico e fatiamento correto por seções atômicas.

---

## Quando Usar Esta Skill
- Ao redigir ou estruturar novas notas no diretório `./vault` ou em vaults de projeto (`./vaults/<projeto>`).
- Ao alternar entre múltiplos projetos com `list_projects`, `switch_project` ou criar novos com `create_project`.
- Ao interagir com o GraphFinder através do servidor MCP (stdio).
- Ao realizar buscas semânticas sem estourar o limite de tokens da LLM.
- Ao solicitar ou aplicar sugestões automáticas de links no grafo.

---

## Fluxo de Leitura e Consulta (Recuperação de Baixo Custo)

Quando o usuário pedir para você consultar ou responder a partir do conhecimento armazenado:

1. **Passo 1: Busca Semântica por Trechos**
   - Execute a tool MCP `search_semantic(query)`.
   - Avalie os trechos retornados com seus respectivos scores de afinidade e contagens de tokens (~50-100 tokens cada).
   - Identifique os deep links no formato `[[Nota#Seção]]`.

2. **Passo 2: Exploração de Vizinhança (Opcional)**
   - Se o trecho mencionar outras notas conectadas ou se você precisar de contexto adjacente, chame `get_graph_neighborhood(note_id, depth: 1)`.
   - Isso revela conexões de entrada (*backlinks*) e saída sem ler o corpo das notas.

3. **Passo 3: Sumário Estrutural (Se necessário)**
   - Se precisar saber a estrutura inteira de uma nota sem gastar tokens com o texto completo, chame `get_compressed_context(note_id)`.
   - Isso traz o resumo em 2 linhas e a lista de seções com suas contagens de tokens.

---

## Fluxo de Escrita e Curadoria (Adicionando Conhecimento)

Ao criar ou atualizar notas no cofre:

### 1. Estruturação em Seções Atômicas
O motor semântico vetoriza os arquivos fatiando-os por cabeçalhos. Cada seção `##` ou `###` deve:
- Conter um único conceito claro.
- Ter entre 30 e 150 tokens.
- Possuir um título descritivo que faça sentido isoladamente (ex: `## 2. Configuração do SQLite WAL` em vez de `## Configuração`).

### 2. Auto-Linker Pré-Gravação
Antes de salvar a nota:
1. Pegue o texto ou parágrafo principal e passe para `suggest_links(text)`.
2. O motor retornará notas ou seções existentes com alta similaridade vetorial (ex: `Score: 84%`).
3. Incorpore essas referências no texto usando a sintaxe `[[Nome da Nota#Seção]]`.

### 3. Registro no Cofre
- Se operando via MCP: chame `store_context(title, content, tags)`.
- Se editando diretamente o sistema de arquivos: salve o arquivo em `./vault/<Título>.md` e acione `sync_vault` ou deixe a sincronização automática indexar.

---

## Documentação e Referências Adicionais

Para modelos prontos e especificações detalhadas:
- [Templates de Notas](./references/note-templates.md): Modelos prontos para Conceitos, Decisões de Arquitetura (ADR) e Tutoriais.
- [Especificação de Ferramentas MCP](./references/mcp-spec.md): Parâmetros, retornos e payloads de cada comando MCP.
