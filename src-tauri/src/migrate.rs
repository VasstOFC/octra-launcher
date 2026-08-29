//! Jednorazowa migracja danych z `.octralauncher-dev` i opcjonalnie `.lumenlauncher`.

use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

const MIGRATION_FLAG: &str = ".migrated-from-dev";

pub fn ensure_data_dir(base: &Path, target_name: &str) -> Result<PathBuf> {
    let target = base.join(target_name);
    if target.exists() {
        return Ok(target);
    }

    let dev = base.join(".octralauncher-dev");
    let lumen = base.join(".lumenlauncher");

    if dev.exists() && !dev.join(MIGRATION_FLAG).exists() {
        migrate_dir(&dev, &target)?;
        let _ = std::fs::write(dev.join(MIGRATION_FLAG), chrono::Utc::now().to_rfc3339());
        return Ok(target);
    }

    if lumen.exists() && !target.exists() {
        copy_tree(&lumen, &target)?;
        let _ = std::fs::write(target.join(MIGRATION_FLAG), "from-lumenlauncher");
    }

    std::fs::create_dir_all(&target)?;
    Ok(target)
}

fn migrate_dir(from: &Path, to: &Path) -> Result<()> {
    if to.exists() {
        return Err(Error::msg("Folder docelowy już istnieje — migracja pominięta."));
    }
    std::fs::rename(from, to).map_err(|e| Error::msg(format!("Migracja danych: {e}")))?;
    Ok(())
}

fn copy_tree(from: &Path, to: &Path) -> Result<()> {
    std::fs::create_dir_all(to)?;
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let src = entry.path();
        let dst = to.join(entry.file_name());
        if src.is_dir() {
            copy_tree(&src, &dst)?;
        } else {
            std::fs::copy(&src, &dst)?;
        }
    }
    Ok(())
}
