use crate::storage::{Database, SnapshotRecord};
use anyhow::{bail, Context, Result};
use chrono::Utc;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use tar::{Archive, Builder};
use uuid::Uuid;

pub struct SnapshotManager {
    snapshots_dir: PathBuf,
    db: Database,
}

impl SnapshotManager {
    pub fn new<P: AsRef<Path>>(sandbox_base_dir: P, db: Database) -> Self {
        let snapshots_dir = sandbox_base_dir.as_ref().join("snapshots");
        let _ = fs::create_dir_all(&snapshots_dir);
        Self {
            snapshots_dir,
            db,
        }
    }

    /// Creates a lightweight tarball snapshot of a project directory.
    pub fn create_snapshot(
        &self,
        project_id: &str,
        project_dir: &Path,
        title: &str,
        description: Option<&str>,
        git_commit: Option<&str>,
    ) -> Result<SnapshotRecord> {
        let snapshot_id = format!("snap_{}", Uuid::new_v4());
        let tar_gz_path = self.snapshots_dir.join(format!("{}.tar.gz", snapshot_id));

        let tar_gz_file = File::create(&tar_gz_path)
            .with_context(|| format!("Failed to create snapshot file {:?}", tar_gz_path))?;
        let enc = GzEncoder::new(tar_gz_file, Compression::default());
        let mut tar_builder = Builder::new(enc);

        Self::append_to_tar(&mut tar_builder, project_dir, Path::new(""))?;
        tar_builder.finish()?;

        let record = SnapshotRecord {
            id: snapshot_id,
            project_id: project_id.to_string(),
            title: title.to_string(),
            description: description.map(String::from),
            snapshot_path: tar_gz_path.to_string_lossy().to_string(),
            git_commit: git_commit.map(String::from),
            created_at: Utc::now().to_rfc3339(),
        };

        self.db.save_snapshot(&record)?;
        Ok(record)
    }

    fn append_to_tar<W: std::io::Write>(
        tar: &mut Builder<W>,
        real_path: &Path,
        rel_prefix: &Path,
    ) -> Result<()> {
        if !real_path.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(real_path)?.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            // Ignore transient / huge dirs
            if name == ".git"
                || name == "node_modules"
                || name == "target"
                || name == ".venv"
                || name == "venv"
                || name == "__pycache__"
            {
                continue;
            }

            let entry_rel = rel_prefix.join(&name);
            if path.is_dir() {
                Self::append_to_tar(tar, &path, &entry_rel)?;
            } else if path.is_file() {
                if let Ok(mut f) = File::open(&path) {
                    let _ = tar.append_file(&entry_rel, &mut f);
                }
            }
        }

        Ok(())
    }

    /// Restores a snapshot archive back into the target project directory.
    pub fn restore_snapshot(&self, snapshot_id: &str, target_project_dir: &Path) -> Result<()> {
        let snapshots = self.db.list_snapshots("")?;
        let snap = snapshots.into_iter().find(|s| s.id == snapshot_id)
            .or_else(|| {
                // Fallback: check file directly
                let p = self.snapshots_dir.join(format!("{}.tar.gz", snapshot_id));
                if p.exists() {
                    Some(SnapshotRecord {
                        id: snapshot_id.to_string(),
                        project_id: "".to_string(),
                        title: "".to_string(),
                        description: None,
                        snapshot_path: p.to_string_lossy().to_string(),
                        git_commit: None,
                        created_at: "".to_string(),
                    })
                } else {
                    None
                }
            })
            .context("Snapshot not found in database or disk")?;

        let tar_gz_path = PathBuf::from(&snap.snapshot_path);
        if !tar_gz_path.exists() {
            bail!("Snapshot archive file missing at {:?}", tar_gz_path);
        }

        let tar_gz_file = File::open(&tar_gz_path)?;
        let tar = GzDecoder::new(tar_gz_file);
        let mut archive = Archive::new(tar);

        fs::create_dir_all(target_project_dir)?;
        archive.unpack(target_project_dir)?;

        Ok(())
    }

    /// Deletes a snapshot file and removes its database entry.
    pub fn delete_snapshot(&self, snapshot_id: &str) -> Result<()> {
        let tar_gz_path = self.snapshots_dir.join(format!("{}.tar.gz", snapshot_id));
        if tar_gz_path.exists() {
            let _ = fs::remove_file(&tar_gz_path);
        }
        self.db.delete_snapshot(snapshot_id)?;
        Ok(())
    }
}
