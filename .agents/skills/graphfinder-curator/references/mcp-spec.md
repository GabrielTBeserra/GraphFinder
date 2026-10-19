# Especificação das Ferramentas MCP do GraphFinder

O GraphFinder expõe ferramentas compatíveis com o protocolo **Model Context Protocol (JSON-RPC 2.0)** sobre stdio.

---

## 1. `search_semantic` (Busca Atômica por Trechos)
A ferramenta primária para recuperação de conhecimento com economia de tokens.

* **Parâmetros**:
  - `query` (string, obrigatório): Texto da busca ou pergunta em linguagem natural.
  - `limit` (número, opcional): Quantidade de trechos (padrão: 5).
* **Retorno**:
  Lista de seções mais afins, indicando deep link `[[Nota#Seção]]`, porcentagem de afinidade e estimativa de tokens do trecho.

---

## 2. `get_compressed_context` (Compressão Estrutural)
Retorna uma visão ultracompacta de uma nota sem carregar o texto completo.

* **Parâmetros**:
  - `note_id` (string, obrigatório): Nome ou identificador da nota.
* **Retorno**:
  - Resumo de alto nível (~30 tokens).
  - Backlinks e links de saída.
  - Lista de todas as seções H2/H3 com contagem estimada de tokens de cada uma.

---

## 3. `suggest_links` (Auto-Linker Semântico)
Identifica notas e seções existentes no cofre que possuem alta correlação temática com o texto fornecido.

* **Parâmetros**:
  - `text` (string, obrigatório): Parágrafo ou rascunho de conteúdo.
  - `threshold` (número, opcional): Similaridade mínima de corte (ex: 0.65).
* **Retorno**:
  Lista de sugestões com sintaxe recomendada de wikilink (ex: `[[Arquitetura do Sistema#1. SQLite]]` com Score 81%).

---

## 4. `store_context` (Gravação de Memória)
Salva uma nova nota ou atualiza uma nota existente com reindexação semântica imediata.

* **Parâmetros**:
  - `title` (string, obrigatório): Título da nota (usado no nome do arquivo `.md`).
  - `content` (string, obrigatório): Conteúdo em Markdown fatiado em seções `##`.
  - `tags` (array de strings, opcional): Tags temáticas.
* **Retorno**:
  Confirmação de gravação e quantidade de seções atômicas indexadas no banco vetorial.

---

## 5. `get_graph_neighborhood` (Expansão no Grafo)
Retorna a vizinhança topológica de um nó.

* **Parâmetros**:
  - `note_id` (string, obrigatório): Identificador do nó central.
  - `depth` (número, opcional): Profundidade do raio de busca (padrão: 1).
* **Retorno**:
  Lista de nós conectados, arestas direcionadas e tags relacionadas.

---

## 6. `link_notes` (Criação de Aresta)
Cria uma conexão bidirecional entre duas notas, inserindo o wikilink correspondente no arquivo Markdown de origem.

* **Parâmetros**:
  - `from_note` (string, obrigatório): Nota de origem.
  - `to_note` (string, obrigatório): Nota de destino.
