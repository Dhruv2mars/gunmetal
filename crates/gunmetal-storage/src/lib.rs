use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use chrono::{DateTime, Utc};
use gunmetal_core::{
    CreatedGunmetalKey, GunmetalKey, KeyScope, KeyState, ModelDescriptor, NewGunmetalKey,
    NewProviderProfile, NewRequestLogEntry, ProviderKind, ProviderProfile, RequestLogEntry,
    TokenUsage,
};
use rusqlite::{Connection, OptionalExtension, params};
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppPaths {
    pub root: PathBuf,
    pub database: PathBuf,
    pub empty_workspace_dir: PathBuf,
    pub helpers_dir: PathBuf,
    pub logs_dir: PathBuf,
}

impl AppPaths {
    pub fn resolve() -> Result<Self> {
        if let Ok(path) = std::env::var("GUNMETAL_HOME") {
            return Self::from_root(PathBuf::from(path));
        }

        let Some(home) = dirs::home_dir() else {
            bail!("could not resolve user home directory");
        };

        Self::from_root(home.join(".gunmetal"))
    }

    pub fn from_root(root: PathBuf) -> Result<Self> {
        let paths = Self {
            database: root.join("state").join("gunmetal.db"),
            empty_workspace_dir: root.join("empty-workspace"),
            helpers_dir: root.join("helpers"),
            logs_dir: root.join("logs"),
            root,
        };
        paths.ensure()?;
        Ok(paths)
    }

    pub fn ensure(&self) -> Result<()> {
        std::fs::create_dir_all(&self.root)
            .with_context(|| format!("failed to create {}", self.root.display()))?;
        std::fs::create_dir_all(self.database.parent().expect("database parent exists"))
            .with_context(|| format!("failed to create {}", self.database.display()))?;
        std::fs::create_dir_all(&self.helpers_dir)
            .with_context(|| format!("failed to create {}", self.helpers_dir.display()))?;
        std::fs::create_dir_all(&self.logs_dir)
            .with_context(|| format!("failed to create {}", self.logs_dir.display()))?;
        std::fs::create_dir_all(&self.empty_workspace_dir)
            .with_context(|| format!("failed to create {}", self.empty_workspace_dir.display()))?;
        Ok(())
    }

    pub fn storage_handle(&self) -> Result<StorageHandle> {
        StorageHandle::new(self.database.clone())
    }
}

#[derive(Debug, Clone)]
pub struct StorageHandle {
    path: PathBuf,
}

impl StorageHandle {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self> {
        let handle = Self { path: path.into() };
        handle.storage()?;
        Ok(handle)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn storage(&self) -> Result<Storage> {
        Storage::open(&self.path)
    }

    pub fn create_key(&self, draft: NewGunmetalKey) -> Result<CreatedGunmetalKey> {
        self.storage()?.create_key(draft)
    }

    pub fn list_keys(&self) -> Result<Vec<GunmetalKey>> {
        self.storage()?.list_keys()
    }

    pub fn get_key(&self, id: Uuid) -> Result<Option<GunmetalKey>> {
        self.storage()?.get_key(id)
    }

    pub fn authenticate_key(&self, secret: &str) -> Result<Option<GunmetalKey>> {
        self.storage()?.authenticate_key(secret)
    }

    pub fn set_key_state(&self, id: Uuid, state: KeyState) -> Result<()> {
        self.storage()?.set_key_state(id, state)
    }

    pub fn delete_key(&self, id: Uuid) -> Result<()> {
        self.storage()?.delete_key(id)
    }

    pub fn create_profile(&self, draft: NewProviderProfile) -> Result<ProviderProfile> {
        self.storage()?.create_profile(draft)
    }

    pub fn list_profiles(&self) -> Result<Vec<ProviderProfile>> {
        self.storage()?.list_profiles()
    }

    pub fn get_profile(&self, id: Uuid) -> Result<Option<ProviderProfile>> {
        self.storage()?.get_profile(id)
    }

    pub fn replace_models_for_profile(
        &self,
        provider: &ProviderKind,
        profile_id: Option<Uuid>,
        models: &[ModelDescriptor],
    ) -> Result<()> {
        self.storage()?
            .replace_models_for_profile(provider, profile_id, models)
    }

    pub fn list_models(&self) -> Result<Vec<ModelDescriptor>> {
        self.storage()?.list_models()
    }

