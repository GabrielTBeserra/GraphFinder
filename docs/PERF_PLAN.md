# Plano de Otimização — Projetos de Grande Porte (50k+ nós)

Objetivo: eliminar o travamento do gráfico e reduzir o tempo de sync de ~45min para segundos (estrutura) + background (embeddings).

Cada tarefa abaixo foi dimensionada para caber em **um commit independente**.

---

## T01 — Frontend: destravar o loop de renderização do GraphView
**Arquivo:** `src/components/GraphView.tsx`
- Remover `computeConvexHull`/`drawSmoothHull` de `onRenderFramePre` (executado 60x/s, ~8×N pontos).
  - Cachear hulls por comunidade, recalcular apenas quando posições mudarem significativamente ou na parada da física.
- Substituir a mega-chave `currentKey` (`.map().sort().join(';')` sobre todos os nós/arestas) por comparador leve (contagem + versão + hash FNV incremental).
- Reduzir custo de `nodeCanvasObject`: pré-normalizar `id`/`title` lowercase uma única vez por nó (não por frame).

## T02 — Frontend: LOD + limites para grafos enormes
**Arquivo:** `src/components/GraphView.tsx`
- LOD por `globalScale`: abaixo de 0.6 (ou N > 2000) renderizar só círculos sólidos; sem gradiente, shadowBlur, roundRect, measureText.
- Tuning da física para N grande: `alphaMin`, `cooldownTicks`, `warmupTicks`; desativar força `cluster` quando N > 2000.
- Amostragem/filtro automático: para N > LIMITE, exibir por padrão apenas nós com degree >= 2 (God Nodes + hubs) + filtro de grau mínimo na UI.

## T03 — Backend: eliminar N+1 e O(N²) nas resoluções do SQLite
**Arquivo:** `src-tauri/src/engine/db.rs`
- `resolve_pending_code_relations`: substituir SELECT por linha por um join/batch único.
- `resolve_call_graph`: reescrever com resolução em lote (join direto em `code_symbols`/`code_chunks` indexados), evitando ~1,5M subqueries correlacionadas em 50k arquivos.

## T04 — Backend: parsing paralelo com rayon
**Arquivos:** `src-tauri/Cargo.toml`, `src-tauri/src/engine/vault.rs`
- Adicionar `rayon` e usar `par_iter` no parse de notas e arquivos de código durante o scan.
- Paralelizar `resolve_import_path` por arquivo (já dentro do mesmo bloco de parse).

## T05 — Backend: Two-Stage Sync (estrutura primeiro, embeddings depois)
**Arquivo:** `src-tauri/src/engine/vault.rs`, `src/services/api.ts`, `src/App.tsx` (listener)
- Estágio 1: scan+parse+AST+imports+links salvos no SQLite → emitir `vault-structure-ready` (gráfico e FTS5 disponíveis em segundos).
- Estágio 2: embeddings em background cooperativo (`yield_now`), com flag de status para o frontend.
- Para projetos > 5k arquivos: embed apenas 1 chunk-resumo por arquivo por padrão (env `GRAPHFINDER_FULL_EMBED=1` para modo completo).

## T06 — Verificação
- `cargo test` e `cargo check` no `src-tauri`.
- `npm run build` e `npm run lint`.
- Benchmark manual: projeto ~2k arquivos e validação de FPS na UI.

---

## Status
- [ ] T01
- [ ] T02
- [ ] T03
- [ ] T04
- [ ] T05
- [ ] T06
