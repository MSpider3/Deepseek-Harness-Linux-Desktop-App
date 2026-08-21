import { invoke } from '@tauri-apps/api/core';
import type {
  DiffFileSummary,
  DiscoveredModel,
  DshProcessStatus,
  GitStatusInfo,
  HealthStatus,
  ModelRecord,
  ProcessLogEntry,
  ProviderRecord,
  RuntimeInfo,
  SnapshotRecord,
  TestConnectionResult,
  TestResult,
  UpdateCheckResult,
  UpdateHistoryRecord,
  UpdateResult,
  WorkspaceMetadata,
} from '../types';

const isTauri = () => typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

export const tauriApi = {
  // DSH Process Commands
  async getDshStatus(): Promise<DshProcessStatus> {
    if (!isTauri()) return { type: 'Running', data: { port: 5180, url: 'http://127.0.0.1:5180', pid: 1234 } };
    return invoke<DshProcessStatus>('get_dsh_status');
  },

  async getDshLogs(limit = 200): Promise<ProcessLogEntry[]> {
    if (!isTauri()) return [];
    return invoke<ProcessLogEntry[]>('get_dsh_logs', { limit });
  },

  async startDsh(port?: number): Promise<string> {
    if (!isTauri()) return 'http://127.0.0.1:5180';
    return invoke<string>('start_dsh', { port });
  },

  async stopDsh(): Promise<void> {
    if (!isTauri()) return;
    return invoke<void>('stop_dsh');
  },

  async restartDsh(port?: number): Promise<string> {
    if (!isTauri()) return 'http://127.0.0.1:5180';
    return invoke<string>('restart_dsh', { port });
  },

  async checkDshHealth(port: number): Promise<HealthStatus> {
    if (!isTauri()) return { node_healthy: true, dsh_package_healthy: true, webserver_reachable: true, latency_ms: 12 };
    return invoke<HealthStatus>('check_dsh_health', { port });
  },

  // Runtime Manager Commands
  async getRuntimeInfo(): Promise<RuntimeInfo> {
    if (!isTauri()) {
      return {
        is_installed: true,
        current_version: '0.1.0-rc.8',
        previous_version: '0.1.0-rc.7',
        runtime_root: '~/.local/share/deepseek-harness-linux/runtime',
        node_version: 'v22.22.2',
        versions: [
          { version: '0.1.0-rc.8', path: '/path/0.1.0-rc.8', is_current: true, is_previous: false, installed_at: new Date().toISOString() },
          { version: '0.1.0-rc.7', path: '/path/0.1.0-rc.7', is_current: false, is_previous: true, installed_at: new Date().toISOString() },
        ],
      };
    }
    return invoke<RuntimeInfo>('get_runtime_info');
  },

  async installRuntime(versionOrTag?: string): Promise<string> {
    if (!isTauri()) return '0.1.0-rc.8';
    return invoke<string>('install_runtime', { versionOrTag });
  },

  async activateRuntimeVersion(version: string): Promise<void> {
    if (!isTauri()) return;
    return invoke<void>('activate_runtime_version', { version });
  },

  // Updater Commands
  async checkForUpdates(channel?: string): Promise<UpdateCheckResult> {
    if (!isTauri()) {
      return {
        has_update: false,
        current_version: '0.1.0-rc.8',
        target_version: '0.1.0-rc.8',
        channel: channel || 'stable',
        all_versions: ['0.1.0-rc.6', '0.1.0-rc.7', '0.1.0-rc.8'],
      };
    }
    return invoke<UpdateCheckResult>('check_for_updates', { channel });
  },

  async applyUpdate(targetVersion: string): Promise<UpdateResult> {
    if (!isTauri()) return { success: true, new_version: targetVersion, message: 'Update simulated' };
    return invoke<UpdateResult>('apply_update', { targetVersion });
  },

  async rollbackRuntime(): Promise<UpdateResult> {
    if (!isTauri()) return { success: true, new_version: '0.1.0-rc.7', message: 'Rollback simulated' };
    return invoke<UpdateResult>('rollback_runtime');
  },

  async getUpdateHistory(): Promise<UpdateHistoryRecord[]> {
    if (!isTauri()) return [];
    return invoke<UpdateHistoryRecord[]>('get_update_history');
  },

  // Provider Commands
  async listProviders(): Promise<ProviderRecord[]> {
    if (!isTauri()) {
      return [
        {
          id: '1',
          name: 'DeepSeek',
          provider_type: 'deepseek',
          base_url: 'https://api.deepseek.com',
          secret_ref: 'dsh_secret_1',
          is_default: true,
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString(),
        },
      ];
    }
    return invoke<ProviderRecord[]>('list_providers');
  },

  async saveProvider(provider: ProviderRecord, secretValue?: string): Promise<void> {
    if (!isTauri()) return;
    return invoke<void>('save_provider', { provider, secretValue });
  },

  async deleteProvider(id: string): Promise<void> {
    if (!isTauri()) return;
    return invoke<void>('delete_provider', { id });
  },

  async listProviderModels(providerId: string): Promise<ModelRecord[]> {
    if (!isTauri()) return [];
    return invoke<ModelRecord[]>('list_provider_models', { providerId });
  },

  async saveProviderModels(providerId: string, models: ModelRecord[]): Promise<void> {
    if (!isTauri()) return;
    return invoke<void>('save_provider_models', { providerId, models });
  },

  async testProviderConnection(
    providerType: string,
    baseUrl: string,
    apiKey?: string
  ): Promise<TestConnectionResult> {
    if (!isTauri()) return { success: true, latency_ms: 45, message: 'Connection successful (mock)' };
    return invoke<TestConnectionResult>('test_provider_connection', {
      providerType,
      baseUrl,
      apiKey,
    });
  },

  async discoverModels(
    providerType: string,
    baseUrl: string,
    apiKey?: string
  ): Promise<DiscoveredModel[]> {
    if (!isTauri()) {
      return [
        { id: 'deepseek-chat', name: 'DeepSeek V3', context_window: 65536, max_tokens: 8192, supports_reasoning: false, supports_vision: false, supports_tools: true },
        { id: 'deepseek-reasoner', name: 'DeepSeek R1', context_window: 65536, max_tokens: 8192, supports_reasoning: true, supports_vision: false, supports_tools: true },
      ];
    }
    return invoke<DiscoveredModel[]>('discover_models', {
      providerType,
      baseUrl,
      apiKey,
    });
  },

  async hasProviderSecret(secretRef: string): Promise<boolean> {
    if (!isTauri()) return true;
    return invoke<boolean>('has_provider_secret', { secretRef });
  },

  // Sandbox Commands
  async createSandboxWorkspace(projectPath: string): Promise<WorkspaceMetadata> {
    if (!isTauri()) {
      return {
        workspace_id: 'ws_mock',
        original_path: projectPath,
        workspace_path: '/tmp/ws_mock',
        project_type: 'node',
        default_test_command: 'npm test',
        created_at: new Date().toISOString(),
      };
    }
    return invoke<WorkspaceMetadata>('create_sandbox_workspace', { projectPath });
  },

  async runSandboxTests(workspacePath: string, testCommand: string): Promise<TestResult> {
    if (!isTauri()) {
      return {
        success: true,
        exit_code: 0,
        duration_ms: 1250,
        stdout: 'PASS src/index.test.ts\nAll 12 tests passed.',
        stderr: '',
        command: testCommand,
      };
    }
    return invoke<TestResult>('run_sandbox_tests', { workspacePath, testCommand });
  },

  async getSandboxDiff(originalPath: string, workspacePath: string): Promise<DiffFileSummary[]> {
    if (!isTauri()) return [];
    return invoke<DiffFileSummary[]>('get_sandbox_diff', { originalPath, workspacePath });
  },

  async applySandboxChanges(originalPath: string, workspacePath: string): Promise<number> {
    if (!isTauri()) return 1;
    return invoke<number>('apply_sandbox_changes', { originalPath, workspacePath });
  },

  async discardSandbox(workspacePath: string): Promise<void> {
    if (!isTauri()) return;
    return invoke<void>('discard_sandbox', { workspacePath });
  },

  // Project & Snapshot Commands
  async getGitStatus(projectPath: string): Promise<GitStatusInfo> {
    if (!isTauri()) {
      return {
        is_git_repo: true,
        branch: 'main',
        head_commit: 'a1b2c3d',
        modified_files: [],
        staged_files: [],
        untracked_files: [],
        is_clean: true,
      };
    }
    return invoke<GitStatusInfo>('get_git_status', { projectPath });
  },

  async createSnapshot(
    projectId: string,
    projectPath: string,
    title: string,
    description?: string
  ): Promise<SnapshotRecord> {
    if (!isTauri()) {
      return {
        id: 'snap_1',
        project_id: projectId,
        title,
        description,
        snapshot_path: '/tmp/snap_1.tar.gz',
        created_at: new Date().toISOString(),
      };
    }
    return invoke<SnapshotRecord>('create_snapshot', {
      projectId,
      projectPath,
      title,
      description,
    });
  },

  async listSnapshots(projectId: string): Promise<SnapshotRecord[]> {
    if (!isTauri()) return [];
    return invoke<SnapshotRecord[]>('list_snapshots', { projectId });
  },

  async restoreSnapshot(snapshotId: string, projectPath: string): Promise<void> {
    if (!isTauri()) return;
    return invoke<void>('restore_snapshot', { snapshotId, projectPath });
  },

  async deleteSnapshot(snapshotId: string): Promise<void> {
    if (!isTauri()) return;
    return invoke<void>('delete_snapshot', { snapshotId });
  },

  // Settings & Diagnostics Commands
  async getSetting(key: string): Promise<string | null> {
    if (!isTauri()) return null;
    return invoke<string | null>('get_setting', { key });
  },

  async setSetting(key: string, value: string): Promise<void> {
    if (!isTauri()) return;
    return invoke<void>('set_setting', { key, value });
  },

  async installDesktopLauncher(): Promise<string> {
    if (!isTauri()) return '~/.local/share/applications/ai.deepseek.harness.linux.desktop';
    return invoke<string>('install_desktop_launcher');
  },

  async exportDiagnostics(targetPath: string): Promise<void> {
    if (!isTauri()) return;
    return invoke<void>('export_diagnostics', { targetPath });
  },
};
