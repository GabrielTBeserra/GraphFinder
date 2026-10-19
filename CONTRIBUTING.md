# Guia de Contribuicao para o GraphFinder

Obrigado pelo seu interesse em contribuir com o GraphFinder. Este e um projeto 100% open source focado em soberania de dados, arquitetura local-first e economia cirurgica de tokens para agentes de inteligencia artificial.

Este documento estabelece as diretrizes para enviar melhorias, correcoes de bugs e novas ferramentas.

---

## Codigo de Conduta

Ao participar deste projeto, voce concorda em seguir o nosso [Codigo de Conduta](CODE_OF_CONDUCT.md). Tratamos todas as pessoas com respeito, empatia e profissionalismo.

---

## Como Posso Contribuir?

1. **Relatar Bugs**: Abra uma issue detalhando o comportamento inesperado, passos para reproduzir e seu ambiente (sistema operacional, versao do Node e Rust).
2. **Sugerir Funcionalidades**: Abra uma issue descrevendo a proposta de melhoria e como ela se alinha aos principios de minimalismo e eficiencia de contexto.
3. **Enviar Codigo**: Corrija uma issue existente ou implemente uma melhoria alinhada com os objetivos do projeto.
4. **Melhorar a Documentacao**: Ajuste instrucoes, guias de setup e exemplos de integracao com ferramentas MCP.

---

## Fluxo de Desenvolvimento

### 1. Pre-requisitos
* Node.js 18 ou superior
* Rust e Cargo 1.75 ou superior
* Git

### 2. Fork e Clonagem
```bash
git clone https://github.com/SEU_USUARIO/GraphFinder.git
cd GraphFinder
git remote add upstream https://github.com/ORIGINAL_USUARIO/GraphFinder.git
```

### 3. Instalacao de Dependencias
```bash
npm install
```

### 4. Criacao de Branch
Crie uma branch descritiva a partir da `main`:
* Para correcoes: `fix/descricao-curta`
* Para melhorias: `feat/descricao-curta`
* Para documentacao: `docs/descricao-curta`

Exemplo:
```bash
git checkout -b feat/mcp-filter-by-tag
```

### 5. Executando o Ambiente Local
* Executar aplicativo desktop com Tauri e hot reload:
  ```bash
  npm run app
  ```
* Executar apenas a interface web (Vite):
  ```bash
  npm run dev
  ```
* Executar o servidor MCP diretamente:
  ```bash
  npm run mcp
  ```

---

## Padroes de Codigo

### Frontend (React 19, TypeScript, CSS)
* **Sem Emojis**: O projeto adota uma estetica estritamente minimalista, tecnica e sobria. Nao utilize emojis na interface, comentarios de codigo ou documentacao.
* **Componentes Funcionais**: Utilize React funcional com hooks e tipagem rigorosa em TypeScript.
* **Design System**: Respeite os tokens CSS definidos em `src/index.css`. Nao utilize frameworks utilitarios externos que concorram com os estilos existentes.
* **Verificacao de Tipos e Linter**:
  ```bash
  npm run build
  npm run lint
  ```

### Backend (Rust, Tauri v2, SQLite, FastEmbed)
* **Codigo Seguro e Idiomatico**: Escreva codigo Rust formatado com `cargo fmt` e sem avisos de `cargo clippy`.
* **Concorrencia e I/O**: Utilize Tokio para operacoes assincronas e garanta que transacoes no SQLite usem o modo WAL.
* **Fatiamento Atomico**: Qualquer mudanca no motor vetorial deve preservar a quebra estrita de notas por cabecalhos `##` e `###`.

---

## Convencao de Commits

Adotamos a convencao de [Conventional Commits](https://www.conventionalcommits.org/):

* `feat`: Nova funcionalidade
* `fix`: Correcao de bug
* `docs`: Alteracoes em documentacao
* `style`: Formatacao e estilo de codigo (sem alteracao de logica)
* `refactor`: Refatoracao de codigo interno
* `test`: Adicao ou correcao de testes
* `chore`: Atualizacoes de build, dependencias ou ferramentas

Exemplo:
```bash
git commit -m "feat(mcp): adiciona filtro de notas por tag na busca semantica"
```

---

## Enviando um Pull Request

1. Garanta que a sua branch esta atualizada com a `upstream/main`:
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```
2. Execute a validacao completa:
   ```bash
   npm run build
   npm run lint
   cargo check --manifest-path src-tauri/Cargo.toml
   ```
3. Envie sua branch para seu fork:
   ```bash
   git push origin feat/sua-feature
   ```
4. Abra o Pull Request no GitHub utilizando o template padrao fornecido.

---

## Licenca de Contribuicao

Ao contribuir com o GraphFinder, voce concorda que suas contribuicoes serao disponibilizadas sob a licenca [MIT](LICENSE) do projeto.