    pub fn log_request(&self, entry: NewRequestLogEntry) -> Result<RequestLogEntry> {
        self.storage()?.log_request(entry)
    }

    pub fn list_request_logs(&self, limit: usize) -> Result<Vec<RequestLogEntry>> {
        self.storage()?.list_request_logs(limit)
    }
}

pub struct Storage {
    conn: Connection,
}

impl Storage {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }

        let conn =
            Connection::open(path).with_context(|| format!("failed to open {}", path.display()))?;
        Self::from_connection(conn)
    }

    pub fn open_in_memory() -> Result<Self> {
        Self::from_connection(Connection::open_in_memory()?)
    }

    fn from_connection(conn: Connection) -> Result<Self> {
        let storage = Self { conn };
        storage.migrate()?;
        Ok(storage)
    }

    pub fn create_key(&self, draft: NewGunmetalKey) -> Result<CreatedGunmetalKey> {
        if draft.name.trim().is_empty() {
            bail!("key name cannot be empty");
        }

        if draft.scopes.is_empty() {
            bail!("at least one scope is required");
        }

        let id = Uuid::new_v4();
        let now = Utc::now();
        let secret = format!("gm_{}_{}", id.simple(), Uuid::new_v4().simple());
        let prefix = format!("gm_{}", &id.simple().to_string()[..8]);
        let secret_hash = hash_secret(&secret);

        self.conn.execute(
            "insert into keys (
                id, name, prefix, secret_hash, state, expires_at, created_at, updated_at, last_used_at
            ) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                id.to_string(),
                draft.name,
                prefix,
                secret_hash,
                KeyState::Active.to_string(),
                draft.expires_at.map(to_rfc3339),
                to_rfc3339(now),
                to_rfc3339(now),
                Option::<String>::None,
            ],
        )?;

        self.replace_key_scopes(id, &draft.scopes)?;
        self.replace_key_providers(id, &draft.allowed_providers)?;

        let record = self
            .get_key(id)?
            .ok_or_else(|| anyhow!("created key was not persisted"))?;

        Ok(CreatedGunmetalKey { record, secret })
    }

    pub fn list_keys(&self) -> Result<Vec<GunmetalKey>> {
        let mut stmt = self.conn.prepare(
            "select id, name, prefix, state, expires_at, created_at, updated_at, last_used_at
             from keys
             order by created_at desc",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((
                parse_uuid(row.get::<_, String>(0)?)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                parse_key_state(row.get::<_, String>(3)?)?,
                parse_optional_datetime(row.get::<_, Option<String>>(4)?)?,
                parse_datetime(row.get::<_, String>(5)?)?,
                parse_datetime(row.get::<_, String>(6)?)?,
                parse_optional_datetime(row.get::<_, Option<String>>(7)?)?,
            ))
        })?;

        rows.map(|row| {
            let (id, name, prefix, state, expires_at, created_at, updated_at, last_used_at) = row?;
            Ok(GunmetalKey {
                id,
                name,
                prefix,
                state,
                scopes: self.list_key_scopes(id)?,
                allowed_providers: self.list_key_providers(id)?,
                expires_at,
                created_at,
                updated_at,
                last_used_at,
            })
        })
        .collect()
    }

    pub fn get_key(&self, id: Uuid) -> Result<Option<GunmetalKey>> {
        let mut stmt = self.conn.prepare(
            "select id, name, prefix, state, expires_at, created_at, updated_at, last_used_at
             from keys
             where id = ?1",
        )?;

        let maybe = stmt
            .query_row([id.to_string()], |row| {
                Ok(GunmetalKey {
                    id: parse_uuid(row.get::<_, String>(0)?)?,
                    name: row.get(1)?,
                    prefix: row.get(2)?,
                    state: parse_key_state(row.get::<_, String>(3)?)?,
                    scopes: Vec::new(),
                    allowed_providers: Vec::new(),
                    expires_at: parse_optional_datetime(row.get::<_, Option<String>>(4)?)?,
                    created_at: parse_datetime(row.get::<_, String>(5)?)?,
                    updated_at: parse_datetime(row.get::<_, String>(6)?)?,
                    last_used_at: parse_optional_datetime(row.get::<_, Option<String>>(7)?)?,
                })
            })
            .optional()?;

        maybe
            .map(|mut key| {
                key.scopes = self.list_key_scopes(key.id)?;
                key.allowed_providers = self.list_key_providers(key.id)?;
                Ok(key)
            })
            .transpose()
    }

    pub fn authenticate_key(&self, secret: &str) -> Result<Option<GunmetalKey>> {
        let hash = hash_secret(secret);
        let mut stmt = self
            .conn
            .prepare("select id from keys where secret_hash = ?1 limit 1")?;
        let maybe_id = stmt
            .query_row([hash], |row| row.get::<_, String>(0))
            .optional()?;

        let Some(id) = maybe_id else {
            return Ok(None);
        };

        let key_id = parse_uuid(id)?;
        let now = Utc::now();
        let Some(key) = self.get_key(key_id)? else {
            return Ok(None);
        };

        if !key.is_usable_at(now) {
            return Ok(None);
        }

        self.conn.execute(
            "update keys set last_used_at = ?2, updated_at = ?2 where id = ?1",
            params![key.id.to_string(), to_rfc3339(now)],
        )?;

        self.get_key(key.id)
    }

    pub fn set_key_state(&self, id: Uuid, state: KeyState) -> Result<()> {
        let changed = self.conn.execute(
            "update keys set state = ?2, updated_at = ?3 where id = ?1",
            params![id.to_string(), state.to_string(), to_rfc3339(Utc::now())],
        )?;

        if changed == 0 {
            bail!("key not found");
        }

        Ok(())
    }

    pub fn delete_key(&self, id: Uuid) -> Result<()> {
        self.conn
            .execute("delete from key_scopes where key_id = ?1", [id.to_string()])?;
        self.conn.execute(
            "delete from key_allowed_providers where key_id = ?1",
            [id.to_string()],
        )?;
        let changed = self
            .conn
            .execute("delete from keys where id = ?1", [id.to_string()])?;

        if changed == 0 {
            bail!("key not found");
        }

        Ok(())
    }

    pub fn create_profile(&self, draft: NewProviderProfile) -> Result<ProviderProfile> {
        if draft.name.trim().is_empty() {
            bail!("profile name cannot be empty");
        }

        let now = Utc::now();
        let id = Uuid::new_v4();
        self.conn.execute(
            "insert into provider_profiles (
                id, provider, name, base_url, enabled, credentials_json, created_at, updated_at
            ) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                id.to_string(),
                draft.provider.to_string(),
                draft.name,
                draft.base_url,
                if draft.enabled { 1 } else { 0 },
                draft.credentials.map(|value| value.to_string()),
                to_rfc3339(now),
                to_rfc3339(now),
            ],
        )?;

        self.get_profile(id)?
            .ok_or_else(|| anyhow!("created profile was not persisted"))
    }

    pub fn list_profiles(&self) -> Result<Vec<ProviderProfile>> {
        let mut stmt = self.conn.prepare(
            "select id, provider, name, base_url, enabled, credentials_json, created_at, updated_at
             from provider_profiles
             order by created_at desc",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(ProviderProfile {
                id: parse_uuid(row.get::<_, String>(0)?)?,
                provider: parse_provider(row.get::<_, String>(1)?)?,
                name: row.get(2)?,
                base_url: row.get(3)?,
                enabled: row.get::<_, i64>(4)? == 1,
                credentials: parse_optional_json(row.get::<_, Option<String>>(5)?)?,
                created_at: parse_datetime(row.get::<_, String>(6)?)?,
                updated_at: parse_datetime(row.get::<_, String>(7)?)?,
            })
        })?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into)
    }

    pub fn get_profile(&self, id: Uuid) -> Result<Option<ProviderProfile>> {
        let mut stmt = self.conn.prepare(
            "select id, provider, name, base_url, enabled, credentials_json, created_at, updated_at
             from provider_profiles
             where id = ?1",
        )?;

        stmt.query_row([id.to_string()], |row| {
            Ok(ProviderProfile {
                id: parse_uuid(row.get::<_, String>(0)?)?,
                provider: parse_provider(row.get::<_, String>(1)?)?,
                name: row.get(2)?,
                base_url: row.get(3)?,
                enabled: row.get::<_, i64>(4)? == 1,
                credentials: parse_optional_json(row.get::<_, Option<String>>(5)?)?,
                created_at: parse_datetime(row.get::<_, String>(6)?)?,
                updated_at: parse_datetime(row.get::<_, String>(7)?)?,
            })
        })
        .optional()
        .map_err(Into::into)
    }

    pub fn replace_models_for_profile(
        &self,
        provider: &ProviderKind,
        profile_id: Option<Uuid>,
        models: &[ModelDescriptor],
    ) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        match profile_id {
            Some(profile_id) => {
                tx.execute(
                    "delete from models where provider = ?1 and profile_id = ?2",
                    params![provider.to_string(), profile_id.to_string()],
                )?;
            }
            None => {
                tx.execute(
                    "delete from models where provider = ?1 and profile_id is null",
                    params![provider.to_string()],
                )?;
            }
        }

        for model in models {
            tx.execute(
                "insert into models (id, provider, profile_id, upstream_name, display_name)
                 values (?1, ?2, ?3, ?4, ?5)",
                params![
                    model.id,
                    model.provider.to_string(),
                    model.profile_id.map(|value| value.to_string()),
                    model.upstream_name,
                    model.display_name,
                ],
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    pub fn list_models(&self) -> Result<Vec<ModelDescriptor>> {
        let mut stmt = self.conn.prepare(
            "select id, provider, profile_id, upstream_name, display_name
             from models
             order by provider asc, id asc",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(ModelDescriptor {
                id: row.get(0)?,
                provider: parse_provider(row.get::<_, String>(1)?)?,
                profile_id: row
                    .get::<_, Option<String>>(2)?
                    .map(parse_uuid)
                    .transpose()?,
                upstream_name: row.get(3)?,
                display_name: row.get(4)?,
            })
        })?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into)
    }

    pub fn log_request(&self, entry: NewRequestLogEntry) -> Result<RequestLogEntry> {
        let id = Uuid::new_v4();
        let started_at = Utc::now();

        self.conn.execute(
            "insert into request_logs (
                id, started_at, key_id, profile_id, provider, model, endpoint, status_code,
                duration_ms, input_tokens, output_tokens, total_tokens, error_message
            ) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                id.to_string(),
                to_rfc3339(started_at),
                entry.key_id.map(|value| value.to_string()),
                entry.profile_id.map(|value| value.to_string()),
                entry.provider.to_string(),
                entry.model,
                entry.endpoint,
                entry.status_code.map(i64::from),
                to_i64(entry.duration_ms)?,
                entry.usage.input_tokens.map(i64::from),
                entry.usage.output_tokens.map(i64::from),
                entry.usage.total_tokens.map(i64::from),
                entry.error_message,
            ],
        )?;

        self.list_request_logs(1)?
            .into_iter()
            .next()
            .ok_or_else(|| anyhow!("request log was not persisted"))
    }

    pub fn list_request_logs(&self, limit: usize) -> Result<Vec<RequestLogEntry>> {
        let mut stmt = self.conn.prepare(
            "select id, started_at, key_id, profile_id, provider, model, endpoint, status_code,
                    duration_ms, input_tokens, output_tokens, total_tokens, error_message
             from request_logs
             order by started_at desc
             limit ?1",
        )?;

        let rows = stmt.query_map([to_i64(limit as u64)?], |row| {
            Ok(RequestLogEntry {
                id: parse_uuid(row.get::<_, String>(0)?)?,
                started_at: parse_datetime(row.get::<_, String>(1)?)?,
                key_id: row
                    .get::<_, Option<String>>(2)?
                    .map(parse_uuid)
                    .transpose()?,
                profile_id: row
                    .get::<_, Option<String>>(3)?
                    .map(parse_uuid)
                    .transpose()?,
                provider: parse_provider(row.get::<_, String>(4)?)?,
                model: row.get(5)?,
                endpoint: row.get(6)?,
                status_code: row
                    .get::<_, Option<i64>>(7)?
                    .map(u16::try_from)
                    .transpose()
                    .map_err(to_from_sql_err)?,
                duration_ms: row.get::<_, i64>(8)?.try_into().map_err(to_from_sql_err)?,
                usage: TokenUsage {
                    input_tokens: row
                        .get::<_, Option<i64>>(9)?
                        .map(u32::try_from)
                        .transpose()
                        .map_err(to_from_sql_err)?,
                    output_tokens: row
                        .get::<_, Option<i64>>(10)?
                        .map(u32::try_from)
                        .transpose()
                        .map_err(to_from_sql_err)?,
                    total_tokens: row
                        .get::<_, Option<i64>>(11)?
                        .map(u32::try_from)
                        .transpose()
                        .map_err(to_from_sql_err)?,
                },
                error_message: row.get(12)?,
            })
        })?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into)
    }

    fn migrate(&self) -> Result<()> {
        self.conn.execute_batch(
            "
            pragma journal_mode = wal;
            pragma foreign_keys = on;

            create table if not exists keys (
                id text primary key,
                name text not null,
                prefix text not null unique,
                secret_hash text not null unique,
                state text not null,
                expires_at text null,
                created_at text not null,
                updated_at text not null,
                last_used_at text null
            );

            create table if not exists key_scopes (
                key_id text not null,
                scope text not null,
                primary key (key_id, scope),
                foreign key (key_id) references keys(id) on delete cascade
            );

            create table if not exists key_allowed_providers (
                key_id text not null,
                provider text not null,
                primary key (key_id, provider),
                foreign key (key_id) references keys(id) on delete cascade
            );

            create table if not exists provider_profiles (
                id text primary key,
                provider text not null,
                name text not null,
                base_url text null,
                enabled integer not null,
                credentials_json text null,
                created_at text not null,
                updated_at text not null
            );

            create table if not exists models (
                id text primary key,
                provider text not null,
                profile_id text null,
                upstream_name text not null,
                display_name text not null,
                foreign key (profile_id) references provider_profiles(id) on delete set null
            );

            create table if not exists request_logs (
                id text primary key,
                started_at text not null,
                key_id text null,
                profile_id text null,
                provider text not null,
                model text not null,
                endpoint text not null,
                status_code integer null,
                duration_ms integer not null,
                input_tokens integer null,
                output_tokens integer null,
                total_tokens integer null,
                error_message text null,
                foreign key (key_id) references keys(id) on delete set null,
                foreign key (profile_id) references provider_profiles(id) on delete set null
            );
            ",
        )?;

        Ok(())
    }

    fn replace_key_scopes(&self, key_id: Uuid, scopes: &[KeyScope]) -> Result<()> {
        self.conn.execute(
            "delete from key_scopes where key_id = ?1",
            [key_id.to_string()],
        )?;

        for scope in scopes {
            self.conn.execute(
                "insert into key_scopes (key_id, scope) values (?1, ?2)",
                params![key_id.to_string(), scope.to_string()],
            )?;
        }

        Ok(())
    }

    fn replace_key_providers(&self, key_id: Uuid, providers: &[ProviderKind]) -> Result<()> {
        self.conn.execute(
            "delete from key_allowed_providers where key_id = ?1",
            [key_id.to_string()],
        )?;

        for provider in providers {
            self.conn.execute(
                "insert into key_allowed_providers (key_id, provider) values (?1, ?2)",
                params![key_id.to_string(), provider.to_string()],
            )?;
        }

        Ok(())
    }

    fn list_key_scopes(&self, key_id: Uuid) -> Result<Vec<KeyScope>> {
        let mut stmt = self
            .conn
            .prepare("select scope from key_scopes where key_id = ?1 order by scope asc")?;
        let rows = stmt.query_map([key_id.to_string()], |row| row.get::<_, String>(0))?;

        rows.map(|row| parse_scope(row?))
            .collect::<Result<Vec<_>, _>>()
    }

    fn list_key_providers(&self, key_id: Uuid) -> Result<Vec<ProviderKind>> {
        let mut stmt = self.conn.prepare(
            "select provider from key_allowed_providers where key_id = ?1 order by provider asc",
        )?;
        let rows = stmt.query_map([key_id.to_string()], |row| row.get::<_, String>(0))?;

        let raw = rows.collect::<rusqlite::Result<Vec<_>>>()?;
        raw.into_iter()
            .map(parse_provider_anyhow)
            .collect::<Result<Vec<_>, _>>()
    }
}

