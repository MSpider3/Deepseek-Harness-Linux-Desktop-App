import React, { useState, useEffect } from 'react';
import { Sidebar, NavTab } from './components/Sidebar';
import { TopBar } from './components/TopBar';
import { DshWebView } from './components/DshWebView';
import { ProvidersView } from './components/ProvidersView';
import { UpdatesView } from './components/UpdatesView';
import { SandboxView } from './components/SandboxView';
import { SnapshotsView } from './components/SnapshotsView';
import { LogsView } from './components/LogsView';
import { SettingsView } from './components/SettingsView';
import { FirstRunModal } from './components/FirstRunModal';
import { tauriApi } from './services/tauriApi';
import type { DshProcessStatus, RuntimeInfo, ProviderRecord } from './types';

export const App: React.FC = () => {
  const [currentTab, setCurrentTab] = useState<NavTab>('chat');
  const [sidebarCollapsed, setSidebarCollapsed] = useState(false);
  const [status, setStatus] = useState<DshProcessStatus>({ type: 'Stopped' });
  const [runtimeInfo, setRuntimeInfo] = useState<RuntimeInfo | undefined>(undefined);
  const [providers, setProviders] = useState<ProviderRecord[]>([]);
  const [loading, setLoading] = useState(false);
  const [showFirstRun, setShowFirstRun] = useState(false);

  const refreshRuntimeAndProviders = async () => {
    try {
      const rInfo = await tauriApi.getRuntimeInfo();
      setRuntimeInfo(rInfo);

      const pList = await tauriApi.listProviders();
      setProviders(pList);

      if (!rInfo.is_installed || pList.length === 0) {
        setShowFirstRun(true);
      }
    } catch (e) {
      console.error('Failed to load runtime or providers', e);
    }
  };

  const refreshStatus = async () => {
    try {
      const st = await tauriApi.getDshStatus();
      setStatus(st);
    } catch (e) {
      console.error('Failed to get DSH status', e);
    }
  };

  useEffect(() => {
    refreshRuntimeAndProviders();
    refreshStatus();
    const interval = setInterval(refreshStatus, 2000);
    return () => clearInterval(interval);
  }, []);

  const activeProvider = providers.find((p) => p.is_default) || providers[0];

  const handleStartDsh = async () => {
    setLoading(true);
    try {
      await tauriApi.startDsh();
      await refreshStatus();
    } catch (e: any) {
      alert(`Failed to start DSH: ${e.toString()}`);
    } finally {
      setLoading(false);
    }
  };

  const handleStopDsh = async () => {
    setLoading(true);
    try {
      await tauriApi.stopDsh();
      await refreshStatus();
    } catch (e: any) {
      alert(`Failed to stop DSH: ${e.toString()}`);
    } finally {
      setLoading(false);
    }
  };

  const handleRestartDsh = async () => {
    setLoading(true);
    try {
      await tauriApi.restartDsh();
      await refreshStatus();
    } catch (e: any) {
      alert(`Failed to restart DSH: ${e.toString()}`);
    } finally {
      setLoading(false);
    }
  };

  const handleRollback = async () => {
    setLoading(true);
    try {
      await tauriApi.rollbackRuntime();
      await refreshRuntimeAndProviders();
      await handleRestartDsh();
    } catch (e: any) {
      alert(`Rollback failed: ${e.toString()}`);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="app-container">
      <Sidebar
        currentTab={currentTab}
        onTabChange={setCurrentTab}
        collapsed={sidebarCollapsed}
        onToggleCollapse={() => setSidebarCollapsed(!sidebarCollapsed)}
        status={status}
      />

      <main className="main-content">
        <TopBar
          status={status}
          activeProvider={activeProvider}
          onStart={handleStartDsh}
          onStop={handleStopDsh}
          onRestart={handleRestartDsh}
          loading={loading}
        />

        <div className="view-viewport">
          {currentTab === 'chat' && (
            <DshWebView
              status={status}
              runtimeInfo={runtimeInfo}
              activeProvider={activeProvider}
              onStart={handleStartDsh}
              onRestart={handleRestartDsh}
              onRollback={handleRollback}
              onOpenSettings={() => setCurrentTab('logs')}
            />
          )}

          {currentTab === 'providers' && (
            <ProvidersView
              onProvidersChanged={() => {
                refreshRuntimeAndProviders();
              }}
            />
          )}

          {currentTab === 'updates' && (
            <UpdatesView
              runtimeInfo={runtimeInfo}
              onRuntimeUpdated={() => {
                refreshRuntimeAndProviders();
                refreshStatus();
              }}
            />
          )}

          {currentTab === 'sandbox' && <SandboxView />}

          {currentTab === 'snapshots' && <SnapshotsView />}

          {currentTab === 'logs' && <LogsView />}

          {currentTab === 'settings' && <SettingsView runtimeInfo={runtimeInfo} />}
        </div>
      </main>

      {showFirstRun && (
        <FirstRunModal
          runtimeInfo={runtimeInfo}
          onComplete={() => {
            setShowFirstRun(false);
            refreshRuntimeAndProviders();
            handleStartDsh();
          }}
        />
      )}
    </div>
  );
};
