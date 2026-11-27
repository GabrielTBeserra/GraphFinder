import { Callout, CodeBlock, DataTable, DocSection } from './docs';

type Language = 'en' | 'pt';

export function serverToc(language: Language) {
  const pt = language === 'pt';
  return [
    { id: 'server-overview', label: pt ? 'Visão geral' : 'Overview' },
    { id: 'server-install', label: pt ? 'Instalação' : 'Installation' },
    { id: 'server-first-user', label: pt ? 'Primeiro usuário' : 'First user' },
    { id: 'server-config', label: pt ? 'Configuração' : 'Configuration' },
    { id: 'server-projects', label: pt ? 'Projetos e fontes' : 'Projects & sources' },
    { id: 'server-providers', label: pt ? 'Providers Git' : 'Git providers' },
    { id: 'server-automation', label: pt ? 'Indexação automática' : 'Automatic indexing' },
    { id: 'server-tokens', label: pt ? 'Tokens e MCP' : 'Tokens & MCP' },
    { id: 'server-operations', label: pt ? 'Operação' : 'Operations' },
    { id: 'server-backup', label: pt ? 'Backup e segurança' : 'Backup & security' },
    { id: 'server-troubleshooting', label: pt ? 'Problemas e FAQ' : 'Troubleshooting & FAQ' },
  ];
}

