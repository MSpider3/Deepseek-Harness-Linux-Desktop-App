import React, { useState, useEffect } from 'react';
import {
  GitBranch,
  Camera,
  RotateCcw,
  Trash2,
  Clock,
  X,
  FolderOpen
} from 'lucide-react';
import type { GitStatusInfo, SnapshotRecord } from '../types';
import { tauriApi } from '../services/tauriApi';

export const SnapshotsView: React.FC = () => {
  const [projectPath, setProjectPath] = useState('.');
  const [gitStatus, setGitStatus] = useState<GitStatusInfo | null>(null);
  const [snapshots, setSnapshots] = useState<SnapshotRecord[]>([]);

  // Create Snapshot Modal
  const [modalOpen, setModalOpen] = useState(false);
  const [snapTitle, setSnapTitle] = useState('');
  const [snapDesc, setSnapDesc] = useState('');
  const [creating, setCreating] = useState(false);

  const loadData = async () => {
    if (!projectPath.trim()) return;
    try {
      const git = await tauriApi.getGitStatus(projectPath);
      setGitStatus(git);
      const sList = await tauriApi.listSnapshots(projectPath);
      setSnapshots(sList);
    } catch (e) {
      console.error('Failed to load project snapshot data', e);
    }
  };

  useEffect(() => {
    loadData();
  }, [projectPath]);

  const handleCreateSnapshot = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!snapTitle.trim() || !projectPath.trim()) return;
    setCreating(true);
    try {
      await tauriApi.createSnapshot(projectPath, projectPath, snapTitle.trim(), snapDesc.trim() || undefined);
      setModalOpen(false);
      setSnapTitle('');
      setSnapDesc('');
      await loadData();
    } catch (e: any) {
      alert(`Failed to create snapshot: ${e.toString()}`);
    } finally {
      setCreating(false);
    }
  };

  const handleRestore = async (id: string) => {
    if (!confirm('Restore project from this snapshot? All current uncommitted modifications will be replaced.')) return;
    try {
      await tauriApi.restoreSnapshot(id, projectPath);
      alert('Snapshot restored successfully!');
      await loadData();
    } catch (e: any) {
      alert(`Failed to restore snapshot: ${e.toString()}`);
    }
  };

  const handleDelete = async (id: string) => {
    if (!confirm('Delete this snapshot archive?')) return;
    try {
      await tauriApi.deleteSnapshot(id);
      await loadData();
    } catch (e: any) {
      alert(`Failed to delete snapshot: ${e.toString()}`);
    }
  };

  return (
    <div className="panel-view">
      <div className="panel-header" style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <div>
          <h1 className="panel-title">Snapshots & Git Integration</h1>
          <p className="panel-description">
            Automatic point-in-time tarball backups and Git workspace tracking for maximum safety.
          </p>
        </div>
        <button className="btn btn-primary" onClick={() => setModalOpen(true)} disabled={!projectPath.trim()}>
          <Camera size={16} />
          <span>Create Snapshot</span>
        </button>
      </div>

      {/* Target Project Selection */}
      <div className="card">
        <div className="card-title">
          <FolderOpen size={16} />
          <span>Target Workspace</span>
        </div>

        <div style={{ display: 'flex', gap: '12px', alignItems: 'center' }}>
          <input
            type="text"
            className="form-input"
            value={projectPath}
            onChange={(e) => setProjectPath(e.target.value)}
            placeholder="Enter project workspace path (e.g. . or /path/to/project)"
            style={{ flex: 1 }}
          />
          <button className="btn btn-secondary" onClick={loadData} disabled={!projectPath.trim()}>
            <span>Load Workspace</span>
          </button>
        </div>
      </div>

      {/* Git Status Card */}
      {gitStatus && (
        <div className="card">
          <div className="card-title">
            <GitBranch size={16} />
            <span>Git Repository Status</span>
          </div>

          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)', gap: '16px' }}>
            <div style={{ background: 'var(--bg-tertiary)', padding: '12px', borderRadius: 'var(--radius-md)' }}>
              <div style={{ fontSize: '11.5px', color: 'var(--text-muted)' }}>BRANCH</div>
              <div style={{ fontSize: '14px', fontWeight: 600 }}>{gitStatus.branch || 'Detached / Non-Git'}</div>
            </div>

            <div style={{ background: 'var(--bg-tertiary)', padding: '12px', borderRadius: 'var(--radius-md)' }}>
              <div style={{ fontSize: '11.5px', color: 'var(--text-muted)' }}>COMMIT</div>
              <div style={{ fontSize: '14px', fontWeight: 600 }}>
                <code>{gitStatus.head_commit || 'None'}</code>
              </div>
            </div>

            <div style={{ background: 'var(--bg-tertiary)', padding: '12px', borderRadius: 'var(--radius-md)' }}>
              <div style={{ fontSize: '11.5px', color: 'var(--text-muted)' }}>MODIFIED FILES</div>
              <div style={{ fontSize: '14px', fontWeight: 600, color: gitStatus.modified_files.length > 0 ? 'var(--status-warning)' : 'var(--status-success)' }}>
                {gitStatus.modified_files.length} files
              </div>
            </div>

            <div style={{ background: 'var(--bg-tertiary)', padding: '12px', borderRadius: 'var(--radius-md)' }}>
              <div style={{ fontSize: '11.5px', color: 'var(--text-muted)' }}>UNTRACKED FILES</div>
              <div style={{ fontSize: '14px', fontWeight: 600 }}>{gitStatus.untracked_files.length} files</div>
            </div>
          </div>
        </div>
      )}

      {/* Snapshots Table */}
      <div className="card">
        <div className="card-title">
          <Clock size={16} />
          <span>Point-in-Time Snapshots ({snapshots.length})</span>
        </div>

        {snapshots.length > 0 ? (
          <table className="data-table">
            <thead>
              <tr>
                <th>Title / Description</th>
                <th>Snapshot ID</th>
                <th>Git Commit</th>
                <th>Created At</th>
                <th>Actions</th>
              </tr>
            </thead>
            <tbody>
              {snapshots.map((s) => (
                <tr key={s.id}>
                  <td>
                    <div style={{ fontWeight: 600, fontSize: '13.5px' }}>{s.title}</div>
                    {s.description && (
                      <div style={{ fontSize: '12px', color: 'var(--text-secondary)', marginTop: '2px' }}>
                        {s.description}
                      </div>
                    )}
                  </td>
                  <td>
                    <code>{s.id}</code>
                  </td>
                  <td>
                    {s.git_commit ? <code>{s.git_commit}</code> : <span style={{ color: 'var(--text-muted)' }}>-</span>}
                  </td>
                  <td>{new Date(s.created_at).toLocaleString()}</td>
                  <td>
                    <div style={{ display: 'flex', gap: '6px' }}>
                      <button
                        className="btn btn-secondary"
                        style={{ padding: '4px 10px', fontSize: '12px' }}
                        onClick={() => handleRestore(s.id)}
                        title="Restore this snapshot"
                      >
                        <RotateCcw size={13} />
                        <span>Restore</span>
                      </button>
                      <button
                        className="btn-icon"
                        onClick={() => handleDelete(s.id)}
                        title="Delete snapshot"
                      >
                        <Trash2 size={14} color="var(--status-error)" />
                      </button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        ) : (
          <div style={{ padding: '30px', textAlign: 'center', color: 'var(--text-muted)' }}>
            No snapshots recorded yet. Create a snapshot before making large code modifications.
          </div>
        )}
      </div>

      {/* Create Modal */}
      {modalOpen && (
        <div className="modal-backdrop">
          <div className="modal-content">
            <div className="modal-header">
              <h3 style={{ fontSize: '16px', fontWeight: 600 }}>Create Project Snapshot</h3>
              <button className="btn-icon" onClick={() => setModalOpen(false)}>
                <X size={18} />
              </button>
            </div>

            <form onSubmit={handleCreateSnapshot}>
              <div className="modal-body">
                <div className="form-group">
                  <label className="form-label">Snapshot Title</label>
                  <input
                    type="text"
                    className="form-input"
                    value={snapTitle}
                    onChange={(e) => setSnapTitle(e.target.value)}
                    placeholder="e.g. Before agent refactor"
                    required
                  />
                </div>

                <div className="form-group">
                  <label className="form-label">Description (Optional)</label>
                  <textarea
                    className="form-textarea"
                    value={snapDesc}
                    onChange={(e) => setSnapDesc(e.target.value)}
                    placeholder="Additional context about this checkpoint..."
                    rows={3}
                  />
                </div>
              </div>

              <div className="modal-footer">
                <button type="button" className="btn btn-secondary" onClick={() => setModalOpen(false)}>
                  Cancel
                </button>
                <button type="submit" className="btn btn-primary" disabled={creating}>
                  {creating ? 'Archiving...' : 'Save Snapshot'}
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
};
