import React from 'react';
import { Play, AlertTriangle, RotateCw, Shield, Terminal, ArrowDownLeft } from 'lucide-react';
import type { DshProcessStatus, RuntimeInfo, ProviderRecord } from '../types';

interface DshWebViewProps {
  status: DshProcessStatus;
  runtimeInfo?: RuntimeInfo;
  activeProvider?: ProviderRecord;
  onStart: () => void;
  onRestart: () => void;
  onRollback?: () => void;
  onOpenSettings: () => void;
}

export const DshWebView: React.FC<DshWebViewProps> = ({
  status,
  runtimeInfo,
  activeProvider,
  onStart,
  onRestart,
  onRollback,
  onOpenSettings,
}) => {
  if (status.type === 'Running') {
    return (
      <div style={{ width: '100%', height: '100%', position: 'relative' }}>
        <iframe
          src={status.data.url}
          className="iframe-container"
          title="Official DeepSeek Harness Web UI"
          allow="clipboard-read; clipboard-write;"
        />
      </div>
    );
  }

  if (status.type === 'Starting') {
    return (
      <div
        className="panel-view"
        style={{
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          justifyContent: 'center',
          textAlign: 'center',
        }}
      >
        <div style={{ marginBottom: '20px' }}>
          <RotateCw size={48} color="var(--accent-cyan)" className="animate-spin" />
        </div>
        <h2 style={{ fontSize: '20px', fontWeight: 600, marginBottom: '8px' }}>
          Booting DeepSeek Harness Runtime...
        </h2>
        <p style={{ color: 'var(--text-secondary)', fontSize: '14px', maxWidth: '460px' }}>
          Initializing isolated Node process, resolving Cordis plugins, and preparing web server on port {status.data.port}.
        </p>
      </div>
    );
  }

  if (status.type === 'Crashed' || status.type === 'Error') {
    const errorMsg = status.type === 'Crashed' ? status.data.message : status.data.message;
    return (
      <div className="panel-view" style={{ maxWidth: '640px', margin: '40px auto' }}>
        <div className="card" style={{ borderColor: 'rgba(239, 68, 68, 0.4)' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '12px', marginBottom: '16px' }}>
            <div
              style={{
                width: '40px',
                height: '40px',
                borderRadius: 'var(--radius-md)',
                backgroundColor: 'rgba(239, 68, 68, 0.15)',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
              }}
            >
              <AlertTriangle size={22} color="var(--status-error)" />
            </div>
            <div>
              <h3 style={{ fontSize: '17px', fontWeight: 600, color: 'var(--status-error)' }}>
                DSH Runtime Error
              </h3>
              <p style={{ color: 'var(--text-secondary)', fontSize: '13px' }}>
                {status.type === 'Crashed'
                  ? `Process exited unexpectedly (crash count: ${status.data.restart_count})`
                  : 'Failed to boot DSH process'}
              </p>
            </div>
          </div>

          <div
            className="code-block"
            style={{ marginBottom: '20px', maxHeight: '180px', overflowY: 'auto' }}
          >
            {errorMsg}
          </div>

          <div style={{ display: 'flex', gap: '10px' }}>
            <button className="btn btn-primary" onClick={onRestart}>
              <RotateCw size={14} />
              <span>Retry Boot</span>
            </button>
            {runtimeInfo?.previous_version && onRollback && (
              <button className="btn btn-secondary" onClick={onRollback}>
                <ArrowDownLeft size={14} />
                <span>Rollback to {runtimeInfo.previous_version}</span>
              </button>
            )}
            <button className="btn btn-secondary" onClick={onOpenSettings}>
              <Terminal size={14} />
              <span>View Logs</span>
            </button>
          </div>
        </div>
      </div>
    );
  }

  // Stopped State
  return (
    <div className="panel-view" style={{ maxWidth: '780px', margin: '40px auto' }}>
      <div className="card" style={{ textAlign: 'center', padding: '40px 30px' }}>
        <div
          style={{
            width: '64px',
            height: '64px',
            borderRadius: 'var(--radius-lg)',
            background: 'var(--accent-gradient)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            margin: '0 auto 20px',
            boxShadow: 'var(--accent-glow)',
          }}
        >
          <Play size={28} color="#ffffff" style={{ marginLeft: '4px' }} />
        </div>

        <h2 style={{ fontSize: '24px', fontWeight: 700, marginBottom: '10px' }}>
          DeepSeek Harness Linux
        </h2>
        <p style={{ color: 'var(--text-secondary)', fontSize: '14.5px', maxWidth: '520px', margin: '0 auto 24px' }}>
          Native desktop shell wrapping the official upstream <code>@deepseek-ai/dsh</code> runtime with isolated versions, safe test workspaces, and encrypted credential storage.
        </p>

        <div style={{ display: 'flex', justifyContent: 'center', gap: '12px', marginBottom: '32px' }}>
          <button className="btn btn-primary" onClick={onStart} style={{ padding: '10px 24px', fontSize: '14px' }}>
            <Play size={16} />
            <span>Launch DSH Web Workspace</span>
          </button>
        </div>

        <div
          style={{
            display: 'grid',
            gridTemplateColumns: 'repeat(3, 1fr)',
            gap: '16px',
            textAlign: 'left',
            marginTop: '20px',
            borderTop: '1px solid var(--border-subtle)',
            paddingTop: '24px',
          }}
        >
          <div style={{ padding: '12px', background: 'var(--bg-tertiary)', borderRadius: 'var(--radius-md)' }}>
            <div style={{ fontSize: '12px', color: 'var(--text-muted)', marginBottom: '4px' }}>DSH VERSION</div>
            <div style={{ fontSize: '13.5px', fontWeight: 600 }}>
              {runtimeInfo?.current_version || '0.1.0-rc.8 (Official)'}
            </div>
          </div>

          <div style={{ padding: '12px', background: 'var(--bg-tertiary)', borderRadius: 'var(--radius-md)' }}>
            <div style={{ fontSize: '12px', color: 'var(--text-muted)', marginBottom: '4px' }}>DEFAULT PROVIDER</div>
            <div style={{ fontSize: '13.5px', fontWeight: 600 }}>
              {activeProvider?.name || 'DeepSeek (Official)'}
            </div>
          </div>

          <div style={{ padding: '12px', background: 'var(--bg-tertiary)', borderRadius: 'var(--radius-md)' }}>
            <div style={{ fontSize: '12px', color: 'var(--text-muted)', marginBottom: '4px' }}>SECURITY</div>
            <div style={{ fontSize: '13.5px', fontWeight: 600, color: 'var(--status-success)', display: 'flex', alignItems: 'center', gap: '4px' }}>
              <Shield size={14} /> Keyring Protected
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
