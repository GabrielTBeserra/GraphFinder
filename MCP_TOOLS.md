# Ferramentas (Tools) do MCP Server do GraphFinder

Todas as ferramentas abaixo estão **100% implementadas e ativas** no executável `graphfinder.exe --mcp`.

---

## 1. `search_semantic` (Busca Semântica por Seções)
* **Objetivo**: Recuperação cirúrgica de conhecimento com máxima economia de tokens.
* **Como funciona**: Em vez de devolver notas inteiras, busca na tabela `note_chunks` e retorna apenas os 5 trechos mais afins (2 a 4 linhas cada), indicando o deep link `[[Nota#Seção]]` e a contagem estimada de tokens.
* **Parâmetros**:
  ```json
  {
    "query": "string (termo ou pergunta em linguagem natural)",
    "limit": "integer (opcional, padrão: 5)",
    "format": "dense_chunks | summaries (opcional, padrão: dense_chunks)"
  }
  ```

---

## 2. `get_compressed_context` (Contexto Comprimido)
* **Objetivo**: Fornecer a estrutura completa de uma nota consumindo menos de 100 tokens.
* **Como funciona**: Retorna um resumo executivo de 2 linhas, as conexões no grafo (backlinks e outgoing) e a lista de todas as seções H2/H3 com suas respectivas estimativas de tokens.
* **Parâmetros**:
  ```json
  {
    "target": "string (título ou ID da nota)"
  }
  ```

---

## 3. `suggest_links` (Auto-Linker Semântico)
* **Objetivo**: Conectar pensamentos e evitar notas isoladas no grafo.
* **Como funciona**: Analisa um texto ou rascunho em tempo real, calcula o embedding vetorial e compara com todos os nós e seções do cofre, retornando conexões recomendadas com score de afinidade.
* **Parâmetros**:
  ```json
  {
    "text": "string (parágrafo ou rascunho de texto)",
    "current_note": "string (opcional, ID da nota atual para ignorar auto-referência)",
    "threshold": "number (opcional, padrão: 0.60)"
  }
  ```

---

## 4. `store_context` (Gravação de Memória)
* **Objetivo**: Registrar ou atualizar notas no cofre a partir da IA.
* **Como funciona**: Salva o arquivo `.md` no vault, divide o texto por cabeçalhos (`##` e `###`), calcula os embeddings locais offline e indexa na tabela `note_chunks`.
* **Parâmetros**:
  ```json
  {
    "title": "string (título da nota)",
    "content": "string (conteúdo formatado em Markdown)",
    "tags": ["array de strings (opcional)"],
    "links": ["array de notas para conectar com wikilinks (opcional)"]
  }
  ```

---

## 5. `get_graph_neighborhood` (Expansão no Grafo)
* **Objetivo**: Navegar pela topologia de relações sem ler o corpo dos arquivos.
* **Como funciona**: Retorna todos os nós conectados a uma nota em um raio de 1 ou 2 saltos.
* **Parâmetros**:
  ```json
  {
    "target": "string (título ou ID da nota)",
    "depth": "integer (1 ou 2)"
  }
  ```

---

## 6. `link_notes` (Criação de Ligação)
* **Objetivo**: Criar uma conexão explícita entre duas notas no cofre.
* **Como funciona**: Adiciona a sintaxe `[[Nome da Nota]]` no final do arquivo de origem e atualiza o grafo.
* **Parâmetros**:
  ```json
  {
    "from_note": "string (nota de origem)",
    "to_note": "string (nota de destino)"
  }
  ```

---

## 7. `list_recent_notes` (Listagem Rápida)
* **Objetivo**: Mapeamento inicial do cofre pela IA.
* **Como funciona**: Lista os títulos, tags e resumos ultracompactos das últimas notas modificadas.
* **Parâmetros**:
  ```json
  {
    "limit": "integer (opcional, padrão: 10)"
  }
  ```
