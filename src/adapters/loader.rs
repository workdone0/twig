//! Content-validated, atomically published private SQLite caches.
use crate::core::store::Store;
use anyhow::{bail, Context, Result};
use sha2::{Digest, Sha256};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc,
};

pub trait Loader: Send + Sync {
    fn load(&self, file: &Path, force_rebuild: bool) -> Result<Store>;
}

#[derive(Default)]
pub struct LoadProgress {
    pub bytes: AtomicU64,
    pub total: AtomicU64,
    pub nodes: AtomicU64,
}

#[derive(Clone, Default)]
pub struct LoadOptions {
    pub cache_dir: Option<PathBuf>,
    pub no_cache: bool,
    pub cancelled: Arc<AtomicBool>,
    pub progress: Arc<LoadProgress>,
}

impl LoadOptions {
    pub fn check_cancelled(&self) -> Result<()> {
        if self.cancelled.load(Ordering::Relaxed) {
            bail!("Loading cancelled");
        }
        Ok(())
    }
}

/// Snapshot the input so the digest and parser always see identical bytes.
/// Published databases are immutable and opened read-only; concurrent readers
/// can safely keep an older generation open while a new one is built.
pub fn load_cached(
    file: &Path,
    force: bool,
    options: &LoadOptions,
    parse: impl FnOnce(&mut dyn Read, &mut Store) -> Result<()>,
) -> Result<Store> {
    options.check_cancelled()?;
    let mut source =
        std::fs::File::open(file).with_context(|| format!("opening {}", file.display()))?;
    options
        .progress
        .total
        .store(source.metadata()?.len(), Ordering::Relaxed);
    options.progress.bytes.store(0, Ordering::Relaxed);
    options.progress.nodes.store(0, Ordering::Relaxed);
    let mut snapshot = tempfile::tempfile()?;
    let mut digest = Sha256::new();
    let mut buffer = [0; 64 * 1024];
    loop {
        options.check_cancelled()?;
        let n = source.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        digest.update(&buffer[..n]);
        snapshot.write_all(&buffer[..n])?;
        options
            .progress
            .bytes
            .fetch_add(n as u64, Ordering::Relaxed);
    }
    snapshot.seek(SeekFrom::Start(0))?;
    // Version the cache contract independently from the package version.
    let fingerprint = format!("{:x}", digest.finalize());
    let directory = if options.no_cache {
        None
    } else {
        let dir = match &options.cache_dir {
            Some(dir) => dir.clone(),
            None => crate::core::paths::cache_dir()?,
        };
        private_dir(&dir)?;
        prune_cache(&dir)?;
        Some(dir)
    };
    let target = directory.as_ref().map(|dir| {
        dir.join(format!(
            "v2-{}-{fingerprint}.db",
            crate::core::paths::db_filename_for(file)
        ))
    });
    if !force {
        if let Some(path) = &target {
            if let Some(store) = open_existing(path)? {
                return Ok(store);
            }
        }
    }
    let temp = match &directory {
        Some(dir) => tempfile::Builder::new()
            .prefix(".ingest-")
            .tempdir_in(dir)?,
        None => tempfile::tempdir()?,
    };
    let db = temp.path().join("data.db");
    let mut store = Store::open(&db)?;
    store.db_conn().execute_batch(
        "PRAGMA journal_mode=DELETE; PRAGMA synchronous=OFF; PRAGMA temp_store=FILE;",
    )?;
    drop_indexes(&store)?;
    struct CancellableReader<'a> {
        file: std::fs::File,
        options: &'a LoadOptions,
    }
    impl Read for CancellableReader<'_> {
        fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
            self.options
                .check_cancelled()
                .map_err(std::io::Error::other)?;
            self.file.read(buffer)
        }
    }
    let mut reader = CancellableReader {
        file: snapshot,
        options,
    };
    parse(&mut reader, &mut store)?;
    options.check_cancelled()?;
    rebuild_indexes(&store)?;
    store
        .db_conn()
        .execute_batch("PRAGMA synchronous=FULL; PRAGMA user_version=2;")?;
    drop(store);
    std::fs::OpenOptions::new()
        .write(true)
        .open(&db)?
        .sync_all()?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&db, std::fs::Permissions::from_mode(0o600))?;
    }
    // Hard links publish a complete file without replacing another generation.
    // A competing builder may win; we can still use our private completed DB.
    if let Some(path) = target {
        if std::fs::hard_link(&db, &path).is_ok() {
            return Ok(Store::open_read_only(&path)?);
        }
        if !force {
            if let Some(existing) = open_existing(&path)? {
                return Ok(existing);
            }
        }
    }
    let mut store = Store::open_read_only(&db)?;
    store.temporary = Some(temp);
    Ok(store)
}

