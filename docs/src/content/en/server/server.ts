export default `
<section id="server">
          <h2><span class="idx">01</span>Private server<a class="anchor" href="#server" aria-label="Link to Private server">#</a></h2>
          <p class="lead-p">In addition to the local-first desktop app, GraphFinder now has a <strong>private server</strong> mode for centralising team projects. It keeps the same surgical retrieval model for agents, but uses PostgreSQL/pgvector, indexing queues and per-user access tokens.</p>
          <p>The operating model changes from “every person indexes their own folder” to:</p>
          <div class="code"><div class="code-bar">flow</div>
<pre data-lang="text">centralised projects
  └─ Git sources / managed vault
      └─ indexing jobs
          └─ atomically published generations
              └─ search + graph + MCP authorised by each user's token</pre></div>

          <h3>Deployment services</h3>
          <div class="table-wrap"><table>
            <thead><tr><th>Service</th><th>Responsibility</th></tr></thead>
            <tbody>
              <tr><td><strong>PostgreSQL + pgvector</strong></td><td>Metadata, users, sessions, tokens, jobs, chunks, embeddings and index generations.</td></tr>
              <tr><td><strong>api</strong></td><td>Administrative REST API, search and the MCP endpoint at <code>/mcp</code>.</td></tr>
              <tr><td><strong>worker</strong></td><td>Clones/updates repositories, parses files, computes embeddings and publishes generations.</td></tr>
              <tr><td><strong>admin</strong></td><td>Web panel for projects, sources, jobs, search, tokens and users.</td></tr>
              <tr><td><strong>proxy</strong></td><td>Caddy edge proxy for the panel and API. Production deployments should terminate TLS here or behind another trusted reverse proxy.</td></tr>
            </tbody>
          </table></div>

          <h3>Daily workflow in the panel</h3>
          <ol>
            <li><strong>Sign in</strong> — login is only email and password. Tenants do not appear in the UX; they remain an internal isolation and audit concept.</li>
            <li><strong>Create a project</strong> — for example <code>my-product</code>.</li>
            <li><strong>Add a Git source</strong> — HTTPS URL and branch (<code>main</code>, <code>master</code> or another tracked branch).</li>
            <li><strong>Trigger indexing</strong> — in <em>Index jobs</em>. Progress refreshes approximately every second.</li>
            <li><strong>Validate in Search lab</strong> — test hybrid, lexical and vector search before distributing tokens.</li>
            <li><strong>Create one token per person</strong> — in <em>Access → API tokens</em>. Prefer read-only tokens by default.</li>
            <li><strong>Configure the MCP client</strong> — each person uses their own Bearer token; visible projects and permissions are derived from it.</li>
          </ol>


          <h3>Private repositories and automatic indexing</h3>
          <p>Private Git access is centralised in <strong>Providers</strong>. A provider credential is a named HTTPS token for GitHub, GitLab, Azure DevOps or a generic/self-hosted Git server. The API stores only an encrypted copy and a non-sensitive suffix hint; raw tokens are never returned.</p>
          <ol>
            <li>Create a provider token in the Git platform with read access to the repository.</li>
            <li>In the GraphFinder panel, open <em>Providers</em> and store the credential with the exact repository host, for example <code>github.com</code>, <code>gitlab.com</code>, <code>dev.azure.com</code> or your self-hosted hostname.</li>
            <li>When adding a source, select that provider. The API validates that provider host and repository host match.</li>
            <li>Choose manual indexing, a periodic interval, or <strong>after push · webhook</strong>.</li>
          </ol>
          <p>For <strong>after push</strong>, the panel generates the endpoint and a one-time webhook secret. Configure that URL/secret in GitHub webhooks, GitLab webhooks, Azure DevOps service hooks, or a generic HMAC integration. GraphFinder validates the delivery, repository URL and branch before enqueueing a job. Deleted branches and branch mismatches are ignored.</p>
          <div class="table-wrap"><table>
            <thead><tr><th>Mode</th><th>Behaviour</th></tr></thead>
            <tbody>
              <tr><td><strong>manual</strong></td><td>Only indexes when someone triggers a job.</td></tr>
              <tr><td><strong>after push</strong></td><td>Webhook enqueues indexing for the tracked branch.</td></tr>
              <tr><td><strong>interval</strong></td><td>Worker schedules indexing every 5 minutes to 24 hours.</td></tr>
            </tbody>
          </table></div>
          <p>Set <code>GF_CREDENTIALS_ENCRYPTION_KEY</code> before storing provider credentials. It must be 32 bytes in raw, hex or base64 form, for example <code>openssl rand -hex 32</code>. Back it up with the deployment secrets; losing it makes stored Git tokens unreadable. Self-hosted/private-network Git hosts remain blocked unless explicitly enabled with <code>GF_ALLOW_PRIVATE_GIT_HOSTS</code> or <code>GF_ALLOWED_GIT_HOSTS</code>.</p>

          <h3>Server MCP configuration</h3>
          <p>The remote endpoint uses Streamable HTTP at <code>/mcp</code> and requires a Bearer token:</p>
          <div class="code"><div class="code-bar">mcp_config.json</div>
<pre data-lang="json">{
  "mcpServers": {
    "graphfinder": {
      "url": "http://YOUR-SERVER:18083/mcp",
      "headers": {
        "Authorization": "Bearer YOUR_TOKEN"
      }
    }
  }
}</pre></div>
          <p>If the server is behind HTTPS, use the HTTPS origin instead of the internal HTTP port. Do not expose this endpoint to the public internet without TLS, a reverse proxy and a token policy.</p>

          <h3>Token scopes</h3>
          <div class="table-wrap"><table>
            <thead><tr><th>Scope</th><th>Allows</th></tr></thead>
            <tbody>
              <tr><td><code>search:read</code></td><td>List authorised projects, search notes/content, query the graph and inspect job status.</td></tr>
              <tr><td><code>code:read</code></td><td>Search code, read symbols, dependencies and file context.</td></tr>
              <tr><td><code>index:trigger</code></td><td>Enqueue reindexing for authorised projects/sources.</td></tr>
              <tr><td><code>vault:write</code></td><td>Create or update managed vault notes.</td></tr>
              <tr><td><code>vault:delete</code></td><td>Delete managed vault notes.</td></tr>
            </tbody>
          </table></div>
          <p>A typical read-only token uses <code>search:read,code:read</code>. An operational agent may also receive <code>index:trigger</code>. A token can be restricted to a specific project; revoking it ends access immediately.</p>

          <h3>MCP tools currently exposed by the server</h3>
          <p>The private server exposes 19 tools, including <code>list_projects</code>, <code>search_semantic</code>, <code>search_code</code>, <code>find_files</code>, <code>get_file_context</code>, <code>read_code_chunk</code>, <code>get_compressed_context</code>, <code>get_code_dependencies</code>, <code>get_graph_neighborhood</code>, <code>find_path</code>, <code>explain_node</code>, <code>trigger_index</code>, <code>get_index_status</code> and managed-vault tools.</p>
          <div class="code"><div class="code-bar">recommended pipeline</div>
<pre data-lang="text">list_projects
  → search_semantic or search_code
  → get_file_context
  → read_code_chunk
  → get_code_dependencies / find_path when analysing impact</pre></div>

          <h3>Generations and rollback</h3>
          <p>Every indexing run creates a new generation. Publication is atomic: the project continues serving the previous active generation while the new one is built. When the job succeeds, the new generation goes live with final progress <code>100%</code>. If a regression happens, an older generation can be restored through the panel/API with explicit confirmation.</p>

          <h3>Server parser languages and limits</h3>
          <p>The server AST-lite parser currently extracts symbols and imports best for <strong>Rust, TypeScript/JavaScript, Python and Go</strong>. Repositories in other languages can still be indexed as text/documents, but their symbol/dependency graph may be incomplete. For modern TypeScript, relative imports such as <code>./module.js</code> are resolved to indexed <code>.ts</code>/<code>.tsx</code> files.</p>

          <h3>Operations and security</h3>
          <ul>
            <li>Use PostgreSQL with persistent volumes and backups; it is the primary server state.</li>
            <li>The first indexing run can be slow because the ONNX embedding model must be downloaded into the cache.</li>
            <li>Keep the panel on a private network, VPN/Tailscale, or behind HTTPS with a reverse proxy.</li>
            <li>Password login has PostgreSQL-backed rate limiting; optional OIDC/SSO can be configured through environment variables.</li>
            <li>Create one token per person/agent, with least-privilege scopes and project restrictions where appropriate.</li>
            <li>Never share tokens in repositories, tickets or public documents.</li>
          </ul>
        </section>
`;
