import React, { useState, useEffect } from 'react';
import {
  RefreshCw,
  ArrowDownLeft,
  CheckCircle,
  AlertTriangle,
  Clock,
  Layers,
  Sparkles,
  Download
} from 'lucide-react';
import type { RuntimeInfo, UpdateCheckResult, UpdateHistoryRecord, UpdateResult } from '../types';
import { tauriApi } from '../services/tauriApi';

interface UpdatesViewProps {
  runtimeInfo?: RuntimeInfo;
  onRuntimeUpdated?: () => void;
}

export const UpdatesView: React.FC<UpdatesViewProps> = ({ runtimeInfo, onRuntimeUpdated }) => {
  const [channel, setChannel] = useState<'stable' | 'rc' | 'dev'>('stable');
  const [checking, setChecking] = useState(false);
  const [checkResult, setCheckResult] = useState<UpdateCheckResult | null>(null);
  const [updating, setUpdating] = useState(false);
  const [updateResult, setUpdateResult] = useState<UpdateResult | null>(null);
  const [history, setHistory] = useState<UpdateHistoryRecord[]>([]);

  const loadHistory = async () => {
    try {
      const hList = await tauriApi.getUpdateHistory();
      setHistory(hList);
    } catch (e) {
      console.error('Failed to load update history', e);
    }
  };

  useEffect(() => {
    loadHistory();
  }, []);

  const handleCheckUpdates = async () => {
    setChecking(true);
    setCheckResult(null);
    setUpdateResult(null);
    try {
      const res = await tauriApi.checkForUpdates(channel);
      setCheckResult(res);
    } catch (e: any) {
      alert(`Update check failed: ${e.toString()}`);
    } finally {
      setChecking(false);
    }
  };

  const handleApplyUpdate = async (targetVer: string) => {
    setUpdating(true);
    setUpdateResult(null);
    try {
      const res = await tauriApi.applyUpdate(targetVer);
      setUpdateResult(res);
      await loadHistory();
      onRuntimeUpdated?.();
    } catch (e: any) {
      alert(`Update failed: ${e.toString()}`);
    } finally {
      setUpdating(false);
    }
  };

  const handleRollback = async () => {
    if (!confirm('Are you sure you want to roll back to the previous version?')) return;
    setUpdating(true);
    try {
      const res = await tauriApi.rollbackRuntime();
      setUpdateResult(res);
      await loadHistory();
      onRuntimeUpdated?.();
    } catch (e: any) {
      alert(`Rollback failed: ${e.toString()}`);
    } finally {
      setUpdating(false);
    }
  };

  return (
    <div className="panel-view">
      <div className="panel-header" style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <div>
          <h1 className="panel-title">Runtime & Atomic Updates</h1>
          <p className="panel-description">
            Manage isolated DeepSeek Harness versions, atomic update channels, and instant 1-click rollbacks.
          </p>
        </div>
        <div style={{ display: 'flex', gap: '10px' }}>
          {runtimeInfo?.previous_version && (
            <button className="btn btn-secondary" onClick={handleRollback} disabled={updating}>
              <ArrowDownLeft size={16} />
              <span>Rollback to {runtimeInfo.previous_version}</span>
            </button>
          )}
          <button className="btn btn-primary" onClick={handleCheckUpdates} disabled={checking || updating}>
            <RefreshCw size={16} className={checking ? 'animate-spin' : ''} />
            <span>{checking ? 'Checking...' : 'Check for Updates'}</span>
          </button>
        </div>
      </div>

      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '18px', marginBottom: '24px' }}>
        <div className="card" style={{ marginBottom: 0 }}>
          <div style={{ fontSize: '12px', color: 'var(--text-muted)', marginBottom: '4px' }}>CURRENT ACTIVE DSH</div>
          <div style={{ fontSize: '20px', fontWeight: 700, color: 'var(--accent-cyan)' }}>
            {runtimeInfo?.current_version || 'Not Installed'}
          </div>
          <div style={{ fontSize: '12px', color: 'var(--text-secondary)', marginTop: '4px' }}>
            Isolated Symlink: <code>runtime/current</code>
          </div>
        </div>

        <div className="card" style={{ marginBottom: 0 }}>
          <div style={{ fontSize: '12px', color: 'var(--text-muted)', marginBottom: '4px' }}>PREVIOUS KNOWN-GOOD</div>
          <div style={{ fontSize: '20px', fontWeight: 700, color: runtimeInfo?.previous_version ? 'var(--text-primary)' : 'var(--text-muted)' }}>
            {runtimeInfo?.previous_version || 'None'}
          </div>
          <div style={{ fontSize: '12px', color: 'var(--text-secondary)', marginTop: '4px' }}>
            Rollback Target: <code>runtime/previous</code>
          </div>
        </div>

        <div className="card" style={{ marginBottom: 0 }}>
          <div style={{ fontSize: '12px', color: 'var(--text-muted)', marginBottom: '4px' }}>UPDATE CHANNEL</div>
          <div style={{ display: 'flex', gap: '6px', marginTop: '6px' }}>
            {(['stable', 'rc', 'dev'] as const).map((ch) => (
              <button
                key={ch}
                onClick={() => setChannel(ch)}
                style={{
                  padding: '5px 10px',
                  borderRadius: 'var(--radius-sm)',
                  border: `1px solid ${channel === ch ? 'var(--accent-blue)' : 'var(--border-subtle)'}`,
                  backgroundColor: channel === ch ? 'var(--bg-elevated)' : 'var(--bg-tertiary)',
                  color: channel === ch ? 'var(--accent-cyan)' : 'var(--text-secondary)',
                  fontSize: '12px',
                  cursor: 'pointer',
                  textTransform: 'uppercase',
                  fontWeight: 600,
                }}
              >
                {ch}
              </button>
            ))}
          </div>
        </div>
      </div>

      {/* Update Check Results Card */}
      {checkResult && (
        <div className="card" style={{ border: '1px solid var(--accent-blue)', background: 'linear-gradient(180deg, var(--bg-secondary) 0%, rgba(2, 132, 199, 0.08) 100%)' }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
              <div
                style={{
                  width: '42px',
                  height: '42px',
                  borderRadius: 'var(--radius-md)',
                  background: checkResult.has_update ? 'var(--accent-gradient)' : 'rgba(16, 185, 129, 0.15)',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                }}
              >
                {checkResult.has_update ? <Sparkles size={20} color="#fff" /> : <CheckCircle size={20} color="var(--status-success)" />}
              </div>
              <div>
                <div style={{ fontSize: '16px', fontWeight: 600 }}>
                  {checkResult.has_update
                    ? `Update Available: ${checkResult.target_version}`
                    : `You're on the latest ${channel.toUpperCase()} version (${checkResult.target_version})`}
                </div>
                <div style={{ fontSize: '13px', color: 'var(--text-secondary)' }}>
                  Official npm package: <code>@deepseek-ai/dsh@{checkResult.target_version}</code>
                </div>
              </div>
            </div>

            {checkResult.has_update && (
              <button
                className="btn btn-primary"
                onClick={() => handleApplyUpdate(checkResult.target_version)}
                disabled={updating}
              >
                <Download size={16} className={updating ? 'animate-spin' : ''} />
                <span>{updating ? 'Installing Atomically...' : 'Install & Activate'}</span>
              </button>
            )}
          </div>
        </div>
      )}

      {/* Update Success/Feedback Alert */}
      {updateResult && (
        <div
          style={{
            padding: '14px 18px',
            borderRadius: 'var(--radius-md)',
            backgroundColor: updateResult.success ? 'rgba(16, 185, 129, 0.15)' : 'rgba(239, 68, 68, 0.15)',
            border: `1px solid ${updateResult.success ? 'var(--status-success)' : 'var(--status-error)'}`,
            color: updateResult.success ? 'var(--status-success)' : 'var(--status-error)',
            marginBottom: '20px',
            fontSize: '13.5px',
            display: 'flex',
            alignItems: 'center',
            gap: '10px',
          }}
        >
          {updateResult.success ? <CheckCircle size={18} /> : <AlertTriangle size={18} />}
          <span>{updateResult.message}</span>
        </div>
      )}

      {/* Installed Versions & Update History */}
      <div className="card">
        <div className="card-title">
          <Layers size={16} />
          <span>Installed Runtime Versions</span>
        </div>

        {runtimeInfo?.versions && runtimeInfo.versions.length > 0 ? (
          <table className="data-table">
            <thead>
              <tr>
                <th>Version</th>
                <th>Installed Path</th>
                <th>Status</th>
                <th>Installed Date</th>
                <th>Actions</th>
              </tr>
            </thead>
            <tbody>
              {runtimeInfo.versions.map((ver) => (
                <tr key={ver.version}>
                  <td>
                    <strong>{ver.version}</strong>
                  </td>
                  <td>
                    <code>{ver.path}</code>
                  </td>
                  <td>
                    {ver.is_current ? (
                      <span style={{ fontSize: '11px', background: 'rgba(16, 185, 129, 0.2)', color: 'var(--status-success)', padding: '2px 8px', borderRadius: '4px' }}>
                        ACTIVE
                      </span>
                    ) : ver.is_previous ? (
                      <span style={{ fontSize: '11px', background: 'rgba(59, 130, 246, 0.2)', color: 'var(--accent-blue)', padding: '2px 8px', borderRadius: '4px' }}>
                        PREVIOUS
                      </span>
                    ) : (
                      <span style={{ fontSize: '11px', color: 'var(--text-muted)' }}>INACTIVE</span>
                    )}
                  </td>
                  <td>{new Date(ver.installed_at).toLocaleDateString()}</td>
                  <td>
                    {!ver.is_current && (
                      <button
                        className="btn btn-secondary"
                        style={{ padding: '4px 10px', fontSize: '12px' }}
                        onClick={async () => {
                          await tauriApi.activateRuntimeVersion(ver.version);
                          onRuntimeUpdated?.();
                        }}
                      >
                        Activate
                      </button>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        ) : (
          <div style={{ padding: '20px', textAlign: 'center', color: 'var(--text-muted)' }}>
            No isolated runtime versions installed. Click Check for Updates or Install.
          </div>
        )}
      </div>

      {/* Update History */}
      {history.length > 0 && (
        <div className="card">
          <div className="card-title">
            <Clock size={16} />
            <span>Update & Rollback History</span>
          </div>

          <table className="data-table">
            <thead>
              <tr>
                <th>From</th>
                <th>To</th>
                <th>Status</th>
                <th>Message</th>
                <th>Timestamp</th>
              </tr>
            </thead>
            <tbody>
              {history.map((h) => (
                <tr key={h.id}>
                  <td>{h.from_version || 'Initial'}</td>
                  <td>
                    <strong>{h.to_version}</strong>
                  </td>
                  <td>
                    <span
                      style={{
                        fontSize: '11px',
                        padding: '2px 6px',
                        borderRadius: '4px',
                        background:
                          h.status === 'success'
                            ? 'rgba(16, 185, 129, 0.15)'
                            : h.status === 'rolled_back'
                            ? 'rgba(245, 158, 11, 0.15)'
                            : 'rgba(239, 68, 68, 0.15)',
                        color:
                          h.status === 'success'
                            ? 'var(--status-success)'
                            : h.status === 'rolled_back'
                            ? 'var(--status-warning)'
                            : 'var(--status-error)',
                      }}
                    >
                      {h.status.toUpperCase()}
                    </span>
                  </td>
                  <td>{h.error_message || 'OK'}</td>
                  <td>{new Date(h.timestamp).toLocaleString()}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
};
