export const MCP_CONFIG_SNIPPET = `{
  "mcpServers": {
    "graphfinder": {
      "command": "Z:\\\\GraphFinder\\\\src-tauri\\\\target\\\\debug\\\\graphfinder.exe",
      "args": ["--mcp"],
      "env": {
        "GRAPHFINDER_VAULT": "Z:\\\\GraphFinder\\\\vault",
        "GRAPHFINDER_DB": "Z:\\\\GraphFinder\\\\index.db"
      }
    }
  }
}`;

export const AGENT_RULE_CONTENT = `# GraphFinder: Diretrizes de Conhecimento, Escrita e MCP

Este documento define as regras obrigatórias para agentes de IA, editores e desenvolvedores que operam ou gerenciam o cofre de notas do **GraphFinder**. O objetivo é garantir **máxima densidade semântica**, **economia extrema de tokens** e **alta conectividade no grafo**.

---

## 1. Princípios Fundamentais de Arquitetura

1. **Local-First & Markdown Puro**: Todas as notas vivem em arquivos \`.md\` no diretório \`./vault\` e devem ser 100% legíveis por humanos e compatíveis com Obsidian.
2. **Chunking Semântico Atômico**: O motor FastEmbed/SQLite vetoriza as notas dividindo-as por cabeçalhos (\`##\` e \`###\`). Cada cabeçalho gera um vetor independente na tabela \`note_chunks\`.
3. **Economia de Tokens**: Agentes de IA **NUNCA** devem solicitar ou injetar arquivos Markdown inteiros no contexto da LLM, a menos que estritamente requisitado pelo usuário. Sempre prefira trechos atômicos (\`[[Nota#Seção]]\`).

---

## 2. Padrões de Escrita e Estruturação de Notas

Toda nota criada ou editada no cofre DEVE seguir rigorosamente a seguinte estrutura:

### A. Título e Resumo Executivo
* A nota deve iniciar com um cabeçalho H1 (\`# Título da Nota\`).
* O primeiro parágrafo após o H1 deve ser um resumo executivo de alta densidade (máximo 2 a 3 linhas), sintetizando o propósito do documento.

### B. Granularidade de Cabeçalhos (\`##\` e \`###\`)
* Divida o conteúdo em tópicos autocontidos de **40 a 150 tokens** cada.
* Dê nomes semânticos e descritivos aos cabeçalhos (ex: \`## 2. Configuração do SQLite WAL\` em vez de apenas \`## Config\`).
* O motor de busca usa esses títulos para criar deep links precisos: \`[[Nota#2. Configuração do SQLite WAL]]\`.

### C. Conexões Bidirecionais Obrigatórias (Wikilinks)
* Conecte termos e conceitos a outras notas existentes usando a sintaxe \`[[Nome da Nota]]\` ou \`[[Nome da Nota#Seção]]\`.
* Cada nota deve possuir no mínimo **1 a 3 conexões** com outras partes do grafo para evitar nós órfãos.
* Sempre verifique conexões potenciais chamando a ferramenta MCP \`suggest_links\`.

### D. Taxonomia de Tags
* Utilize hashtags \`#tag\` no final das seções ou da nota.
* Use termos em minúsculas (ex: \`#arquitetura\`, \`#sqlite\`, \`#rag\`, \`#mcp\`).
* O frontend do GraphFinder usa essas tags para colorir e agrupar visualmente os clusters do grafo.

---

## 3. Protocolo de Operação para Agentes de IA via MCP

Ao atuar como agente conectado ao servidor MCP do GraphFinder, siga este pipeline de decisão para poupar contexto:

\`\`\`
[Pergunta do Usuário]
        │
        ▼
1. Executar \`search_semantic(query)\`
   - Retorna os 5 trechos mais relevantes (~50 tokens cada).
        │
        ├─► Resposta encontrada? ──► Responder citando os deep links [[Nota#Seção]].
        │
        └─► Precisa de visão geral? ──► Executar \`get_compressed_context(note_id)\`
                                         (Resumo + mapa de seções com tokens).
\`\`\`

### Regras de Ouro do MCP:
1. **Nunca leia o arquivo bruto sem necessidade**: Use \`search_semantic\` primeiro. Ele já extrai a subseção exata onde a informação reside.
2. **Ao salvar novo conhecimento (\`store_context\`)**:
   - Forneça título claro.
   - Forneça conteúdo bem formatado com seções \`##\`.
   - Adicione wikilinks explícitos para notas que você consultou anteriormente.
   - Forneça lista de \`tags\`.
3. **Ao sugerir links (\`suggest_links\`)**:
   - Passe o rascunho do texto antes de salvá-lo para descobrir afinidades com notas já existentes no cofre.

---

## 4. Multi-Vaults & Suporte a Múltiplos Projetos

O GraphFinder suporta múltiplos projetos isolados (\`./vaults/<projeto_id>\`), cada um com seu próprio cofre Markdown e banco de dados SQLite vetorial independente:

- \`list_projects()\`: Lista todos os projetos disponíveis, caminhos e contagem de notas.
- \`create_project(name)\`: Cria um novo cofre isolado em \`./vaults/<slug>/\` com nota inicial.
- \`switch_project(project_id)\`: Alterna o cofre padrão ativo do GraphFinder.
- **Acesso Direto por Vault**: As ferramentas de consulta e armazenamento (\`search_semantic\`, \`store_context\`, \`get_compressed_context\`, \`get_graph_neighborhood\`, \`suggest_links\`, \`link_notes\`) aceitam o parâmetro opcional \`project: "<id>"\`, permitindo que agentes operem diretamente em múltiplos cofres sem contaminação cruzada.`;