fn hash_secret(secret: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn to_rfc3339(value: DateTime<Utc>) -> String {
    value.to_rfc3339()
}

fn parse_datetime(value: String) -> rusqlite::Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(&value)
        .map(|value| value.with_timezone(&Utc))
        .map_err(to_from_sql_err)
}

fn parse_optional_datetime(value: Option<String>) -> rusqlite::Result<Option<DateTime<Utc>>> {
    value.map(parse_datetime).transpose()
}

fn parse_optional_json(value: Option<String>) -> rusqlite::Result<Option<serde_json::Value>> {
    value
        .map(|item| serde_json::from_str(&item).map_err(to_from_sql_err))
        .transpose()
}

fn parse_uuid(value: String) -> rusqlite::Result<Uuid> {
    Uuid::parse_str(&value).map_err(to_from_sql_err)
}

fn parse_provider(value: String) -> rusqlite::Result<ProviderKind> {
    value.parse::<ProviderKind>().map_err(to_from_sql_message)
}

fn parse_scope(value: String) -> Result<KeyScope> {
    value.parse::<KeyScope>().map_err(|error| anyhow!(error))
}

fn parse_key_state(value: String) -> rusqlite::Result<KeyState> {
    value.parse::<KeyState>().map_err(to_from_sql_message)
}