pub fn private_dir(dir: &Path) -> Result<()> {
    std::fs::create_dir_all(dir)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}

pub fn open_existing(path: &Path) -> Result<Option<Store>> {
    if !path.is_file() {
        return Ok(None);
    }
    let Ok(store) = Store::open_read_only(path) else {
        return Ok(None);
    };
    let version: i64 = store
        .conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap_or(0);
    if version != 2 || store.node_count().unwrap_or(0) == 0 {
        return Ok(None);
    }
    let valid: String = store
        .conn
        .query_row("PRAGMA quick_check", [], |r| r.get(0))
        .unwrap_or_default();
    if valid != "ok" {
        return Ok(None);
    }
    Ok(Some(store))
}

/// Best-effort retention: completed generations older than seven days, then
/// oldest generations until the existing cache is below 512 MiB. Active readers
/// retain their handles on Unix; Windows can refuse deletion, which is harmless.
pub fn prune_cache(dir: &Path) -> Result<()> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        if !entry.file_name().to_string_lossy().starts_with("v2-")
            || entry.path().extension().is_none_or(|e| e != "db")
        {
            continue;
        }
        let meta = entry.metadata()?;
        if meta.is_file() {
            files.push((meta.modified()?, meta.len(), entry.path()));
        }
    }
    files.sort();
    let mut total: u64 = files.iter().map(|f| f.1).sum();
    for (modified, size, path) in files {
        let expired = modified.elapsed().unwrap_or_default().as_secs() > 7 * 86400;
        if (expired || total > 512 * 1024 * 1024) && std::fs::remove_file(path).is_ok() {
            total = total.saturating_sub(size);
        }
    }
    Ok(())
}

pub fn clear_cache() -> Result<usize> {
    let dir = crate::core::paths::cache_dir()?;
    let mut removed = 0;
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_file()
            && path
                .extension()
                .is_some_and(|e| e == "db" || e == "db-wal" || e == "db-shm")
        {
            std::fs::remove_file(path)?;
            removed += 1;
        }
    }
    Ok(removed)
}

pub fn drop_indexes(store: &Store) -> Result<()> {
    store.conn.execute_batch("DROP INDEX IF EXISTS idx_parent_rank; DROP INDEX IF EXISTS idx_path;
        DROP TRIGGER IF EXISTS nodes_ai; DROP TRIGGER IF EXISTS nodes_ad; DROP TRIGGER IF EXISTS nodes_au;
        DROP TABLE IF EXISTS nodes_search;")?;
    Ok(())
}

pub fn rebuild_indexes(store: &Store) -> Result<()> {
    // Search uses literal substring matching; avoid maintaining an unused FTS
    // index. Traversal/path indexes are rebuilt after bounded batch inserts.
    store.conn.execute_batch(
        "CREATE INDEX IF NOT EXISTS idx_parent_rank ON nodes(parent_id, rank);
        CREATE UNIQUE INDEX IF NOT EXISTS idx_path ON nodes(path);",
    )?;
    Ok(())
}

impl twig_core::parser::ParseControl for LoadOptions {
    fn check_cancelled(&self) -> Result<()> {
        self.check_cancelled()
    }
    fn node_emitted(&self) {
        self.progress.nodes.fetch_add(1, Ordering::Relaxed);
    }
}
