# Diretrizes de Curadoria de Conhecimento e Grafo Semântico

Esta regra aplica-se a todas as operações de escrita, leitura e consulta no cofre de conhecimento do **GraphFinder**.

## 1. Estrutura de Notas (Markdown)
- **H1 Único**: `# Título` na primeira linha.
- **Subseções Atômicas**: `## Seção` e `### Subseção` estruturando conceitos em blocos concisos (50–150 tokens). Isso viabiliza deep linking direto (`[[Nota#Seção]]`).
- **Resumo no Topo**: O primeiro bloco de texto deve ser conciso e de alta densidade informacional.
- **Wikilinks**: Sempre referenciar outras notas do cofre usando `[[Nome da Nota]]` ou `[[Nome da Nota#Seção]]`.
- **Tags**: Incluir hashtags contextuais `#tag` para alimentação dos clusters de cores do grafo.

## 2. Consumo Eficiente via MCP (Economia de Tokens)
- Sempre priorizar `search_semantic` (busca por trechos atômicos).
- Usar `get_compressed_context` para inspecionar o sumário de seções sem consumir o corpo integral.
- Utilizar `suggest_links` ao redigir novos parágrafos para estabelecer conexões semânticas com outros nós do cofre.
- Evitar duplicação de conceitos; conectar novas ideias a notas existentes.
