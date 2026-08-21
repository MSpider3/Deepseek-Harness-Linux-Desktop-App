import React from 'react';
import { Play, Square, RotateCw, ExternalLink, Shield } from 'lucide-react';
import type { DshProcessStatus, ProviderRecord } from '../types';

interface TopBarProps {
  status: DshProcessStatus;
  activeProvider?: ProviderRecord;
  onStart: () => void;
  onStop: () => void;
  onRestart: () => void;
  loading: boolean;
}

export const TopBar: React.FC<TopBarProps> = ({
  status,
  activeProvider,
  onStart,
  onStop,
  onRestart,
  loading,
}) => {
  const isRunning = status.type === 'Running';
  const isStarting = status.type === 'Starting';

  const handleOpenBrowser = () => {
    if (status.type === 'Running') {
      window.open(status.data.url, '_blank');
    }
  };

  return (
    <header className="top-bar">
      <div className="top-bar-left">
        <div className="top-bar-title">
          <span style={{ color: 'var(--text-secondary)', fontSize: '13px' }}>Runtime:</span>
          <span style={{ fontWeight: 600 }}>
            {status.type === 'Running'
              ? `Running (${status.data.url})`
              : status.type === 'Starting'
              ? `Booting on port ${status.data.port}...`
              : status.type}
          </span>
        </div>

        {activeProvider && (
          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '6px',
              padding: '4px 10px',
              borderRadius: 'var(--radius-full)',
              backgroundColor: 'var(--bg-tertiary)',
              border: '1px solid var(--border-subtle)',
              fontSize: '12px',
              color: 'var(--text-secondary)',
            }}
          >
            <Shield size={13} color="var(--accent-cyan)" />
            <span>
              Provider: <strong style={{ color: 'var(--text-primary)' }}>{activeProvider.name}</strong>
            </span>
          </div>
        )}
      </div>

      <div className="top-bar-right">
        {isRunning ? (
          <>
            <button className="btn btn-secondary" onClick={handleOpenBrowser} title="Open in external browser">
              <ExternalLink size={14} />
              <span>Open in Browser</span>
            </button>
            <button className="btn btn-secondary" onClick={onRestart} disabled={loading}>
              <RotateCw size={14} className={loading ? 'animate-spin' : ''} />
              <span>Restart</span>
            </button>
            <button className="btn btn-danger" onClick={onStop} disabled={loading}>
              <Square size={14} />
              <span>Stop</span>
            </button>
          </>
        ) : (
          <button className="btn btn-primary" onClick={onStart} disabled={loading || isStarting}>
            <Play size={14} />
            <span>{isStarting ? 'Starting...' : 'Start DSH'}</span>
          </button>
        )}
      </div>
    </header>
  );
};
