import React, { useState, useEffect } from 'react';
import {
  Download,
  AppWindow,
  Shield,
  Info
} from 'lucide-react';
import type { RuntimeInfo } from '../types';
import { tauriApi } from '../services/tauriApi';

interface SettingsViewProps {
  runtimeInfo?: RuntimeInfo;
}

export const SettingsView: React.FC<SettingsViewProps> = ({ runtimeInfo }) => {
  const [autostart, setAutostart] = useState(false);
  const [defaultPort, setDefaultPort] = useState('5180');
  const [disableTelemetry, setDisableTelemetry] = useState(true);
  const [exporting, setExporting] = useState(false);

  useEffect(() => {
    tauriApi.getSetting('autostart').then((val) => setAutostart(val === 'true'));
    tauriApi.getSetting('default_port').then((val) => setDefaultPort(val || '5180'));
    tauriApi.getSetting('disable_telemetry').then((val) => setDisableTelemetry(val !== 'false'));
  }, []);

  const handleToggleAutostart = async (checked: boolean) => {
    setAutostart(checked);
    await tauriApi.setSetting('autostart', checked ? 'true' : 'false');
  };

  const handleSavePort = async (val: string) => {
    setDefaultPort(val);
    await tauriApi.setSetting('default_port', val);
  };

  const handleToggleTelemetry = async (checked: boolean) => {
    setDisableTelemetry(checked);
    await tauriApi.setSetting('disable_telemetry', checked ? 'true' : 'false');
  };

  const handleInstallDesktopLauncher = async () => {
    try {
      const path = await tauriApi.installDesktopLauncher();
      alert(`Successfully installed desktop launcher at ${path}!`);
    } catch (e: any) {
      alert(`Failed to install launcher: ${e.toString()}`);
    }
  };

  const handleExportDiagnostics = async () => {
    setExporting(true);
    try {
      const target = `/tmp/dsh_diagnostics_${Date.now()}.json`;
      await tauriApi.exportDiagnostics(target);
      alert(`Diagnostics exported to ${target}`);
    } catch (e: any) {
      alert(`Failed to export diagnostics: ${e.toString()}`);
    } finally {
      setExporting(false);
    }
  };

  return (
    <div className="panel-view" style={{ maxWidth: '860px' }}>
      <div className="panel-header">
        <h1 className="panel-title">Settings & Linux Integration</h1>
        <p className="panel-description">
          Configure native desktop integration, privacy, diagnostics, and application preferences.
        </p>
      </div>

      {/* Linux Integration Section */}
      <div className="card">
        <div className="card-title">
          <AppWindow size={16} />
          <span>Linux Desktop Integration</span>
        </div>

        <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <div>
              <div style={{ fontWeight: 600, fontSize: '14px' }}>Freedesktop .desktop Launcher</div>
              <div style={{ fontSize: '12.5px', color: 'var(--text-secondary)' }}>
                Integrate into GNOME/KDE application menus and search launcher.
              </div>
            </div>
            <button className="btn btn-secondary" onClick={handleInstallDesktopLauncher}>
              <span>Install .desktop File</span>
            </button>
          </div>

          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', borderTop: '1px solid var(--border-subtle)', paddingTop: '14px' }}>
            <div>
              <div style={{ fontWeight: 600, fontSize: '14px' }}>Launch on System Startup</div>
              <div style={{ fontSize: '12.5px', color: 'var(--text-secondary)' }}>
                Automatically start minimized to the system tray on login.
              </div>
            </div>
            <input
              type="checkbox"
              checked={autostart}
              onChange={(e) => handleToggleAutostart(e.target.checked)}
              style={{ width: '18px', height: '18px', cursor: 'pointer' }}
            />
          </div>
        </div>
      </div>

      {/* DSH Runtime & Privacy Settings */}
      <div className="card">
        <div className="card-title">
          <Shield size={16} />
          <span>Runtime & Privacy</span>
        </div>

        <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <div>
              <div style={{ fontWeight: 600, fontSize: '14px' }}>Disable Telemetry (DSH_TELEMETRY_DISABLED)</div>
              <div style={{ fontSize: '12.5px', color: 'var(--text-secondary)' }}>
                Disables OpenTelemetry session reporting in all booted profiles.
              </div>
            </div>
            <input
              type="checkbox"
              checked={disableTelemetry}
              onChange={(e) => handleToggleTelemetry(e.target.checked)}
              style={{ width: '18px', height: '18px', cursor: 'pointer' }}
            />
          </div>

          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', borderTop: '1px solid var(--border-subtle)', paddingTop: '14px' }}>
            <div>
              <div style={{ fontWeight: 600, fontSize: '14px' }}>Default Web Port</div>
              <div style={{ fontSize: '12.5px', color: 'var(--text-secondary)' }}>
                Starting port for local loopback web server binding.
              </div>
            </div>
            <input
              type="number"
              className="form-input"
              value={defaultPort}
              onChange={(e) => handleSavePort(e.target.value)}
              style={{ width: '100px', textAlign: 'center' }}
            />
          </div>
        </div>
      </div>

      {/* Diagnostics & Export */}
      <div className="card">
        <div className="card-title">
          <Download size={16} />
          <span>Diagnostics & Troubleshooting</span>
        </div>

        <p style={{ fontSize: '13px', color: 'var(--text-secondary)', marginBottom: '16px' }}>
          Export sanitized diagnostic bundle containing process status, anonymized provider metadata, and recent logs without leaking secrets or API keys.
        </p>

        <button className="btn btn-secondary" onClick={handleExportDiagnostics} disabled={exporting}>
          <Download size={14} />
          <span>{exporting ? 'Exporting...' : 'Export Diagnostics JSON'}</span>
        </button>
      </div>

      {/* System Information Card */}
      <div className="card" style={{ background: 'var(--bg-tertiary)' }}>
        <div className="card-title">
          <Info size={16} />
          <span>System & Environment</span>
        </div>

        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)', gap: '12px', fontSize: '13px' }}>
          <div>
            <span style={{ color: 'var(--text-muted)' }}>Application: </span>
            <strong>DeepSeek Harness Linux v0.1.0</strong>
          </div>
          <div>
            <span style={{ color: 'var(--text-muted)' }}>Tauri Core: </span>
            <strong>2.2.4 (Rust 1.97)</strong>
          </div>
          <div>
            <span style={{ color: 'var(--text-muted)' }}>Node.js Runtime: </span>
            <strong>{runtimeInfo?.node_version || 'v22.22.2'}</strong>
          </div>
          <div>
            <span style={{ color: 'var(--text-muted)' }}>Upstream DSH: </span>
            <strong>@deepseek-ai/dsh (Official)</strong>
          </div>
          <div>
            <span style={{ color: 'var(--text-muted)' }}>Secret Store: </span>
            <strong>Linux Keyring / AES-GCM Vault</strong>
          </div>
          <div>
            <span style={{ color: 'var(--text-muted)' }}>Database: </span>
            <strong>SQLite 3 (WAL mode)</strong>
          </div>
        </div>
      </div>
    </div>
  );
};