fn parse_provider_anyhow(value: String) -> Result<ProviderKind> {
    value
        .parse::<ProviderKind>()
        .map_err(|error| anyhow!(error))
}

fn to_i64(value: u64) -> Result<i64> {
    i64::try_from(value).context("value exceeds sqlite integer range")
}

fn to_from_sql_err<E>(error: E) -> rusqlite::Error
where
    E: std::error::Error + Send + Sync + 'static,
{
    rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(error))
}

fn to_from_sql_message(error: String) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(
        0,
        rusqlite::types::Type::Text,
        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, error)),
    )
}

#[cfg(test)]
mod tests {
    use chrono::Duration;
    use gunmetal_core::{KeyScope, KeyState, NewProviderProfile, NewRequestLogEntry, ProviderKind};
    use serde_json::json;
    use tempfile::TempDir;

    use super::{AppPaths, Storage, StorageHandle};

    #[test]
    fn creates_authenticates_and_revokes_keys() {
        let storage = Storage::open_in_memory().unwrap();

        let created = storage
            .create_key(gunmetal_core::NewGunmetalKey {
                name: "default".to_owned(),
                scopes: vec![KeyScope::Inference, KeyScope::ModelsRead],
                allowed_providers: vec![ProviderKind::Codex, ProviderKind::Copilot],
                expires_at: Some(chrono::Utc::now() + Duration::days(1)),
            })
            .unwrap();

        assert!(created.secret.starts_with("gm_"));
        assert_eq!(created.record.name, "default");
        assert_eq!(created.record.allowed_providers.len(), 2);

        let authenticated = storage.authenticate_key(&created.secret).unwrap().unwrap();
        assert_eq!(authenticated.id, created.record.id);
        assert!(authenticated.last_used_at.is_some());

        storage
            .set_key_state(created.record.id, KeyState::Disabled)
            .unwrap();
        assert!(storage.authenticate_key(&created.secret).unwrap().is_none());

        storage
            .set_key_state(created.record.id, KeyState::Revoked)
            .unwrap();
        let revoked = storage.get_key(created.record.id).unwrap().unwrap();
        assert_eq!(revoked.state, KeyState::Revoked);
    }

