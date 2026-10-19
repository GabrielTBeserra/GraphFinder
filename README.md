<div align="center">

<br/>

<img src="./assets/logo.svg" alt="GraphFinder Logo" width="128" height="128" />

# GraphFinder

### Neural Knowledge Graph & High-Density Semantic Vault for Humans & AI Agents

<p align="center">
  <b>Local-First • Plain Markdown • Atomic Semantic Chunking • Native MCP Server • Obsidian Compatible</b>
</p>

[![Release](https://img.shields.io/badge/release-v0.2.0-18181b?style=flat-square)](https://github.com)
[![Tauri](https://img.shields.io/badge/Tauri-v2.12-18181b?style=flat-square&logo=tauri&logoColor=white)](https://tauri.app)
[![Rust](https://img.shields.io/badge/Rust-2021-18181b?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org)
[![React](https://img.shields.io/badge/React-19.2-18181b?style=flat-square&logo=react&logoColor=white)](https://react.dev)
[![TypeScript](https://img.shields.io/badge/TypeScript-6.0-18181b?style=flat-square&logo=typescript&logoColor=white)](https://www.typescriptlang.org)
[![MCP](https://img.shields.io/badge/MCP-Server_Native-18181b?style=flat-square)](https://modelcontextprotocol.io)
[![License](https://img.shields.io/badge/License-MIT-18181b?style=flat-square)](LICENSE)

<br/>

<a href="#visao-geral">Visao Geral</a> •
<a href="#principais-recursos">Recursos</a> •
<a href="#servidor-mcp">Servidor MCP</a> •
<a href="#arquitetura">Arquitetura</a> •
<a href="#instalacao-e-uso">Instalacao</a> •
<a href="#multi-vaults">Multi-Vaults</a> •
<a href="#curadoria-de-notas">Curadoria</a> •
<a href="#contribuicao-e-comunidade">Contribuicao</a> •
<a href="#licenca">Licenca</a>

<br/><br/>

<img src="./assets/banner.svg" alt="GraphFinder Interface Overview" width="100%" />

</div>

---

## Visao Geral

O **GraphFinder** e uma aplicacao desktop de alta performance projetada para unir o fluxo de pensamento humano a precisao de agentes de Inteligencia Artificial.

A maioria dos sistemas de RAG tradicionais fatiam documentos arbitrariamente ou consomem janelas inteiras de contexto de LLMs com milhares de tokens desnecessarios. Ao mesmo tempo, gerenciadores de notas pessoais convencionais nao fornecem vetorizacao local nem APIs semanticas atomicas.

O **GraphFinder** soluciona esse descompasso:
1. **Para Humanos**: Um ambiente local-first, minimalista e fluido para redigir em Markdown puro, visualizar conexoes atraves de um grafo interativo e manter controle absoluto sobre seus arquivos locais.
2. **Para Agentes de IA**: Um servidor MCP embutido que entrega recuperacao cirurgica no nivel de subsecao (`[[Nota#Secao]]`), gastando **~50 tokens por consulta** e garantindo zero poluicao de contexto.

---

## Principais Recursos

### 1. Chunking Semantico Atomico
* O motor vetorial em Rust fatia os arquivos exclusivamente por cabecalhos logicos (`##` e `###`).
* Cada secao gera um vetor independente via **FastEmbed** (ONNX local, sem dependencia de nuvem ou chaves de API).
* O SQLite em modo WAL armazena e recupera metadados e chunks de maneira concorrente e imediata.

### 2. Grafo Interativo 2D
* Renderizacao limpa com fisica de forcas via Canvas/WebGL (`force-graph`).
* Destaque topologico de vizinhanca e calculo de densidade de conexoes.
* Agrupamento visual por taxonomia de hashtags (`#arquitetura`, `#sqlite`, `#mcp`).

### 3. Servidor MCP Nativo (`--mcp`)
* O executavel desktop suporta nativamente o modo MCP via `stdio`.
* Disponibiliza 7 ferramentas prontas para clientes como Claude Desktop, Cursor, Gemini Antigravity, Windsurf e Zed.

### 4. Multi-Vaults Isolados
* Suporte nativo a multiplos cofres (`./vaults/<projeto_id>`), cada um com seu cofre Markdown e banco de dados vetorial isolado.
* Zero contaminacao cruzada entre clientes, pesquisas ou bases de conhecimento.

### 5. Local-First e Sem Vendor Lock-in
* Seus arquivos permanecem salvos em `.md` no disco local.
* 100% compativel com Obsidian — voce pode apontar o Obsidian e o GraphFinder para o mesmo diretorio.
* Nenhuma informacao confidencial trafega para servidores externos.

---

## Servidor MCP

O protocolo Model Context Protocol (MCP) permite que agentes de IA interajam com sua base de notas com minima sobrecarga de contexto:

```
[Pergunta do Usuario / Agente]
            |
            v
   1. search_semantic(query)
      +--> Retorna ate 5 subsecoes atomicas (~50 tokens cada).
            |
            +--> Trecho suficiente? ---> Responde citando [[Nota#Secao]].
            |
            +--> Necessita do panorama da nota? ---> get_compressed_context(note_id)
                                                      (Resumo executivo + mapa de secoes).
```

### Ferramentas Integradas

| Ferramenta | Descricao | Parametros Principais | Economia de Tokens |
| :--- | :--- | :--- | :--- |
| `search_semantic` | Busca por similaridade vetorial na tabela de chunks | `query`, `limit`, `format` | **~90%** vs ler notas inteiras |
| `get_compressed_context` | Resumo executivo, backlinks e lista de secoes | `target` (ID ou titulo) | Nota completa em **< 100 tokens** |
| `suggest_links` | Analisa rascunho de texto e recomenda conexoes | `text`, `threshold` | Evita notas orfas no grafo |
| `store_context` | Grava nota Markdown com chunking e vetorizacao imediata | `title`, `content`, `tags`, `links` | Memoria persistente local |
| `get_graph_neighborhood` | Retorna nos conectados em 1 ou 2 saltos | `target`, `depth` | Navegacao topologica rapida |
| `link_notes` | Adiciona wikilink bidirecional entre duas notas | `from_note`, `to_note` | Edicao estruturada direta |
| `list_recent_notes` | Lista as notas mais recentes com resumos compactos | `limit` | Mapeamento inicial do vault |

> Todas as ferramentas aceitam o parametro opcional `project: "<id>"` para consultar cofres alternativos.

---

### Configuracao nos Agentes

#### Claude Desktop (`claude_desktop_config.json`)
```json
{
  "mcpServers": {
    "graphfinder": {
      "command": "cargo",
      "args": [
        "run",
        "--manifest-path",
        "Z:/GraphFinder/src-tauri/Cargo.toml",
        "--",
        "--mcp"
      ]
    }
  }
}
```
*(Ou aponte para o executavel compilado `graphfinder.exe --mcp`)*

#### Cursor / Gemini Antigravity / Windsurf (`mcp_config.json`)
```json
{
  "mcpServers": {
    "graphfinder": {
      "command": "Z:/GraphFinder/src-tauri/target/release/graphfinder.exe",
      "args": ["--mcp"]
    }
  }
}
```

---

## Arquitetura

```mermaid
flowchart TB
    subgraph Frontend ["Frontend (Desktop & Web)"]
        React["React 19 + TypeScript"]
        ForceGraph["Force Graph Canvas"]
        Editor["Markdown Editor & Parser"]
        VaultSwitcher["Multi-Vault Switcher"]
    end

    subgraph RustCore ["Rust Core (Tauri v2)"]
        IPC["Tauri IPC Commands"]
        MCPServer["Native MCP Server (--mcp)"]
        FastEmbedEngine["FastEmbed (ONNX Runtime)"]
        SQLiteEngine["SQLite WAL Engine"]
        Chunker["Atomic H2/H3 Chunker"]
    end

    subgraph Storage ["Armazenamento Local"]
        VaultDir["Cofre Padrao (./vault/*.md)"]
        VaultsDir["Projetos Isolados (./vaults/*)"]
        DB["index.db (Embeddings & Metadados)"]
    end

    Frontend <-->|Tauri IPC| RustCore
    MCPServer <-->|JSON-RPC Stdio| AI["Agentes Externos (Claude, Cursor, etc.)"]
    RustCore <--> Storage
```

### Tecnologias
* **Desktop Runtime**: [Tauri v2](https://tauri.app/)
* **Backend de Dados & Vetores**: [Rust](https://www.rust-lang.org/), [FastEmbed](https://github.com/Anush008/fastembed-rs), [Rusqlite](https://github.com/rusqlite/rusqlite)
* **Frontend UI**: [React 19](https://react.dev/), [TypeScript](https://www.typescriptlang.org/), [Vite](https://vitejs.dev/)
* **Grafos**: [Force-Graph](https://github.com/vasturiano/force-graph)

---

## Instalacao e Uso

### Pre-requisitos
* **Node.js** 18+ ([nodejs.org](https://nodejs.org/))
* **Rust & Cargo** 1.75+ ([rustup.rs](https://rustup.rs/))
* No Windows: Ferramentas de compilacao C++ (Visual Studio Build Tools)

### 1. Clonar e Instalar Dependencias
```bash
git clone https://github.com/SEU_USUARIO/GraphFinder.git
cd GraphFinder
npm install
```

### 2. Executar o Aplicativo Desktop
```bash
npm run app
```

### 3. Executar o Servidor MCP
```bash
npm run mcp
```

### 4. Compilar para Producao
```bash
npm run tauri build
```
Os executaveis finais serao gerados em `src-tauri/target/release/bundle/`.

---

## Multi-Vaults

Estrutura de diretorios do projeto:

```
GraphFinder/
├── assets/                  # Identidade visual vetorial e assets do repositorio
├── public/                  # Icones estaticos e favicon
├── src/                     # Codigo React / TypeScript
│   ├── components/          # Componentes visuais (GraphView, NoteEditor, etc.)
│   ├── services/            # Servicos de API e comunicacao IPC
│   └── index.css            # Design System minimalista
├── src-tauri/               # Backend Rust e Servidor MCP
│   ├── src/                 # Logica de embeddings, SQLite e comandos
│   └── Cargo.toml           # Manifesto do Rust
├── vault/                   # Cofre principal de notas Markdown
└── vaults/                  # Cofres adicionais isolados por projeto
```

---

## Curadoria de Notas

Para maximizar a precisao da recuperacao vetorial e a economia de contexto:

1. **H1 com Titulo e Resumo**: Inicie o documento com `# Titulo da Nota` e um resumo direto de 2 a 3 linhas.
2. **Subsecoes H2 e H3**: Mantenha blocos concisos entre **40 e 150 tokens**. Use titulos semanticos claros (ex: `## 2. Configuracao do SQLite WAL`).
3. **Wikilinks**: Utilize `[[Nome da Nota]]` ou `[[Nome da Nota#Secao]]` para tecer o grafo e evitar nos orfaos.
4. **Hashtags de Taxonomia**: Adicione tags em minusculas (ex: `#arquitetura #mcp #rust`) no final dos topicos para clusterizacao visual.

---

## Contribuicao e Comunidade

O GraphFinder e um projeto 100% open source. Contribuicoes de codigo, documentacao, relatorios de bugs e propostas de novas ferramentas MCP sao muito bem-vindas.

* **Guia de Contribuicao**: Consulte o [CONTRIBUTING.md](CONTRIBUTING.md) para configuracao do ambiente local e padroes de commit.
* **Codigo de Conduta**: Seguimos o padrao [Contributor Covenant](CODE_OF_CONDUCT.md).
* **Seguranca**: Leia a nossa [Politica de Seguranca](SECURITY.md) para reporte responsavel de vulnerabilidades.
* **Historico de Versoes**: Acompanhe as novidades e melhorias no [CHANGELOG.md](CHANGELOG.md).

---

## Licenca

Distribuido sob a licenca [MIT](LICENSE). O codigo e livre para estudo, modificacao e distribuicao.

---

<div align="center">
  <sub>Construido com foco em privacidade local, economia de tokens e arquitetura de conhecimento.</sub>
</div>
