use sqlx::PgPool;
use uuid::Uuid;

use crate::{Principal, StorageResult};

pub const SCOPE_SEARCH_READ: &str = "search:read";
pub const SCOPE_CODE_READ: &str = "code:read";
pub const SCOPE_PROJECT_WRITE: &str = "project:write";
pub const SCOPE_INDEX_TRIGGER: &str = "index:trigger";
pub const SCOPE_TOKENS_MANAGE: &str = "tokens:manage";

impl Principal {
    pub fn token_project_allowed(&self, project_id: Uuid) -> bool {
        self.project_ids
            .as_ref()
            .map_or(true, |projects| projects.contains(&project_id))
    }

    pub fn has_token_scope(&self, scope: &str) -> bool {
        self.scopes.iter().any(|s| s == "*" || s == scope)
    }
}

pub async fn ensure_tenant_membership(
    pool: &PgPool,
    tenant_id: Uuid,
    user_id: Uuid,
    role: &str,
) -> StorageResult<()> {
    sqlx::query(
        r#"
        INSERT INTO tenant_memberships (tenant_id,user_id,role) VALUES ($1,$2,$3)
        ON CONFLICT (tenant_id,user_id) DO UPDATE SET role=EXCLUDED.role
        "#,
    )
    .bind(tenant_id)
    .bind(user_id)
    .bind(role)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn ensure_project_membership(
    pool: &PgPool,
    project_id: Uuid,
    user_id: Uuid,
    role: &str,
) -> StorageResult<()> {
    sqlx::query(
        r#"
        INSERT INTO project_members (project_id,user_id,role) VALUES ($1,$2,$3)
        ON CONFLICT (project_id,user_id) DO UPDATE SET role=EXCLUDED.role
        "#,
    )
    .bind(project_id)
    .bind(user_id)
    .bind(role)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn tenant_role(
    pool: &PgPool,
    tenant_id: Uuid,
    user_id: Uuid,
) -> StorageResult<Option<String>> {
    Ok(
        sqlx::query_scalar("SELECT role FROM tenant_memberships WHERE tenant_id=$1 AND user_id=$2")
            .bind(tenant_id)
            .bind(user_id)
            .fetch_optional(pool)
            .await?,
    )
}

pub async fn project_role(
    pool: &PgPool,
    project_id: Uuid,
    user_id: Uuid,
) -> StorageResult<Option<String>> {
    Ok(
        sqlx::query_scalar("SELECT role FROM project_members WHERE project_id=$1 AND user_id=$2")
            .bind(project_id)
            .bind(user_id)
            .fetch_optional(pool)
            .await?,
    )
}

fn project_role_allows(role: &str, scope: &str) -> bool {
    match role {
        "maintainer" | "owner" | "admin" => true,
        "editor" => matches!(
            scope,
            SCOPE_SEARCH_READ | SCOPE_CODE_READ | SCOPE_PROJECT_WRITE | SCOPE_INDEX_TRIGGER
        ),
        "viewer" => matches!(scope, SCOPE_SEARCH_READ | SCOPE_CODE_READ),
        _ => false,
    }
}

pub async fn authorize_tenant_scope(
    pool: &PgPool,
    principal: &Principal,
    scope: &str,
) -> StorageResult<bool> {
    if principal.kind == "api_token" {
        return Ok(principal.has_token_scope(scope));
    }
    let Some(user_id) = principal.user_id else {
        return Ok(false);
    };
    Ok(matches!(
        tenant_role(pool, principal.tenant_id, user_id)
            .await?
            .as_deref(),
        Some("owner" | "admin")
    ))
}

pub async fn authorize_project_scope(
    pool: &PgPool,
    principal: &Principal,
    project_id: Uuid,
    scope: &str,
) -> StorageResult<bool> {
    if !principal.token_project_allowed(project_id) {
        return Ok(false);
    }
    let tenant_matches = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM projects WHERE id=$1 AND tenant_id=$2)",
    )
    .bind(project_id)
    .bind(principal.tenant_id)
    .fetch_one(pool)
    .await?;
    if !tenant_matches {
        return Ok(false);
    }

    if principal.kind == "api_token" {
        return Ok(principal.has_token_scope(scope));
    }

    let Some(user_id) = principal.user_id else {
        return Ok(false);
    };
    if matches!(
        tenant_role(pool, principal.tenant_id, user_id)
            .await?
            .as_deref(),
        Some("owner" | "admin")
    ) {
        return Ok(true);
    }
    Ok(matches!(
        project_role(pool, project_id, user_id).await?.as_deref(),
        Some(role) if project_role_allows(role, scope)
    ))
}
