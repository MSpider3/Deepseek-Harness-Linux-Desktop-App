import React, { useState, useEffect, useRef } from 'react';
import { Search, Download, RotateCw } from 'lucide-react';
import type { ProcessLogEntry } from '../types';
import { tauriApi } from '../services/tauriApi';

export const LogsView: React.FC = () => {
  const [logs, setLogs] = useState<ProcessLogEntry[]>([]);
  const [filterStream, setFilterStream] = useState<'all' | 'stdout' | 'stderr' | 'system'>('all');
  const [searchQuery, setSearchQuery] = useState('');
  const [autoScroll, setAutoScroll] = useState(true);
  const logEndRef = useRef<HTMLDivElement>(null);

  const fetchLogs = async () => {
    try {
      const lList = await tauriApi.getDshLogs(300);
      setLogs(lList);
    } catch (e) {
      console.error('Failed to fetch logs', e);
    }
  };

  useEffect(() => {
    fetchLogs();
    const interval = setInterval(fetchLogs, 1500);
    return () => clearInterval(interval);
  }, []);

  useEffect(() => {
    if (autoScroll && logEndRef.current) {
      logEndRef.current.scrollIntoView({ behavior: 'smooth' });
    }
  }, [logs, autoScroll]);

  const filteredLogs = logs.filter((l) => {
    if (filterStream !== 'all' && l.stream !== filterStream) return false;
    if (searchQuery.trim()) {
      return l.message.toLowerCase().includes(searchQuery.toLowerCase());
    }
    return true;
  });

  const handleExportLogs = () => {
    const text = logs.map((l) => `[${l.timestamp}] [${l.stream.toUpperCase()}] ${l.message}`).join('\n');
    const blob = new Blob([text], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `dsh_runtime_logs_${Date.now()}.log`;
    a.click();
    URL.revokeObjectURL(url);
  };

  return (
    <div className="panel-view">
      <div className="panel-header" style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <div>
          <h1 className="panel-title">Process & Runtime Logs</h1>
          <p className="panel-description">
            Live sanitized stdout, stderr, and system logs from the DeepSeek Harness supervisor.
          </p>
        </div>
        <div style={{ display: 'flex', gap: '10px' }}>
          <button className="btn btn-secondary" onClick={handleExportLogs}>
            <Download size={14} />
            <span>Export Logs</span>
          </button>
          <button className="btn btn-secondary" onClick={fetchLogs}>
            <RotateCw size={14} />
            <span>Refresh</span>
          </button>
        </div>
      </div>

      {/* Log Filters Bar */}
      <div className="card" style={{ padding: '12px 16px', marginBottom: '16px', display: 'flex', gap: '16px', alignItems: 'center' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '8px', flex: 1 }}>
          <Search size={16} color="var(--text-muted)" />
          <input
            type="text"
            className="form-input"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Search logs..."
            style={{ padding: '6px 10px', fontSize: '13px' }}
          />
        </div>

        <div style={{ display: 'flex', gap: '6px' }}>
          {(['all', 'stdout', 'stderr', 'system'] as const).map((st) => (
            <button
              key={st}
              onClick={() => setFilterStream(st)}
              style={{
                padding: '4px 10px',
                borderRadius: 'var(--radius-sm)',
                border: `1px solid ${filterStream === st ? 'var(--accent-blue)' : 'var(--border-subtle)'}`,
                backgroundColor: filterStream === st ? 'var(--bg-elevated)' : 'var(--bg-tertiary)',
                color: filterStream === st ? 'var(--accent-cyan)' : 'var(--text-secondary)',
                fontSize: '12px',
                cursor: 'pointer',
                textTransform: 'uppercase',
                fontWeight: 600,
              }}
            >
              {st}
            </button>
          ))}
        </div>

        <label style={{ display: 'flex', alignItems: 'center', gap: '6px', fontSize: '12.5px', color: 'var(--text-secondary)', cursor: 'pointer' }}>
          <input
            type="checkbox"
            checked={autoScroll}
            onChange={(e) => setAutoScroll(e.target.checked)}
          />
          <span>Auto-scroll</span>
        </label>
      </div>

      {/* Terminal Output Area */}
      <div
        className="code-block"
        style={{
          height: 'calc(100vh - 270px)',
          overflowY: 'auto',
          padding: '16px',
          fontSize: '12px',
          lineHeight: '1.7',
        }}
      >
        {filteredLogs.length > 0 ? (
          filteredLogs.map((l, idx) => {
            const isStderr = l.stream === 'stderr';
            const isSystem = l.stream === 'system';
            return (
              <div key={idx} style={{ display: 'flex', gap: '10px' }}>
                <span style={{ color: 'var(--text-muted)', userSelect: 'none', minWidth: '70px' }}>
                  {new Date(l.timestamp).toLocaleTimeString()}
                </span>
                <span
                  style={{
                    minWidth: '55px',
                    userSelect: 'none',
                    fontWeight: 600,
                    color: isStderr
                      ? 'var(--status-error)'
                      : isSystem
                      ? 'var(--accent-cyan)'
                      : 'var(--text-secondary)',
                  }}
                >
                  [{l.stream.toUpperCase()}]
                </span>
                <span style={{ color: isStderr ? '#fca5a5' : '#f1f5f9', whiteSpace: 'pre-wrap', wordBreak: 'break-all' }}>
                  {l.message}
                </span>
              </div>
            );
          })
        ) : (
          <div style={{ color: 'var(--text-muted)', textAlign: 'center', padding: '40px' }}>
            No log records matching filter.
          </div>
        )}
        <div ref={logEndRef} />
      </div>
    </div>
  );
};
