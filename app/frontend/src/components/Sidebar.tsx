import React from 'react';
import {
  MessageSquare,
  Cpu,
  RefreshCw,
  ShieldCheck,
  GitBranch,
  Terminal,
  Settings,
  ChevronLeft,
  ChevronRight,
  Layers
} from 'lucide-react';
import type { DshProcessStatus } from '../types';

export type NavTab =
  | 'chat'
  | 'providers'
  | 'updates'
  | 'sandbox'
  | 'snapshots'
  | 'logs'
  | 'settings';

interface SidebarProps {
  currentTab: NavTab;
  onTabChange: (tab: NavTab) => void;
  collapsed: boolean;
  onToggleCollapse: () => void;
  status: DshProcessStatus;
}

export const Sidebar: React.FC<SidebarProps> = ({
  currentTab,
  onTabChange,
  collapsed,
  onToggleCollapse,
  status,
}) => {
  const isRunning = status.type === 'Running';
  const isStarting = status.type === 'Starting';

  const navItems = [
    { id: 'chat' as NavTab, label: 'DSH Chat / Workspace', icon: MessageSquare },
    { id: 'providers' as NavTab, label: 'Providers & Models', icon: Cpu },
    { id: 'updates' as NavTab, label: 'Runtime & Updates', icon: RefreshCw },
    { id: 'sandbox' as NavTab, label: 'Sandbox & Safe Testing', icon: ShieldCheck },
    { id: 'snapshots' as NavTab, label: 'Snapshots & Git', icon: GitBranch },
    { id: 'logs' as NavTab, label: 'Process Logs', icon: Terminal },
    { id: 'settings' as NavTab, label: 'Settings & Diagnostics', icon: Settings },
  ];

  return (
    <aside className={`sidebar ${collapsed ? 'collapsed' : ''}`}>
      <div className="sidebar-header">
        <div className="brand-badge">
          <div className="brand-logo">
            <Layers size={18} color="#ffffff" />
          </div>
          {!collapsed && (
            <div className="brand-title">
              <span>DeepSeek Harness</span>
              <span>LINUX DESKTOP</span>
            </div>
          )}
        </div>
        <button
          className="btn-icon"
          onClick={onToggleCollapse}
          title={collapsed ? 'Expand sidebar' : 'Collapse sidebar'}
        >
          {collapsed ? <ChevronRight size={16} /> : <ChevronLeft size={16} />}
        </button>
      </div>

      <nav className="nav-section">
        {navItems.map((item) => {
          const Icon = item.icon;
          const isActive = currentTab === item.id;
          return (
            <button
              key={item.id}
              className={`nav-item ${isActive ? 'active' : ''}`}
              onClick={() => onTabChange(item.id)}
              title={collapsed ? item.label : undefined}
            >
              <Icon size={18} />
              {!collapsed && <span>{item.label}</span>}
            </button>
          );
        })}
      </nav>

      <div className="sidebar-footer">
        <div className="runtime-status-pill" title={`DSH Status: ${status.type}`}>
          <div
            className={`status-dot ${
              isRunning ? 'online' : isStarting ? '' : status.type === 'Error' || status.type === 'Crashed' ? 'error' : ''
            }`}
          />
          {!collapsed && (
            <span style={{ overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
              {isRunning
                ? `Port: ${status.data.port}`
                : isStarting
                ? `Starting (${status.data.port})...`
                : status.type}
            </span>
          )}
        </div>
      </div>
    </aside>
  );
};