export default function ServerManual({ language }: { language: Language }) {
  const pt = language === 'pt';
  const tx = (en: string, ptText: string) => (pt ? ptText : en);
  const codeProps = { copyLabel: pt ? 'Copiar' : 'Copy', copiedLabel: pt ? 'Copiado' : 'Copied' };

  return (
    <>
      <DocSection id="server-overview" index="01" title={tx('Private server overview', 'Visão geral do servidor privado')}>
        <p className="lead-p">{tx('GraphFinder Server is the multi-user, self-hosted shape of GraphFinder. It centralises repositories, indexing workers, embeddings and token-authenticated MCP access for a team.', 'O GraphFinder Server é a forma multiusuário e self-hosted do GraphFinder. Ele centraliza repositórios, workers de indexação, embeddings e acesso MCP autenticado por token para uma equipe.')}</p>
        <p>{tx('Use it when several people or agents should query the same indexed projects. Use the desktop client when one person wants a local-first vault and embedded MCP server.', 'Use-o quando várias pessoas ou agentes precisam consultar os mesmos projetos indexados. Use o cliente desktop quando uma pessoa quer um vault local-first e MCP embarcado.')}</p>
        <CodeBlock title={tx('deployment flow', 'fluxo de deployment')} language="text" code={`centralized projects
  └─ Git sources / managed vault
      └─ queue + indexing workers
          └─ immutable generations
              └─ search + graph + MCP authorized per token`} {...codeProps} />
        <DataTable
          headers={[tx('Service', 'Serviço'), tx('Responsibility', 'Responsabilidade')]}
          rows={[
            [<strong>PostgreSQL + pgvector</strong>, tx('Users, sessions, projects, sources, jobs, chunks, embeddings, links and generations.', 'Usuários, sessões, projetos, fontes, jobs, chunks, embeddings, links e gerações.')],
            [<strong>api</strong>, tx('REST API, search, token validation, webhooks and remote MCP at /mcp.', 'API REST, busca, validação de tokens, webhooks e MCP remoto em /mcp.')],
            [<strong>worker</strong>, tx('Clones repositories, parses files, computes embeddings and publishes generations.', 'Clona repositórios, analisa arquivos, calcula embeddings e publica gerações.')],
            [<strong>admin</strong>, tx('Web panel for overview, projects, sources, providers, jobs, search, users and tokens.', 'Painel web para overview, projetos, fontes, providers, jobs, busca, usuários e tokens.')],
            [<strong>proxy</strong>, tx('Caddy edge proxy routing API/MCP and the admin panel.', 'Proxy Caddy roteando API/MCP e o painel admin.')],
          ]}
        />
        <Callout type="info">{tx('PostgreSQL is the authoritative state. Model cache and repository cache are disposable performance artifacts.', 'PostgreSQL é o estado autoritativo. Cache de modelo e de repositórios são artefatos descartáveis de performance.')}</Callout>
      </DocSection>

      <DocSection id="server-install" index="02" title={tx('Installation', 'Instalação')}>
        <h3>{tx('Requirements', 'Requisitos')}</h3>
        <ul>
          <li>{tx('Linux host or VM with Docker Engine 24+ and Docker Compose plugin.', 'Host ou VM Linux com Docker Engine 24+ e plugin Docker Compose.')}</li>
          <li>{tx('At least 2 vCPU, 4 GB RAM and disk for PostgreSQL, repository clones and the embedding model.', 'Pelo menos 2 vCPU, 4 GB RAM e disco para PostgreSQL, clones e o modelo de embeddings.')}</li>
          <li>{tx('Outbound internet access on first run for container images, Git repositories and the ONNX model.', 'Acesso de saída à internet na primeira execução para imagens, repositórios Git e o modelo ONNX.')}</li>
          <li>{tx('For production, DNS plus HTTPS through Caddy or an existing reverse proxy.', 'Para produção, DNS mais HTTPS pelo Caddy ou por um proxy reverso existente.')}</li>
        </ul>
        <h3>{tx('Install and start', 'Instalar e iniciar')}</h3>
        <CodeBlock title="bash" language="bash" code={`git clone https://github.com/GabrielTBeserra/GraphFinder.git
cd GraphFinder/server/deploy
cp .env.example .env

# Generate strong secrets
openssl rand -hex 32  # POSTGRES_PASSWORD
openssl rand -hex 32  # GF_CREDENTIALS_ENCRYPTION_KEY

# Edit .env, then build and start
docker compose build
docker compose up -d
docker compose ps
docker compose logs -f api worker`} {...codeProps} />
        <p>{tx('Fill at minimum POSTGRES_PASSWORD and GF_CREDENTIALS_ENCRYPTION_KEY. The migration service runs automatically before api and worker. The first worker startup can take longer because the embedding model is downloaded once into model-cache.', 'Preencha no mínimo POSTGRES_PASSWORD e GF_CREDENTIALS_ENCRYPTION_KEY. O serviço de migration roda automaticamente antes de api e worker. A primeira subida do worker pode demorar porque o modelo é baixado uma vez no model-cache.')}</p>
        <DataTable
          headers={[tx('Variable', 'Variável'), tx('Default', 'Padrão'), tx('Purpose', 'Finalidade')]}
          rows={[
            [<code>GF_HTTP_PORT</code>, <code>8080</code>, tx('HTTP port exposed by Caddy.', 'Porta HTTP exposta pelo Caddy.')],
            [<code>GF_HTTPS_PORT</code>, <code>8443</code>, tx('HTTPS port when using a domain.', 'Porta HTTPS ao usar domínio.')],
            [<code>SITE_ADDRESS</code>, <code>:80</code>, tx('Caddy site address; use your DNS name in production.', 'Endereço do site no Caddy; use seu DNS em produção.')],
          ]}
        />
      </DocSection>

      <DocSection id="server-first-user" index="03" title={tx('First user and first login', 'Primeiro usuário e primeiro login')}>
        <p>{tx('A clean database has no users. Create the first tenant owner with the server CLI. The command is idempotent and can also rotate that user password.', 'Um banco limpo não tem usuários. Crie o primeiro owner do tenant com a CLI do servidor. O comando é idempotente e também pode rotacionar a senha desse usuário.')}</p>
        <CodeBlock title="bash" language="bash" code={`cd GraphFinder/server/deploy

docker compose run --rm migrate

docker compose run --rm \
  -e GF_BOOTSTRAP_ADMIN_PASSWORD='replace-with-a-long-random-password' \
  api bootstrap-admin \
  --tenant-slug default \
  --email admin@example.com \
  --display-name 'Administrator'`} {...codeProps} />
        <ol>
          <li>{tx('Open the admin panel, for example http://server-ip:8080.', 'Abra o painel admin, por exemplo http://ip-do-servidor:8080.')}</li>
          <li>{tx('Sign in with the email and password passed to bootstrap-admin.', 'Entre com o email e senha passados ao bootstrap-admin.')}</li>
          <li>{tx('Do not share the administrator account. Create named users in Access → Users.', 'Não compartilhe a conta de administrador. Crie usuários nomeados em Access → Users.')}</li>
          <li>{tx('If the bootstrap password was shared or stored insecurely, rotate it immediately.', 'Se a senha bootstrap foi compartilhada ou armazenada inseguramente, rotacione imediatamente.')}</li>
        </ol>
        <Callout type="warn">{tx('If the PostgreSQL volume is wiped, old browser sessions become invalid. Log out, press Ctrl+F5 and sign in with the new bootstrap admin.', 'Se o volume PostgreSQL for apagado, sessões antigas do navegador ficam inválidas. Faça logout, pressione Ctrl+F5 e entre com o novo admin bootstrap.')}</Callout>
        <h3>{tx('Optional SSO/OIDC', 'SSO/OIDC opcional')}</h3>
        <p>{tx('Human users can authenticate through OIDC using GF_OIDC_ISSUER, GF_OIDC_CLIENT_ID, GF_OIDC_CLIENT_SECRET and GF_OIDC_REDIRECT_URI. The public callback is /api/v1/auth/oidc/callback. MCP agents should use personal access tokens.', 'Usuários humanos podem autenticar por OIDC usando GF_OIDC_ISSUER, GF_OIDC_CLIENT_ID, GF_OIDC_CLIENT_SECRET e GF_OIDC_REDIRECT_URI. O callback público é /api/v1/auth/oidc/callback. Agentes MCP devem usar tokens de acesso pessoal.')}</p>
      </DocSection>

      <DocSection id="server-config" index="04" title={tx('Configuration reference', 'Referência de configuração')}>
        <p>{tx('Configuration is environment-variable based. Keep real values in server/deploy/.env or a secret manager; never commit them.', 'A configuração usa variáveis de ambiente. Guarde valores reais em server/deploy/.env ou em um secret manager; nunca commite.')}</p>
        <DataTable
          headers={[tx('Variable', 'Variável'), tx('Required?', 'Obrigatória?'), tx('Meaning', 'Significado')]}
          rows={[
            [<code>POSTGRES_PASSWORD</code>, tx('Yes', 'Sim'), tx('Password for the graphfinder PostgreSQL user.', 'Senha do usuário PostgreSQL graphfinder.')],
            [<code>GF_CREDENTIALS_ENCRYPTION_KEY</code>, tx('Yes for private Git', 'Sim para Git privado'), tx('32-byte key used to encrypt provider tokens. Back it up.', 'Chave de 32 bytes para criptografar tokens de providers. Faça backup.')],
            [<code>GF_DEFAULT_TENANT_SLUG</code>, tx('No', 'Não'), tx('Tenant for local password accounts; default is default.', 'Tenant para contas locais de senha; padrão default.')],
            [<code>GF_ALLOW_PRIVATE_GIT_HOSTS</code>, tx('No', 'Não'), tx('Allows private-network Git hosts; use only on trusted networks.', 'Permite hosts Git em rede privada; use apenas em redes confiáveis.')],
            [<code>GF_ALLOWED_GIT_HOSTS</code>, tx('No', 'Não'), tx('Comma-separated exact host allowlist.', 'Lista de hosts exatos separada por vírgula.')],
            [<code>GF_GIT_TIMEOUT_SECS</code>, tx('No', 'Não'), tx('Maximum clone/fetch time; default 120 seconds.', 'Tempo máximo de clone/fetch; padrão 120 segundos.')],
            [<code>GF_MAX_REPO_BYTES</code>, tx('No', 'Não'), tx('Repository size guardrail; default 1 GB.', 'Limite de tamanho de repositório; padrão 1 GB.')],
            [<code>GF_WORKER_POLL_MS</code>, tx('No', 'Não'), tx('Worker queue polling interval; default 2000 ms.', 'Intervalo de polling da fila; padrão 2000 ms.')],
          ]}
        />
        <CodeBlock title=".env" language="bash" code={`GF_ALLOW_PRIVATE_GIT_HOSTS=false
GF_ALLOWED_GIT_HOSTS=git.internal.example.com,dev.azure.com`} {...codeProps} />
        <p>{tx('Prefer exact hosts. A broad private-host allowance can make indexing infrastructure reach internal services unintentionally.', 'Prefira hosts exatos. Uma liberação ampla pode fazer a infraestrutura de indexação alcançar serviços internos sem intenção.')}</p>
      </DocSection>

      <DocSection id="server-projects" index="05" title={tx('Projects, sources and first indexing', 'Projetos, fontes e primeira indexação')}>
        <ol>
          <li><strong>{tx('Open Projects', 'Abra Projects')}</strong> — {tx('create a descriptive project such as product-api or team-handbook. Projects isolate files, chunks, embeddings and graph links.', 'crie um projeto descritivo como product-api ou team-handbook. Projetos isolam arquivos, chunks, embeddings e links do grafo.')}</li>
          <li><strong>{tx('Open Sources', 'Abra Sources')}</strong> — {tx('select the project and add an HTTPS Git URL plus branch, usually main.', 'selecione o projeto e adicione uma URL HTTPS Git com a branch, normalmente main.')}</li>
          <li><strong>{tx('Choose indexing mode', 'Escolha o modo de indexação')}</strong> — {tx('manual, periodic interval, or after-push webhook. For first validation, manual is simplest.', 'manual, intervalo periódico ou webhook após push. Para primeira validação, manual é mais simples.')}</li>
          <li><strong>{tx('Trigger indexing', 'Dispare a indexação')}</strong> — {tx('use Index jobs → Trigger indexing and watch queued → running → succeeded.', 'use Index jobs → Trigger indexing e acompanhe queued → running → succeeded.')}</li>
          <li><strong>{tx('Validate retrieval', 'Valide a recuperação')}</strong> — {tx('open Search lab and test semantic, lexical and hybrid queries before distributing tokens.', 'abra Search lab e teste consultas semânticas, lexicais e híbridas antes de distribuir tokens.')}</li>
        </ol>
        <h3>{tx('Generations', 'Gerações')}</h3>
        <p>{tx('Every indexing job builds a new immutable generation while the previous generation continues serving searches. Publication is atomic after success, avoiding half-indexed projects and enabling rollback.', 'Cada job constrói uma nova geração imutável enquanto a anterior continua atendendo buscas. A publicação é atômica após sucesso, evitando projetos parcialmente indexados e permitindo rollback.')}</p>
        <Callout type="tip">{tx('The first indexing job can be slow because of the one-time embedding model download. Later jobs reuse the shared model-cache volume.', 'O primeiro job pode ser lento por causa do download único do modelo. Jobs posteriores reutilizam o volume model-cache.')}</Callout>
      </DocSection>

      <DocSection id="server-providers" index="06" title={tx('Private Git providers', 'Providers Git privados')}>
        <p>{tx('Providers centralise credentials for private repositories. Supported types are GitHub, GitLab, Azure DevOps and generic/self-hosted HTTPS Git servers.', 'Providers centralizam credenciais para repositórios privados. Os tipos suportados são GitHub, GitLab, Azure DevOps e servidores Git HTTPS genéricos/self-hosted.')}</p>
        <DataTable
          headers={[tx('Platform', 'Plataforma'), tx('Recommended credential', 'Credencial recomendada'), tx('Minimum access', 'Acesso mínimo')]}
          rows={[
            ['GitHub', tx('Fine-grained PAT or machine-user PAT', 'PAT fine-grained ou de usuário máquina'), tx('Contents: read on selected repositories', 'Contents: read nos repositórios selecionados')],
            ['GitLab', tx('Project access token or personal access token', 'Project access token ou personal access token'), <code>read_repository</code>],
            ['Azure DevOps', tx('Personal access token', 'Personal access token'), tx('Code → Read', 'Code → Read')],
            [tx('Generic/self-hosted', 'Genérico/self-hosted'), tx('HTTPS username + token/password', 'HTTPS usuário + token/senha'), tx('Read-only clone/fetch', 'Clone/fetch somente leitura')],
          ]}
        />
        <ol>
          <li>{tx('Open Providers → New provider.', 'Abra Providers → New provider.')}</li>
          <li>{tx('Choose a clear name, select the type and enter the exact host: github.com, gitlab.com, dev.azure.com or your self-hosted hostname.', 'Escolha um nome claro, selecione o tipo e informe o host exato: github.com, gitlab.com, dev.azure.com ou seu hostname self-hosted.')}</li>
          <li>{tx('Enter username and token. For Azure DevOps any non-empty username normally works because the PAT is the secret.', 'Informe usuário e token. No Azure DevOps qualquer usuário não vazio normalmente funciona porque o PAT é o segredo.')}</li>
          <li>{tx('When adding a source, select this provider. The API validates exact host matching.', 'Ao adicionar uma fonte, selecione esse provider. A API valida correspondência exata de host.')}</li>
        </ol>
        <Callout type="info">{tx('Tokens are encrypted at rest with AES-256-GCM using GF_CREDENTIALS_ENCRYPTION_KEY. The API never returns plaintext tokens; it only exposes a suffix hint such as …7890.', 'Tokens são criptografados em repouso com AES-256-GCM usando GF_CREDENTIALS_ENCRYPTION_KEY. A API nunca retorna tokens em texto claro; expõe apenas um hint de sufixo como …7890.')}</Callout>
      </DocSection>

      <DocSection id="server-automation" index="07" title={tx('Automatic indexing', 'Indexação automática')}>
        <DataTable
          headers={[tx('Mode', 'Modo'), tx('When to use', 'Quando usar'), tx('Behavior', 'Comportamento')]}
          rows={[
            [tx('Manual', 'Manual'), tx('Controlled reindexing and validation', 'Reindexação controlada e validação'), tx('Only runs when triggered.', 'Só executa quando disparado.')],
            [tx('Interval', 'Intervalo'), tx('Simple automation without webhooks', 'Automação simples sem webhooks'), tx('Every 5 minutes to 24 hours.', 'A cada 5 minutos até 24 horas.')],
            [tx('After push · webhook', 'Após push · webhook'), tx('Fresh index after pushes', 'Índice fresco após pushes'), tx('Validated webhook enqueues a job and deduplicates active duplicates.', 'Webhook validado enfileira job e deduplica duplicados ativos.')],
          ]}
        />
        <h3>{tx('Configure after-push', 'Configurar após push')}</h3>
        <ol>
          <li>{tx('In Sources, edit the source and choose after push · webhook.', 'Em Sources, edite a fonte e escolha after push · webhook.')}</li>
          <li>{tx('Generate webhook configuration and copy the URL/secret immediately; the secret is shown once.', 'Gere a configuração de webhook e copie URL/secret imediatamente; o secret aparece uma vez.')}</li>
          <li>{tx('Add the webhook in the Git platform for push events only.', 'Adicione o webhook na plataforma Git apenas para eventos de push.')}</li>
          <li>{tx('Push to the tracked branch and check Index jobs.', 'Faça push para a branch rastreada e verifique Index jobs.')}</li>
        </ol>
        <DataTable
          headers={[tx('Platform', 'Plataforma'), tx('Validation', 'Validação'), tx('Configuration location', 'Local de configuração')]}
          rows={[
            ['GitHub', <code>X-Hub-Signature-256</code>, tx('Repository → Settings → Webhooks', 'Repositório → Settings → Webhooks')],
            ['GitLab', <code>X-Gitlab-Token</code>, tx('Project → Settings → Webhooks', 'Projeto → Settings → Webhooks')],
            ['Azure DevOps', tx('Basic auth service-hook credentials', 'Credenciais Basic auth do service hook'), tx('Project Settings → Service hooks', 'Project Settings → Service hooks')],
            [tx('Generic HMAC', 'HMAC genérico'), <code>X-Graphfinder-Signature</code>, tx('Custom integration', 'Integração customizada')],
          ]}
        />
        <p>{tx('GraphFinder validates signature/token, repository URL, branch and deletion marker. Wrong signatures return 401; other branches are ignored as branch_mismatch.', 'O GraphFinder valida assinatura/token, URL do repositório, branch e marcador de deleção. Assinaturas erradas retornam 401; outras branches são ignoradas como branch_mismatch.')}</p>
      </DocSection>

      <DocSection id="server-tokens" index="08" title={tx('API tokens and remote MCP', 'Tokens de API e MCP remoto')}>
        <p>{tx('Create one token per person or agent in Access → API tokens. Never share one token across the team; revocation and project restrictions become impossible.', 'Crie um token por pessoa ou agente em Access → API tokens. Nunca compartilhe um token com a equipe; revogação e restrição por projeto ficam impossíveis.')}</p>
        <DataTable
          headers={[tx('Scope', 'Scope'), tx('Allows', 'Permite')]}
          rows={[
            [<code>search:read</code>, tx('Semantic, lexical and hybrid search plus compressed context.', 'Busca semântica, lexical e híbrida, além de contexto comprimido.')],
            [<code>code:read</code>, tx('Files, file context, code chunks, dependencies, paths and graph explanations.', 'Arquivos, contexto, chunks, dependências, caminhos e explicações de grafo.')],
            [<code>index:trigger</code>, tx('Trigger reindexing and query indexing status.', 'Disparar reindexação e consultar status.')],
            [<code>vault:write / vault:delete</code>, tx('Write or delete managed vault notes.', 'Escrever ou excluir notas do vault gerenciado.')],
          ]}
        />
        <CodeBlock title="mcp_config.json" language="json" code={`{
  "mcpServers": {
    "graphfinder": {
      "url": "http://YOUR-SERVER:8080/mcp",
      "headers": {
        "Authorization": "Bearer YOUR_TOKEN"
      }
    }
  }
}`} {...codeProps} />
        <p>{tx('Use the public HTTPS origin in production. A read-only token normally uses search:read + code:read. Add index:trigger only to operational agents allowed to reindex.', 'Use a origem HTTPS pública em produção. Um token somente leitura normalmente usa search:read + code:read. Adicione index:trigger apenas a agentes operacionais autorizados a reindexar.')}</p>
        <CodeBlock title={tx('recommended agent pipeline', 'pipeline recomendado para agentes')} language="text" code={`list_projects
  → search_semantic or search_code
  → get_file_context
  → read_code_chunk
  → get_code_dependencies / find_path for impact analysis`} {...codeProps} />
      </DocSection>

      <DocSection id="server-operations" index="09" title={tx('Daily operations and observability', 'Operação diária e observabilidade')}>
        <CodeBlock title="bash" language="bash" code={`curl -fsS http://SERVER:8080/health
curl -fsS http://SERVER:8080/ready
docker compose ps
docker compose logs --tail=100 api
docker compose logs --tail=100 worker`} {...codeProps} />
        <ul>
          <li><code>/health</code> — {tx('process liveness.', 'processo vivo.')}</li>
          <li><code>/ready</code> — {tx('database/readiness validation for the API.', 'validação de prontidão/banco para a API.')}</li>
          <li>{tx('The Overview panel shows projects, source coverage, queued/running/failed jobs, 7-day activity, success rate and generation state.', 'O painel Overview mostra projetos, cobertura de fontes, jobs em fila/rodando/falhos, atividade de 7 dias, taxa de sucesso e estado das gerações.')}</li>
          <li>{tx('A failed job does not replace the active generation; search keeps serving the last published generation.', 'Um job falho não substitui a geração ativa; a busca continua servindo a última geração publicada.')}</li>
          <li>{tx('Scale workers only after validating database capacity and memory use. Workers share the PostgreSQL queue.', 'Escale workers apenas após validar capacidade do banco e memória. Workers compartilham a fila PostgreSQL.')}</li>
        </ul>
      </DocSection>

      <DocSection id="server-backup" index="10" title={tx('Backup, restore and security', 'Backup, restauração e segurança')}>
        <CodeBlock title="backup" language="bash" code={`cd GraphFinder/server/deploy
export POSTGRES_PASSWORD='your-postgres-password'
./backup.sh`} {...codeProps} />
        <CodeBlock title="restore" language="bash" code={`cd GraphFinder/server/deploy
export POSTGRES_PASSWORD='your-postgres-password'
GF_CONFIRM_RESTORE=yes ./restore.sh backups/graphfinder-YYYYMMDDTHHMMSSZ.dump
docker compose up -d
curl -fsS http://SERVER:8080/ready`} {...codeProps} />
        <Callout type="warn">{tx('Restore is destructive to the target database. Test restores regularly in an isolated compose project before trusting the backup policy.', 'A restauração é destrutiva para o banco de destino. Teste restaurações regularmente em um compose isolado antes de confiar na política de backup.')}</Callout>
        <ul>
          <li>{tx('Expose admin/API only through private network, VPN/Tailscale or HTTPS reverse proxy.', 'Exponha admin/API apenas por rede privada, VPN/Tailscale ou proxy HTTPS.')}</li>
          <li>{tx('Keep POSTGRES_PASSWORD, encryption key, bootstrap passwords, webhook secrets and PATs in a secret manager.', 'Mantenha POSTGRES_PASSWORD, chave de criptografia, senhas bootstrap, secrets de webhook e PATs em gerenciador de segredos.')}</li>
          <li>{tx('Create least-privilege tokens; read-only agents normally need only search:read and code:read.', 'Crie tokens com menor privilégio; agentes somente leitura normalmente precisam apenas de search:read e code:read.')}</li>
          <li>{tx('Back up PostgreSQL, not just repository/model caches. Rotate credentials when staff leave or a token leaks.', 'Faça backup do PostgreSQL, não apenas dos caches. Rotacione credenciais quando alguém sair ou um token vazar.')}</li>
        </ul>
      </DocSection>

      <DocSection id="server-troubleshooting" index="11" title={tx('Troubleshooting and FAQ', 'Problemas e FAQ')}>
        <h3>{tx('Why does the browser show errors after a clean reinstall?', 'Por que o navegador mostra erros após reinstalação limpa?')}</h3>
        <p>{tx('The old session belongs to a database that no longer exists. Log out, hard-refresh with Ctrl+F5, clear site storage if necessary and sign in with the new bootstrap admin.', 'A sessão antiga pertence a um banco que não existe mais. Faça logout, force refresh com Ctrl+F5, limpe o armazenamento do site se necessário e entre com o novo admin bootstrap.')}</p>
        <h3><code>insufficient project scope</code></h3>
        <p>{tx('This is an authorization error for that MCP tool, not a broken MCP connection. Add the missing scope or avoid write/trigger tools with a read-only token.', 'Isso é erro de autorização da ferramenta MCP, não conexão quebrada. Adicione o scope faltante ou evite ferramentas de escrita/trigger com token somente leitura.')}</p>
        <h3>{tx('Clone or fetch fails for a private repository', 'Clone ou fetch falha em repositório privado')}</h3>
        <ul>
          <li>{tx('Confirm exact host matching and token permissions.', 'Confirme correspondência exata de host e permissões do token.')}</li>
          <li>{tx('For internal hosts, configure GF_ALLOWED_GIT_HOSTS or intentionally enable GF_ALLOW_PRIVATE_GIT_HOSTS.', 'Para hosts internos, configure GF_ALLOWED_GIT_HOSTS ou habilite intencionalmente GF_ALLOW_PRIVATE_GIT_HOSTS.')}</li>
          <li>{tx('Check worker logs; credentials are redacted but host and Git failure category remain visible.', 'Verifique logs do worker; credenciais são redigidas, mas host e categoria de falha Git ficam visíveis.')}</li>
        </ul>
        <h3>{tx('Webhook returns 401 or does not enqueue', 'Webhook retorna 401 ou não enfileira')}</h3>
        <p>{tx('401 means signature/token validation failed. A 200 with ignored/branch_mismatch means another branch was pushed. If a job is already active, duplicates are intentionally deduplicated.', '401 significa falha de validação de assinatura/token. Um 200 com ignored/branch_mismatch significa push em outra branch. Se já existe job ativo, duplicados são intencionalmente deduplicados.')}</p>
        <h3>{tx('Migration checksum or database startup problems', 'Problemas de checksum de migration ou inicialização do banco')}</h3>
        <p>{tx('Do not edit applied migration files. SQL files are normalized to LF because sqlx checksums are byte-sensitive; restore canonical files from Git and rebuild the image.', 'Não edite migrations aplicadas. Arquivos SQL são normalizados para LF porque checksums do sqlx são sensíveis a bytes; restaure arquivos canônicos do Git e reconstrua a imagem.')}</p>
      </DocSection>
    </>
  );
}