    #[test]
    fn deletes_keys_cleanly() {
        let storage = Storage::open_in_memory().unwrap();
        let created = storage
            .create_key(gunmetal_core::NewGunmetalKey {
                name: "throwaway".to_owned(),
                scopes: vec![KeyScope::Inference],
                allowed_providers: vec![],
                expires_at: None,
            })
            .unwrap();

        storage.delete_key(created.record.id).unwrap();
        assert!(storage.get_key(created.record.id).unwrap().is_none());
    }

    #[test]
    fn creates_profiles_and_model_registry() {
        let storage = Storage::open_in_memory().unwrap();
        let profile = storage
            .create_profile(NewProviderProfile {
                provider: ProviderKind::OpenRouter,
                name: "team".to_owned(),
                base_url: Some("https://openrouter.ai/api/v1".to_owned()),
                enabled: true,
                credentials: Some(json!({ "api_key": "secret" })),
            })
            .unwrap();

        let profiles = storage.list_profiles().unwrap();
        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].id, profile.id);

        storage
            .replace_models_for_profile(
                &ProviderKind::OpenRouter,
                Some(profile.id),
                &[gunmetal_core::ModelDescriptor {
                    id: "openrouter/openai/gpt-5.1".to_owned(),
                    provider: ProviderKind::OpenRouter,
                    profile_id: Some(profile.id),
                    upstream_name: "openai/gpt-5.1".to_owned(),
                    display_name: "GPT-5.1".to_owned(),
                }],
            )
            .unwrap();

        let models = storage.list_models().unwrap();
        assert_eq!(models.len(), 1);
        assert_eq!(models[0].id, "openrouter/openai/gpt-5.1");
    }

    #[test]
    fn writes_lightweight_request_logs() {
        let storage = Storage::open_in_memory().unwrap();
        let log = storage
            .log_request(NewRequestLogEntry {
                key_id: None,
                profile_id: None,
                provider: ProviderKind::Codex,
                model: "codex/gpt-5.4".to_owned(),
                endpoint: "/v1/chat/completions".to_owned(),
                status_code: Some(200),
                duration_ms: 182,
                usage: gunmetal_core::TokenUsage {
                    input_tokens: Some(42),
                    output_tokens: Some(12),
                    total_tokens: Some(54),
                },
                error_message: None,
            })
            .unwrap();

        let logs = storage.list_request_logs(10).unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].id, log.id);
        assert_eq!(logs[0].usage.total_tokens, Some(54));
    }

    #[test]
    fn storage_handle_reopens_file_backed_state() {
        let temp = TempDir::new().unwrap();
        let handle = StorageHandle::new(temp.path().join("gunmetal.db")).unwrap();

        let created = handle
            .create_key(gunmetal_core::NewGunmetalKey {
                name: "default".to_owned(),
                scopes: vec![KeyScope::Inference],
                allowed_providers: vec![ProviderKind::Codex],
                expires_at: None,
            })
            .unwrap();

        let reopened = StorageHandle::new(handle.path().to_path_buf()).unwrap();
        let authenticated = reopened.authenticate_key(&created.secret).unwrap().unwrap();
        assert_eq!(authenticated.id, created.record.id);
    }

    #[test]
    fn app_paths_create_expected_layout() {
        let temp = TempDir::new().unwrap();
        let paths = AppPaths::from_root(temp.path().join("gunmetal-home")).unwrap();

        assert!(paths.root.exists());
        assert!(paths.empty_workspace_dir.exists());
        assert!(paths.helpers_dir.exists());
        assert!(paths.logs_dir.exists());
        assert_eq!(paths.database.file_name().unwrap(), "gunmetal.db");
    }

    #[test]
    fn replacing_models_only_touches_one_profile_slice() {
        let storage = Storage::open_in_memory().unwrap();
        let codex = storage
            .create_profile(NewProviderProfile {
                provider: ProviderKind::Codex,
                name: "codex".to_owned(),
                base_url: None,
                enabled: true,
                credentials: None,
            })
            .unwrap();
        let openrouter = storage
            .create_profile(NewProviderProfile {
                provider: ProviderKind::OpenRouter,
                name: "openrouter".to_owned(),
                base_url: None,
                enabled: true,
                credentials: None,
            })
            .unwrap();

        storage
            .replace_models_for_profile(
                &ProviderKind::Codex,
                Some(codex.id),
                &[gunmetal_core::ModelDescriptor {
                    id: "codex/gpt-5.4".to_owned(),
                    provider: ProviderKind::Codex,
                    profile_id: Some(codex.id),
                    upstream_name: "gpt-5.4".to_owned(),
                    display_name: "GPT-5.4".to_owned(),
                }],
            )
            .unwrap();
        storage
            .replace_models_for_profile(
                &ProviderKind::OpenRouter,
                Some(openrouter.id),
                &[gunmetal_core::ModelDescriptor {
                    id: "openrouter/openai/gpt-5.1".to_owned(),
                    provider: ProviderKind::OpenRouter,
                    profile_id: Some(openrouter.id),
                    upstream_name: "openai/gpt-5.1".to_owned(),
                    display_name: "GPT-5.1".to_owned(),
                }],
            )
            .unwrap();
        storage
            .replace_models_for_profile(
                &ProviderKind::Codex,
                Some(codex.id),
                &[gunmetal_core::ModelDescriptor {
                    id: "codex/gpt-5.5".to_owned(),
                    provider: ProviderKind::Codex,
                    profile_id: Some(codex.id),
                    upstream_name: "gpt-5.5".to_owned(),
                    display_name: "GPT-5.5".to_owned(),
                }],
            )
            .unwrap();

        let models = storage.list_models().unwrap();
        assert_eq!(models.len(), 2);
        assert!(models.iter().any(|model| model.id == "codex/gpt-5.5"));
        assert!(
            models
                .iter()
                .any(|model| model.id == "openrouter/openai/gpt-5.1")
        );
    }
}
