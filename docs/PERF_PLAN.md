# Plano de Otimização — Projetos de Grande Porte (50k+ nós)

Objetivo: eliminar o travamento do gráfico e reduzir o tempo de sync de ~45min para segundos (estrutura) + background (embeddings).

Cada tarefa abaixo foi dimensionada para caber em **um commit independente**.

---

## T01 — Frontend: destravar o loop de renderização do GraphView
**Arquivo:** `src/components/GraphView.tsx` — commit `8180280`
- `computeConvexHull`/`drawSmoothHull` fora do `onRenderFramePre`: hulls cacheados por comunidade, recomputados no máx. a cada 400ms e pulados quando N > 2000.
- `currentKey` (`.map().sort().join(';')`) substituída por assinatura FNV-1a de passe único O(N+E).
- Campos pré-normalizados por nó (`_idLc`, `_idStemLc`, `_titleLc`, `_titleStemLc`, `_pathLc`) + memoização da seleção — zero toLowerCase/regex por frame.

## T02 — Frontend: LOD + limites para grafos enormes
**Arquivo:** `src/components/GraphView.tsx` — commit `cff87fd`
- LOD: zoom < 0.6 ou N > 2000 → círculo sólido simples (sem gradiente, shadowBlur, pill, texto). Nós selecionados/hovered/glow/God/path mantêm detalhe.
- Física escalonada: `warmupTicks=50`, `cooldownTicks=160`, `alphaMin=0.02`, `alphaDecay=0.05`, `velocityDecay=0.55` para grafos grandes.
- Força `cluster` desativada quando N > 2000.
- Cap de densidade: N > 2000 exibe apenas hubs (degree >= 2, god nodes, seleção) por padrão.

## T03 — Backend: eliminar N+1 e O(N²) nas resoluções do SQLite
**Arquivo:** `src-tauri/src/engine/db.rs` — commit `10fd75a`
- `resolve_call_graph`: mapas símbolo→arquivos e caller→imports carregados uma vez, resolução em memória, aplicada via temp table + UPDATE único (antes: ~1,5M subqueries correlacionadas).
- `resolve_pending_code_relations`: mapa símbolo→arquivos único (antes: um SELECT por linha).

## T04 — Backend: parsing paralelo com rayon
**Arquivos:** `src-tauri/Cargo.toml`, `src-tauri/src/engine/vault.rs` — commit `9ce8b93`
- `rayon = "1.10"` adicionado; parsing de notas e arquivos de código via `into_par_iter()`.

## T05 — Backend/Frontend: Two-Stage Sync
**Arquivos:** `src-tauri/src/engine/vault.rs`, `src/App.tsx` — commit `cc8b586`
- Após o bulk-save estrutural (>25 arquivos) emite `vault-structure-ready`: grafo, lista de arquivos e FTS5 consultáveis em segundos enquanto embeddings rodam em background.
- Cap de chunks embedados por arquivo escalonado: >5000 arquivos → 1, >80 → 4, senão 16 (env `GRAPHFINDER_EMBED_CHUNKS` sobrescreve).
- App.tsx escuta `vault-structure-ready` e recarrega dados.

## T06 — Verificação
- `npx tsc -b` OK · `cargo check` OK · `cargo test`: 57/57 OK · `npm run lint`: sem erros novos (warnings pré-existentes).

---

## Status
- [x] T01
- [x] T02
- [x] T03
- [x] T04
- [x] T05
- [x] T06
