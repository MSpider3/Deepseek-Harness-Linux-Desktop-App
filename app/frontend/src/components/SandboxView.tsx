import React, { useState } from 'react';
import {
  ShieldCheck,
  Play,
  CheckCircle2,
  XCircle,
  FileCode,
  Check,
  Trash2,
  RotateCw,
  FolderOpen
} from 'lucide-react';
import type { WorkspaceMetadata, TestResult, DiffFileSummary } from '../types';
import { tauriApi } from '../services/tauriApi';

export const SandboxView: React.FC = () => {
  const [projectPath, setProjectPath] = useState('.');
  const [workspace, setWorkspace] = useState<WorkspaceMetadata | null>(null);
  const [testCmd, setTestCmd] = useState('');
  const [testResult, setTestResult] = useState<TestResult | null>(null);
  const [runningTests, setRunningTests] = useState(false);
  const [diffSummaries, setDiffSummaries] = useState<DiffFileSummary[]>([]);
  const [loadingDiff, setLoadingDiff] = useState(false);
  const [applying, setApplying] = useState(false);

  const handleCreateWorkspace = async () => {
    if (!projectPath.trim()) return;
    try {
      const ws = await tauriApi.createSandboxWorkspace(projectPath.trim());
      setWorkspace(ws);
      setTestCmd(ws.default_test_command);
      setTestResult(null);
      setDiffSummaries([]);
      await handleRefreshDiff(ws.original_path, ws.workspace_path);
    } catch (e: any) {
      alert(`Failed to create staging workspace: ${e.toString()}`);
    }
  };

  const handleRunTests = async () => {
    if (!workspace || !testCmd.trim()) return;
    setRunningTests(true);
    setTestResult(null);
    try {
      const res = await tauriApi.runSandboxTests(workspace.workspace_path, testCmd.trim());
      setTestResult(res);
      await handleRefreshDiff(workspace.original_path, workspace.workspace_path);
    } catch (e: any) {
      alert(`Test execution failed: ${e.toString()}`);
    } finally {
      setRunningTests(false);
    }
  };

  const handleRefreshDiff = async (orig: string, ws: string) => {
    setLoadingDiff(true);
    try {
      const diffs = await tauriApi.getSandboxDiff(orig, ws);
      setDiffSummaries(diffs);
    } catch (e: any) {
      console.error('Failed to compute diff', e);
    } finally {
      setLoadingDiff(false);
    }
  };

  const handleApplyChanges = async () => {
    if (!workspace) return;
    if (!confirm('Apply sandbox changes back to your original project? A safety checkpoint snapshot will be created automatically.')) return;
    setApplying(true);
    try {
      const count = await tauriApi.applySandboxChanges(workspace.original_path, workspace.workspace_path);
      alert(`Successfully applied changes to ${count} file(s)!`);
      await handleRefreshDiff(workspace.original_path, workspace.workspace_path);
    } catch (e: any) {
      alert(`Failed to apply changes: ${e.toString()}`);
    } finally {
      setApplying(false);
    }
  };

  const handleDiscard = async () => {
    if (!workspace) return;
    if (!confirm('Discard staging workspace and all unapplied modifications?')) return;
    try {
      await tauriApi.discardSandbox(workspace.workspace_path);
      setWorkspace(null);
      setTestResult(null);
      setDiffSummaries([]);
    } catch (e: any) {
      alert(`Failed to discard workspace: ${e.toString()}`);
    }
  };

  return (
    <div className="panel-view">
      <div className="panel-header" style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <div>
          <h1 className="panel-title">Sandbox & Safe Testing Workspace</h1>
          <p className="panel-description">
            Safely test AI modifications in isolated staging workspaces with automated test gating and diff review.
          </p>
        </div>
      </div>

      {/* Target Project Selection */}
      <div className="card">
        <div className="card-title">
          <FolderOpen size={16} />
          <span>Project Workspace</span>
        </div>

        <div style={{ display: 'flex', gap: '12px', alignItems: 'center' }}>
          <input
            type="text"
            className="form-input"
            value={projectPath}
            onChange={(e) => setProjectPath(e.target.value)}
            placeholder="/path/to/your/project"
            style={{ flex: 1 }}
          />
          <button className="btn btn-primary" onClick={handleCreateWorkspace}>
            <ShieldCheck size={16} />
            <span>{workspace ? 'Recreate Staging' : 'Create Staging Workspace'}</span>
          </button>
        </div>
      </div>

      {workspace && (
        <>
          {/* Active Staging Workspace Info */}
          <div className="card" style={{ border: '1px solid var(--accent-cyan)' }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '16px' }}>
              <div>
                <div style={{ fontSize: '16px', fontWeight: 600, display: 'flex', alignItems: 'center', gap: '8px' }}>
                  <span>Active Staging:</span>
                  <code>{workspace.workspace_id}</code>
                  <span
                    style={{
                      fontSize: '11px',
                      textTransform: 'uppercase',
                      padding: '2px 8px',
                      borderRadius: '4px',
                      background: 'rgba(6, 182, 212, 0.2)',
                      color: 'var(--accent-cyan)',
                      fontWeight: 600,
                    }}
                  >
                    {workspace.project_type} Project
                  </span>
                </div>
                <div style={{ fontSize: '12.5px', color: 'var(--text-secondary)', marginTop: '4px' }}>
                  Staging Path: <code>{workspace.workspace_path}</code>
                </div>
              </div>

              <div style={{ display: 'flex', gap: '10px' }}>
                <button className="btn btn-secondary" onClick={() => handleRefreshDiff(workspace.original_path, workspace.workspace_path)}>
                  <RotateCw size={14} className={loadingDiff ? 'animate-spin' : ''} />
                  <span>Refresh Diff</span>
                </button>
                <button className="btn btn-danger" onClick={handleDiscard}>
                  <Trash2 size={14} />
                  <span>Discard</span>
                </button>
              </div>
            </div>

            {/* Test Runner Controls */}
            <div style={{ background: 'var(--bg-tertiary)', padding: '16px', borderRadius: 'var(--radius-md)', marginBottom: '16px' }}>
              <div style={{ fontSize: '13px', fontWeight: 600, marginBottom: '8px', color: 'var(--text-secondary)' }}>
                AUTOMATED TEST GATING
              </div>
              <div style={{ display: 'flex', gap: '12px' }}>
                <input
                  type="text"
                  className="form-input"
                  value={testCmd}
                  onChange={(e) => setTestCmd(e.target.value)}
                  placeholder="e.g. npm test or pytest or cargo test"
                  style={{ flex: 1 }}
                />
                <button className="btn btn-primary" onClick={handleRunTests} disabled={runningTests}>
                  <Play size={14} className={runningTests ? 'animate-spin' : ''} />
                  <span>{runningTests ? 'Running Tests...' : 'Run Tests in Sandbox'}</span>
                </button>
              </div>

              {testResult && (
                <div style={{ marginTop: '16px' }}>
                  <div
                    style={{
                      display: 'flex',
                      alignItems: 'center',
                      gap: '8px',
                      marginBottom: '8px',
                      color: testResult.success ? 'var(--status-success)' : 'var(--status-error)',
                      fontWeight: 600,
                      fontSize: '13.5px',
                    }}
                  >
                    {testResult.success ? <CheckCircle2 size={16} /> : <XCircle size={16} />}
                    <span>
                      {testResult.success
                        ? `Tests Passed in ${testResult.duration_ms}ms (Exit 0)`
                        : `Tests Failed with Exit Code ${testResult.exit_code}`}
                    </span>
                  </div>

                  <div className="code-block" style={{ maxHeight: '200px', overflowY: 'auto' }}>
                    {testResult.stdout}
                    {testResult.stderr && (
                      <div style={{ color: 'var(--status-error)', marginTop: '8px' }}>
                        {testResult.stderr}
                      </div>
                    )}
                  </div>
                </div>
              )}
            </div>

            {/* Diff & Changes Review */}
            <div>
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '12px' }}>
                <h3 style={{ fontSize: '14px', fontWeight: 600, color: 'var(--text-secondary)', textTransform: 'uppercase' }}>
                  Changed Files ({diffSummaries.length})
                </h3>

                {diffSummaries.length > 0 && (
                  <button className="btn btn-primary" onClick={handleApplyChanges} disabled={applying}>
                    <Check size={15} />
                    <span>Approve & Apply to Original Project</span>
                  </button>
                )}
              </div>

              {diffSummaries.length > 0 ? (
                <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
                  {diffSummaries.map((d) => (
                    <div
                      key={d.file_path}
                      style={{
                        border: '1px solid var(--border-subtle)',
                        borderRadius: 'var(--radius-md)',
                        overflow: 'hidden',
                        background: 'var(--bg-primary)',
                      }}
                    >
                      <div
                        style={{
                          padding: '10px 14px',
                          background: 'var(--bg-tertiary)',
                          display: 'flex',
                          justifyContent: 'space-between',
                          alignItems: 'center',
                          fontSize: '13px',
                        }}
                      >
                        <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                          <FileCode size={16} color="var(--accent-cyan)" />
                          <strong>{d.file_path}</strong>
                          <span
                            style={{
                              fontSize: '11px',
                              padding: '2px 6px',
                              borderRadius: '4px',
                              textTransform: 'uppercase',
                              background:
                                d.status === 'added'
                                  ? 'rgba(16, 185, 129, 0.2)'
                                  : d.status === 'deleted'
                                  ? 'rgba(239, 68, 68, 0.2)'
                                  : 'rgba(59, 130, 246, 0.2)',
                              color:
                                d.status === 'added'
                                  ? 'var(--status-success)'
                                  : d.status === 'deleted'
                                  ? 'var(--status-error)'
                                  : 'var(--accent-blue)',
                            }}
                          >
                            {d.status}
                          </span>
                        </div>
                        <div style={{ fontSize: '12px', display: 'flex', gap: '8px' }}>
                          <span style={{ color: 'var(--status-success)' }}>+{d.additions}</span>
                          <span style={{ color: 'var(--status-error)' }}>-{d.deletions}</span>
                        </div>
                      </div>

                      <pre
                        style={{
                          margin: 0,
                          padding: '12px',
                          fontSize: '12px',
                          fontFamily: 'var(--font-mono)',
                          overflowX: 'auto',
                          lineHeight: '1.5',
                        }}
                      >
                        {d.unified_diff.split('\n').map((line, idx) => {
                          const isAdd = line.startsWith('+');
                          const isDel = line.startsWith('-');
                          return (
                            <div
                              key={idx}
                              className={isAdd ? 'diff-line-add' : isDel ? 'diff-line-del' : ''}
                            >
                              {line}
                            </div>
                          );
                        })}
                      </pre>
                    </div>
                  ))}
                </div>
              ) : (
                <div style={{ padding: '24px', textAlign: 'center', color: 'var(--text-muted)' }}>
                  No modified files detected between original project and sandbox staging workspace.
                </div>
              )}
            </div>
          </div>
        </>
      )}
    </div>
  );
};
