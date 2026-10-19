# Arquitetura do Sistema

A arquitetura do GraphFinder é dividida em três pilares principais:

## 1. Armazenamento Físico e SQLite
- **Arquivos Markdown (`.md`)**: Localizados no diretório local do vault. Legíveis por humanos, compatíveis com Obsidian.
- **SQLite Local**: Guarda a tabela de nós, arestas (links bidirecionais extraídos do markdown) e vetores de embedding calculados localmente.

## 2. Motor Semântico em Rust
- Utiliza **FastEmbed** para calcular representações vetoriais semânticas via ONNX Runtime local.
- Expõe algoritmos de grafo (k-hop expansion) para extrair subgrafos relevantes ao redor de uma pesquisa ou nota.
- Veja mais em [[Boas-vindas]].

## 3. Protocolo MCP (Model Context Protocol)
- Permite que ferramentas de IA usem comandos como `store_context` e `search_semantic`.
- Saiba como interagir em [[Guia de Uso]].

Tags: #arquitetura #rust #sqlite #embeddings #mcp

- Conexão Semântica: [[Boas-vindas ao GraphFinder#O que torna o GraphFinder especial?]]
- Conexão Semântica: [[Guia de Uso#Usando via MCP]]