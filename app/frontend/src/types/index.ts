export type DshProcessStatus =
  | { type: 'Stopped' }
  | { type: 'Starting'; data: { port: number } }
  | { type: 'Running'; data: { port: number; url: string; pid: number } }
  | { type: 'Error'; data: { message: string } }
  | { type: 'Crashed'; data: { message: string; restart_count: number } };

export interface RuntimeVersionEntry {
  version: string;
  path: string;
  is_current: boolean;
  is_previous: boolean;
  installed_at: string;
}

export interface RuntimeInfo {
  is_installed: boolean;
  current_version?: string;
  previous_version?: string;
  runtime_root: string;
  executable_path?: string;
  node_version?: string;
  versions: RuntimeVersionEntry[];
}

export interface ProviderRecord {
  id: string;
  name: string;
  provider_type: 'deepseek' | 'openai' | 'anthropic' | 'gemini' | 'openrouter' | 'ollama' | 'custom' | string;
  base_url: string;
  secret_ref?: string;
  is_default: boolean;
  compat_mode?: string;
  created_at: string;
  updated_at: string;
}

export interface ModelRecord {
  id: string;
  provider_id: string;
  model_id: string;
  display_name: string;
  context_window?: number;
  max_tokens?: number;
  supports_reasoning: boolean;
  supports_vision: boolean;
  supports_tools: boolean;
  discovered_at: string;
}

export interface DiscoveredModel {
  id: string;
  name: string;
  context_window?: number;
  max_tokens?: number;
  supports_reasoning: boolean;
  supports_vision: boolean;
  supports_tools: boolean;
}

export interface TestConnectionResult {
  success: boolean;
  latency_ms?: number;
  message: string;
}

export interface UpdateCheckResult {
  has_update: boolean;
  current_version?: string;
  target_version: string;
  channel: 'stable' | 'releasecandidate' | 'development' | string;
  all_versions: string[];
}

export interface UpdateResult {
  success: boolean;
  previous_version?: string;
  new_version: string;
  message: string;
}

export interface UpdateHistoryRecord {
  id: number;
  from_version?: string;
  to_version: string;
  status: string;
  error_message?: string;
  timestamp: string;
}

export interface WorkspaceMetadata {
  workspace_id: string;
  original_path: string;
  workspace_path: string;
  project_type: 'node' | 'python' | 'rust' | 'general';
  default_test_command: string;
  created_at: string;
}

export interface TestResult {
  success: boolean;
  exit_code?: number;
  duration_ms: number;
  stdout: string;
  stderr: string;
  command: string;
}

export interface DiffFileSummary {
  file_path: string;
  status: 'modified' | 'added' | 'deleted';
  additions: number;
  deletions: number;
  unified_diff: string;
}

export interface GitStatusInfo {
  is_git_repo: boolean;
  branch?: string;
  head_commit?: string;
  modified_files: string[];
  staged_files: string[];
  untracked_files: string[];
  is_clean: boolean;
}

export interface SnapshotRecord {
  id: string;
  project_id: string;
  title: string;
  description?: string;
  snapshot_path: string;
  git_commit?: string;
  created_at: string;
}

export interface ProcessLogEntry {
  timestamp: string;
  stream: 'stdout' | 'stderr' | 'system' | string;
  message: string;
}

export interface HealthStatus {
  node_healthy: boolean;
  dsh_package_healthy: boolean;
  webserver_reachable: boolean;
  endpoint_url?: string;
  latency_ms?: number;
  error?: string;
}
