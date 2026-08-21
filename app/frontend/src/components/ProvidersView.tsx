import React, { useState, useEffect } from 'react';
import {
  Plus,
  Trash2,
  Edit2,
  CheckCircle2,
  Zap,
  Lock,
  Search,
  X,
  Eye,
  EyeOff
} from 'lucide-react';
import type { ProviderRecord, ModelRecord, DiscoveredModel } from '../types';
import { tauriApi } from '../services/tauriApi';

interface ProvidersViewProps {
  onProvidersChanged?: () => void;
}

export const ProvidersView: React.FC<ProvidersViewProps> = ({ onProvidersChanged }) => {
  const [providers, setProviders] = useState<ProviderRecord[]>([]);
  const [selectedProvider, setSelectedProvider] = useState<ProviderRecord | null>(null);
  const [models, setModels] = useState<ModelRecord[]>([]);
  const [modalOpen, setModalOpen] = useState(false);

  // Form State
  const [formId, setFormId] = useState('');
  const [formName, setFormName] = useState('');
  const [formType, setFormType] = useState('deepseek');
  const [formBaseUrl, setFormBaseUrl] = useState('https://api.deepseek.com');
  const [formApiKey, setFormApiKey] = useState('');
  const [showApiKey, setShowApiKey] = useState(false);
  const [formIsDefault, setFormIsDefault] = useState(false);
  const [formCompatMode, setFormCompatMode] = useState('');

  // Test & Discovery State
  const [testResult, setTestResult] = useState<{ success: boolean; message: string } | null>(null);
  const [testingConnection, setTestingConnection] = useState(false);
  const [discoveredModels, setDiscoveredModels] = useState<DiscoveredModel[]>([]);
  const [discovering, setDiscovering] = useState(false);

  const loadProviders = async () => {
    try {
      const list = await tauriApi.listProviders();
      setProviders(list);
      if (list.length > 0 && !selectedProvider) {
        setSelectedProvider(list[0]);
        loadModels(list[0].id);
      }
    } catch (e) {
      console.error('Failed to load providers', e);
    }
  };

  const loadModels = async (providerId: string) => {
    try {
      const mList = await tauriApi.listProviderModels(providerId);
      setModels(mList);
    } catch (e) {
      console.error('Failed to load models', e);
    }
  };

  useEffect(() => {
    loadProviders();
  }, []);

  const handleSelectProvider = (prov: ProviderRecord) => {
    setSelectedProvider(prov);
    loadModels(prov.id);
  };

  const handleOpenAddModal = () => {
    setFormId('');
    setFormName('');
    setFormType('deepseek');
    setFormBaseUrl('https://api.deepseek.com');
    setFormApiKey('');
    setFormIsDefault(providers.length === 0);
    setFormCompatMode('');
    setTestResult(null);
    setDiscoveredModels([]);
    setModalOpen(true);
  };

  const handleOpenEditModal = (prov: ProviderRecord) => {
    setFormId(prov.id);
    setFormName(prov.name);
    setFormType(prov.provider_type);
    setFormBaseUrl(prov.base_url);
    setFormApiKey('');
    setFormIsDefault(prov.is_default);
    setFormCompatMode(prov.compat_mode || '');
    setTestResult(null);
    setDiscoveredModels([]);
    setModalOpen(true);
  };

  const handleTypeChange = (type: string) => {
    setFormType(type);
    switch (type) {
      case 'deepseek':
        setFormBaseUrl('https://api.deepseek.com');
        if (!formName || formName === 'OpenAI' || formName === 'Anthropic' || formName === 'Gemini') setFormName('DeepSeek');
        break;
      case 'openai':
        setFormBaseUrl('https://api.openai.com/v1');
        if (!formName || formName === 'DeepSeek') setFormName('OpenAI');
        break;
      case 'anthropic':
        setFormBaseUrl('https://api.anthropic.com');
        if (!formName || formName === 'DeepSeek') setFormName('Anthropic');
        break;
      case 'gemini':
        setFormBaseUrl('https://generativelanguage.googleapis.com/v1beta/openai/');
        if (!formName || formName === 'DeepSeek') setFormName('Google Gemini');
        break;
      case 'openrouter':
        setFormBaseUrl('https://openrouter.ai/api/v1');
        if (!formName || formName === 'DeepSeek') setFormName('OpenRouter');
        break;
      case 'ollama':
        setFormBaseUrl('http://localhost:11434/v1');
        if (!formName || formName === 'DeepSeek') setFormName('Local Ollama');
        break;
      default:
        setFormBaseUrl('https://api.example.com/v1');
        break;
    }
  };

  const handleTestConnection = async () => {
    setTestingConnection(true);
    setTestResult(null);
    try {
      const res = await tauriApi.testProviderConnection(formType, formBaseUrl, formApiKey || undefined);
      setTestResult(res);
    } catch (e: any) {
      setTestResult({ success: false, message: e.toString() });
    } finally {
      setTestingConnection(false);
    }
  };

  const handleDiscoverModels = async () => {
    setDiscovering(true);
    try {
      const mList = await tauriApi.discoverModels(formType, formBaseUrl, formApiKey || undefined);
      setDiscoveredModels(mList);
    } catch (e: any) {
      alert(`Model discovery failed: ${e.toString()}`);
    } finally {
      setDiscovering(false);
    }
  };

  const handleSaveProvider = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!formName.trim() || !formBaseUrl.trim()) return;

    const id = formId || `prov_${Date.now()}`;
    const secretRef = formApiKey.trim() ? `dsh_secret_${id}` : selectedProvider?.secret_ref;

    const record: ProviderRecord = {
      id,
      name: formName.trim(),
      provider_type: formType,
      base_url: formBaseUrl.trim(),
      secret_ref: secretRef,
      is_default: formIsDefault,
      compat_mode: formCompatMode || undefined,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    };

    try {
      await tauriApi.saveProvider(record, formApiKey.trim() || undefined);

      // If discovered models were accepted, save them
      if (discoveredModels.length > 0) {
        const mRecords: ModelRecord[] = discoveredModels.map((dm) => ({
          id: `m_${Date.now()}_${dm.id}`,
          provider_id: id,
          model_id: dm.id,
          display_name: dm.name,
          context_window: dm.context_window,
          max_tokens: dm.max_tokens,
          supports_reasoning: dm.supports_reasoning,
          supports_vision: dm.supports_vision,
          supports_tools: dm.supports_tools,
          discovered_at: new Date().toISOString(),
        }));
        await tauriApi.saveProviderModels(id, mRecords);
      }

      setModalOpen(false);
      await loadProviders();
      if (selectedProvider?.id === id) {
        await loadModels(id);
      }
      onProvidersChanged?.();
    } catch (e: any) {
      alert(`Failed to save provider: ${e.toString()}`);
    }
  };

  const handleDeleteProvider = async (id: string) => {
    if (!confirm('Are you sure you want to delete this provider?')) return;
    try {
      await tauriApi.deleteProvider(id);
      if (selectedProvider?.id === id) {
        setSelectedProvider(null);
        setModels([]);
      }
      await loadProviders();
      onProvidersChanged?.();
    } catch (e: any) {
      alert(`Failed to delete provider: ${e.toString()}`);
    }
  };

  return (
    <div className="panel-view">
      <div className="panel-header" style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <div>
          <h1 className="panel-title">Provider & Model Manager</h1>
          <p className="panel-description">
            Configure AI model providers, endpoints, and credentials securely mapped to the Linux Secret Service.
          </p>
        </div>
        <button className="btn btn-primary" onClick={handleOpenAddModal}>
          <Plus size={16} />
          <span>Add Provider</span>
        </button>
      </div>

      <div style={{ display: 'grid', gridTemplateColumns: '320px 1fr', gap: '24px' }}>
        {/* Left Column: Providers List */}
        <div>
          <div className="card" style={{ padding: '12px' }}>
            <div style={{ fontSize: '12px', fontWeight: 600, color: 'var(--text-muted)', padding: '6px 8px 12px' }}>
              CONFIGURED PROVIDERS ({providers.length})
            </div>

            <div style={{ display: 'flex', flexDirection: 'column', gap: '6px' }}>
              {providers.map((prov) => {
                const isSelected = selectedProvider?.id === prov.id;
                return (
                  <div
                    key={prov.id}
                    onClick={() => handleSelectProvider(prov)}
                    style={{
                      padding: '12px 14px',
                      borderRadius: 'var(--radius-md)',
                      backgroundColor: isSelected ? 'var(--bg-elevated)' : 'var(--bg-tertiary)',
                      border: `1px solid ${isSelected ? 'var(--accent-blue)' : 'var(--border-subtle)'}`,
                      cursor: 'pointer',
                      display: 'flex',
                      alignItems: 'center',
                      justifyContent: 'space-between',
                      transition: 'all 150ms ease',
                    }}
                  >
                    <div>
                      <div style={{ fontWeight: 600, fontSize: '14px', display: 'flex', alignItems: 'center', gap: '6px' }}>
                        {prov.name}
                        {prov.is_default && (
                          <span style={{ fontSize: '10.5px', color: 'var(--accent-cyan)', background: 'rgba(6, 182, 212, 0.15)', padding: '2px 6px', borderRadius: '4px' }}>
                            Default
                          </span>
                        )}
                      </div>
                      <div style={{ fontSize: '12px', color: 'var(--text-secondary)', marginTop: '2px' }}>
                        {prov.provider_type} • {prov.base_url.replace('https://', '')}
                      </div>
                    </div>

                    <div style={{ display: 'flex', gap: '4px' }}>
                      <button
                        className="btn-icon"
                        onClick={(e) => {
                          e.stopPropagation();
                          handleOpenEditModal(prov);
                        }}
                        title="Edit Provider"
                      >
                        <Edit2 size={14} />
                      </button>
                      <button
                        className="btn-icon"
                        onClick={(e) => {
                          e.stopPropagation();
                          handleDeleteProvider(prov.id);
                        }}
                        title="Delete Provider"
                      >
                        <Trash2 size={14} color="var(--status-error)" />
                      </button>
                    </div>
                  </div>
                );
              })}
            </div>
          </div>
        </div>

        {/* Right Column: Provider Details & Model Catalog */}
        <div>
          {selectedProvider ? (
            <div className="card">
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', marginBottom: '20px' }}>
                <div>
                  <h2 style={{ fontSize: '18px', fontWeight: 600 }}>{selectedProvider.name}</h2>
                  <p style={{ fontSize: '13px', color: 'var(--text-secondary)' }}>
                    Endpoint: <code>{selectedProvider.base_url}</code>
                  </p>
                </div>
                <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                  <div style={{ fontSize: '12px', color: 'var(--status-success)', display: 'flex', alignItems: 'center', gap: '4px' }}>
                    <Lock size={13} /> Encrypted Keyring
                  </div>
                  <button className="btn btn-secondary" onClick={() => handleOpenEditModal(selectedProvider)}>
                    <Edit2 size={14} />
                    <span>Configure</span>
                  </button>
                </div>
              </div>

              <div style={{ marginBottom: '16px', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                <h3 style={{ fontSize: '14px', fontWeight: 600, color: 'var(--text-secondary)', textTransform: 'uppercase' }}>
                  Supported Models ({models.length})
                </h3>
              </div>

              {models.length > 0 ? (
                <table className="data-table">
                  <thead>
                    <tr>
                      <th>Model ID</th>
                      <th>Display Name</th>
                      <th>Context Window</th>
                      <th>Capabilities</th>
                    </tr>
                  </thead>
                  <tbody>
                    {models.map((m) => (
                      <tr key={m.id}>
                        <td>
                          <code>{m.model_id}</code>
                        </td>
                        <td>{m.display_name}</td>
                        <td>{m.context_window ? `${(m.context_window / 1024).toFixed(0)}k tokens` : 'Default'}</td>
                        <td>
                          <div style={{ display: 'flex', gap: '6px' }}>
                            {m.supports_reasoning && (
                              <span style={{ fontSize: '11px', background: 'rgba(59, 130, 246, 0.2)', color: 'var(--accent-blue)', padding: '2px 6px', borderRadius: '4px' }}>
                                Reasoning
                              </span>
                            )}
                            {m.supports_vision && (
                              <span style={{ fontSize: '11px', background: 'rgba(16, 185, 129, 0.2)', color: 'var(--status-success)', padding: '2px 6px', borderRadius: '4px' }}>
                                Vision
                              </span>
                            )}
                            {m.supports_tools && (
                              <span style={{ fontSize: '11px', background: 'rgba(245, 158, 11, 0.2)', color: 'var(--status-warning)', padding: '2px 6px', borderRadius: '4px' }}>
                                Tools
                              </span>
                            )}
                          </div>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              ) : (
                <div style={{ padding: '30px', textAlign: 'center', color: 'var(--text-muted)' }}>
                  No models explicitly defined for this provider. DSH will use default catalog or model routing.
                </div>
              )}
            </div>
          ) : (
            <div className="card" style={{ textAlign: 'center', padding: '40px', color: 'var(--text-muted)' }}>
              Select a provider on the left or add a new one.
            </div>
          )}
        </div>
      </div>

      {/* Add / Edit Modal */}
      {modalOpen && (
        <div className="modal-backdrop">
          <div className="modal-content">
            <div className="modal-header">
              <h3 style={{ fontSize: '16px', fontWeight: 600 }}>
                {formId ? 'Configure Provider' : 'Add AI Provider'}
              </h3>
              <button className="btn-icon" onClick={() => setModalOpen(false)}>
                <X size={18} />
              </button>
            </div>

            <form onSubmit={handleSaveProvider}>
              <div className="modal-body">
                <div className="form-group">
                  <label className="form-label">Provider Preset</label>
                  <select
                    className="form-select"
                    value={formType}
                    onChange={(e) => handleTypeChange(e.target.value)}
                  >
                    <option value="deepseek">DeepSeek (Official)</option>
                    <option value="openai">OpenAI</option>
                    <option value="anthropic">Anthropic (Claude)</option>
                    <option value="gemini">Google Gemini</option>
                    <option value="openrouter">OpenRouter</option>
                    <option value="ollama">Local Ollama</option>
                    <option value="custom">Custom OpenAI-Compatible Gateway</option>
                  </select>
                </div>

                <div className="form-group">
                  <label className="form-label">Display Name</label>
                  <input
                    type="text"
                    className="form-input"
                    value={formName}
                    onChange={(e) => setFormName(e.target.value)}
                    placeholder="e.g. DeepSeek Production"
                    required
                  />
                </div>

                <div className="form-group">
                  <label className="form-label">API Base URL</label>
                  <input
                    type="text"
                    className="form-input"
                    value={formBaseUrl}
                    onChange={(e) => setFormBaseUrl(e.target.value)}
                    required
                  />
                </div>

                <div className="form-group">
                  <label className="form-label">API Key / Secret</label>
                  <div style={{ position: 'relative' }}>
                    <input
                      type={showApiKey ? 'text' : 'password'}
                      className="form-input"
                      value={formApiKey}
                      onChange={(e) => setFormApiKey(e.target.value)}
                      placeholder={formId ? '•••••••• (Leave blank to keep existing secret)' : 'sk-...'}
                      style={{ paddingRight: '40px' }}
                    />
                    <button
                      type="button"
                      className="btn-icon"
                      onClick={() => setShowApiKey(!showApiKey)}
                      style={{ position: 'absolute', right: '8px', top: '50%', transform: 'translateY(-50%)' }}
                    >
                      {showApiKey ? <EyeOff size={16} /> : <Eye size={16} />}
                    </button>
                  </div>
                  <div style={{ fontSize: '11.5px', color: 'var(--text-muted)', marginTop: '4px', display: 'flex', alignItems: 'center', gap: '4px' }}>
                    <Lock size={12} color="var(--status-success)" /> Encrypted directly into Linux Secret Service / Keyring.
                  </div>
                </div>

                <div className="form-group">
                  <label style={{ display: 'flex', alignItems: 'center', gap: '8px', cursor: 'pointer', fontSize: '13px' }}>
                    <input
                      type="checkbox"
                      checked={formIsDefault}
                      onChange={(e) => setFormIsDefault(e.target.checked)}
                    />
                    <span>Set as Default Provider for New Sessions</span>
                  </label>
                </div>

                {/* Test Connection & Model Discovery Actions */}
                <div style={{ display: 'flex', gap: '10px', marginTop: '16px' }}>
                  <button
                    type="button"
                    className="btn btn-secondary"
                    onClick={handleTestConnection}
                    disabled={testingConnection}
                  >
                    <Zap size={14} className={testingConnection ? 'animate-spin' : ''} />
                    <span>{testingConnection ? 'Testing...' : 'Test Connection'}</span>
                  </button>

                  <button
                    type="button"
                    className="btn btn-secondary"
                    onClick={handleDiscoverModels}
                    disabled={discovering}
                  >
                    <Search size={14} className={discovering ? 'animate-spin' : ''} />
                    <span>{discovering ? 'Discovering...' : 'Discover Models'}</span>
                  </button>
                </div>

                {testResult && (
                  <div
                    style={{
                      marginTop: '12px',
                      padding: '10px 12px',
                      borderRadius: 'var(--radius-sm)',
                      backgroundColor: testResult.success ? 'rgba(16, 185, 129, 0.15)' : 'rgba(239, 68, 68, 0.15)',
                      color: testResult.success ? 'var(--status-success)' : 'var(--status-error)',
                      fontSize: '12.5px',
                      display: 'flex',
                      alignItems: 'center',
                      gap: '8px',
                    }}
                  >
                    {testResult.success ? <CheckCircle2 size={16} /> : <X size={16} />}
                    <span>{testResult.message}</span>
                  </div>
                )}

                {discoveredModels.length > 0 && (
                  <div style={{ marginTop: '16px' }}>
                    <div style={{ fontSize: '12px', fontWeight: 600, color: 'var(--text-muted)', marginBottom: '6px' }}>
                      DISCOVERED MODELS ({discoveredModels.length})
                    </div>
                    <div style={{ maxHeight: '140px', overflowY: 'auto', background: 'var(--bg-primary)', borderRadius: 'var(--radius-sm)', padding: '6px' }}>
                      {discoveredModels.map((dm) => (
                        <div key={dm.id} style={{ fontSize: '12px', padding: '4px 8px', display: 'flex', justifyContent: 'space-between' }}>
                          <code>{dm.id}</code>
                          <span style={{ color: 'var(--text-muted)' }}>{dm.name}</span>
                        </div>
                      ))}
                    </div>
                  </div>
                )}
              </div>

              <div className="modal-footer">
                <button type="button" className="btn btn-secondary" onClick={() => setModalOpen(false)}>
                  Cancel
                </button>
                <button type="submit" className="btn btn-primary">
                  Save Provider
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
};