export const SKILL_CONTENT = `---
name: graphfinder-curator
description: >-
  Guides authoring, knowledge curation, semantic linking, and high-performance MCP retrieval in GraphFinder.
  Use when creating or updating markdown notes in the vault, querying the knowledge graph, configuring MCP tools,
  or optimizing token efficiency for AI agents.
---

# GraphFinder Curator Skill

Esta skill guia agentes e editores no gerenciamento do cofre de conhecimento do **GraphFinder**, garantindo máxima economia de tokens, preservação do grafo semântico, múltiplos projetos/vaults e fatiamento correto por seções atômicas.

---

## Quando Usar Esta Skill
- Ao redigir ou estruturar novas notas no diretório \`./vault\` ou em qualquer cofre isolado de projeto (\`./vaults/<projeto>\`).
- Ao alternar entre múltiplos projetos com \`list_projects\`, \`switch_project\` ou criar novos com \`create_project\`.
- Ao interagir com o GraphFinder através do servidor MCP (stdio).
- Ao realizar buscas semânticas sem estourar o limite de tokens da LLM.
- Ao solicitar ou aplicar sugestões automáticas de links no grafo.

---

## Fluxo de Leitura e Consulta (Recuperação de Baixo Custo)

Quando o usuário pedir para você consultar ou responder a partir do conhecimento armazenado:

1. **Passo 1: Busca Semântica por Trechos**
   - Execute a tool MCP \`search_semantic(query)\`.
   - Avalie os trechos retornados com seus respectivos scores de afinidade e contagens de tokens (~50-100 tokens cada).
   - Identifique os deep links no formato \`[[Nota#Seção]]\`.

2. **Passo 2: Exploração de Vizinhança (Opcional)**
   - Se o trecho mencionar outras notas conectadas ou se você precisar de contexto adjacente, chame \`get_graph_neighborhood(note_id, depth: 1)\`.
   - Isso revela conexões de entrada (*backlinks*) e saída sem ler o corpo das notas.

3. **Passo 3: Sumário Estrutural (Se necessário)**
   - Se precisar saber a estrutura inteira de uma nota sem gastar tokens com o texto completo, chame \`get_compressed_context(note_id)\`.
   - Isso traz o resumo em 2 linhas e a lista de seções com suas contagens de tokens.

---

## Fluxo de Escrita e Curadoria (Adicionando Conhecimento)

Ao criar ou atualizar notas no cofre:

### 1. Estruturação em Seções Atômicas
O motor semântico vetoriza os arquivos fatiando-os por cabeçalhos. Cada seção \`##\` ou \`###\` deve:
- Conter um único conceito claro.
- Ter entre 30 e 150 tokens.
- Possuir um título descritivo que faça sentido isoladamente (ex: \`## 2. Configuração do SQLite WAL\` em vez de \`## Configuração\`).

### 2. Auto-Linker Pré-Gravação
Antes de salvar a nota:
1. Pegue o texto ou parágrafo principal e passe para \`suggest_links(text)\`.
2. O motor retornará notas ou seções existentes com alta similaridade vetorial (ex: \`Score: 84%\`).
3. Incorpore essas referências no texto usando a sintaxe \`[[Nome da Nota#Seção]]\`.

### 3. Registro no Cofre
- Se operando via MCP: chame \`store_context(title, content, tags)\`.
- Se editando diretamente o sistema de arquivos: salve o arquivo em \`./vault/<Título>.md\` e acione \`sync_vault\` ou deixe a sincronização automática indexar.

---

## Documentação e Referências Adicionais

Para modelos prontos e especificações detalhadas:
- [Templates de Notas](./references/note-templates.md): Modelos prontos para Conceitos, Decisões de Arquitetura (ADR) e Tutoriais.
- [Especificação de Ferramentas MCP](./references/mcp-spec.md): Parâmetros, retornos e payloads de cada comando MCP.`;
