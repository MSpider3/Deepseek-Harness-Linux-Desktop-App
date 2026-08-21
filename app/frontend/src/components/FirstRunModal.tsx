import React, { useState, useEffect, useRef } from 'react';
import {
  Sparkles,
  Download,
  CheckCircle2,
  ArrowRight,
  Lock,
  Loader2,
  Clock,
  AlertCircle,
  RotateCcw
} from 'lucide-react';
import { tauriApi } from '../services/tauriApi';
import type { RuntimeInfo } from '../types';

interface FirstRunModalProps {
  runtimeInfo?: RuntimeInfo;
  onComplete: () => void;
}

interface InstallStage {
  id: number;
  label: string;
  detail: string;
}

const INSTALL_STAGES: InstallStage[] = [
  { id: 1, label: 'Environment Setup', detail: 'Creating isolated runtime directory in ~/.local/share' },
  { id: 2, label: 'Package Resolution', detail: 'Querying npm registry for @deepseek-ai/dsh manifest' },
  { id: 3, label: 'Downloading Packages', detail: 'Fetching Cordis kernel, AI agent plugins & dependencies' },
  { id: 4, label: 'Integrity Verification', detail: 'Running executable smoke test and verifying hash signatures' },
  { id: 5, label: 'Atomic Activation', detail: 'Linking active runtime symlink to latest version' },
];

export const FirstRunModal: React.FC<FirstRunModalProps> = ({ runtimeInfo, onComplete }) => {
  const [step, setStep] = useState<1 | 2 | 3>(1);
  const [installing, setInstalling] = useState(false);
  const [progress, setProgress] = useState(0);
  const [currentStageIndex, setCurrentStageIndex] = useState(0);
  const [elapsedSeconds, setElapsedSeconds] = useState(0);
  const [installError, setInstallError] = useState<string | null>(null);

  const timerRef = useRef<any>(null);
  const progressRef = useRef<any>(null);

  // Provider config
  const [providerType, setProviderType] = useState('deepseek');
  const [providerName, setProviderName] = useState('DeepSeek');
  const [baseUrl, setBaseUrl] = useState('https://api.deepseek.com');
  const [apiKey, setApiKey] = useState('');
  const [savingProvider, setSavingProvider] = useState(false);

  useEffect(() => {
    return () => {
      if (timerRef.current) clearInterval(timerRef.current);
      if (progressRef.current) clearInterval(progressRef.current);
    };
  }, []);

  const handleInstallDsh = async () => {
    setInstalling(true);
    setInstallError(null);
    setProgress(5);
    setCurrentStageIndex(0);
    setElapsedSeconds(0);

    const startTime = Date.now();

    // Elapsed timer
    timerRef.current = setInterval(() => {
      setElapsedSeconds(Math.floor((Date.now() - startTime) / 1000));
    }, 1000);

    // Progress animation
    progressRef.current = setInterval(() => {
      setProgress((prev) => {
        if (prev < 20) {
          setCurrentStageIndex(0);
          return prev + 3;
        } else if (prev < 45) {
          setCurrentStageIndex(1);
          return prev + 2;
        } else if (prev < 75) {
          setCurrentStageIndex(2);
          return prev + 1;
        } else if (prev < 90) {
          setCurrentStageIndex(3);
          return prev + 0.5;
        } else if (prev < 95) {
          setCurrentStageIndex(4);
          return prev + 0.2;
        }
        return prev;
      });
    }, 400);

    try {
      await tauriApi.installRuntime('latest');
      if (timerRef.current) clearInterval(timerRef.current);
      if (progressRef.current) clearInterval(progressRef.current);

      setProgress(100);
      setCurrentStageIndex(INSTALL_STAGES.length - 1);

      setTimeout(() => {
        setStep(2);
      }, 700);
    } catch (e: any) {
      if (timerRef.current) clearInterval(timerRef.current);
      if (progressRef.current) clearInterval(progressRef.current);
      setInstallError(e.toString());
    } finally {
      setInstalling(false);
    }
  };

  const handleSaveProviderAndFinish = async (e: React.FormEvent) => {
    e.preventDefault();
    setSavingProvider(true);
    try {
      const id = `prov_${Date.now()}`;
      await tauriApi.saveProvider(
        {
          id,
          name: providerName,
          provider_type: providerType,
          base_url: baseUrl,
          secret_ref: apiKey.trim() ? `dsh_secret_${id}` : undefined,
          is_default: true,
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString(),
        },
        apiKey.trim() || undefined
      );
      setStep(3);
    } catch (e: any) {
      alert(`Failed to save provider: ${e.toString()}`);
    } finally {
      setSavingProvider(false);
    }
  };

  return (
    <div className="modal-backdrop">
      <div className="modal-content" style={{ maxWidth: '520px' }}>
        <div className="modal-header">
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <Sparkles size={18} color="var(--accent-cyan)" />
            <h3 style={{ fontSize: '16px', fontWeight: 600 }}>Welcome to DeepSeek Harness Linux</h3>
          </div>
          <span style={{ fontSize: '12px', color: 'var(--text-muted)' }}>Step {step} of 3</span>
        </div>

        <div className="modal-body">
          {step === 1 && (
            <div style={{ padding: '8px 0' }}>
              {!installing && !installError && progress === 0 && (
                <div style={{ textAlign: 'center', padding: '12px 0 20px' }}>
                  <div
                    style={{
                      width: '56px',
                      height: '56px',
                      borderRadius: 'var(--radius-lg)',
                      background: 'var(--accent-gradient)',
                      display: 'flex',
                      alignItems: 'center',
                      justifyContent: 'center',
                      margin: '0 auto 16px',
                      boxShadow: 'var(--accent-glow)',
                    }}
                  >
                    <Download size={26} color="#fff" />
                  </div>

                  <h4 style={{ fontSize: '18px', fontWeight: 600, marginBottom: '8px' }}>
                    Install Isolated DSH Runtime
                  </h4>
                  <p style={{ color: 'var(--text-secondary)', fontSize: '13.5px', marginBottom: '24px', lineHeight: '1.5' }}>
                    DeepSeek Harness Linux maintains its own isolated Node runtime directory for <code>@deepseek-ai/dsh</code> ({runtimeInfo?.current_version || 'official upstream package'}) with atomic update and rollback safety.
                  </p>

                  <button
                    className="btn btn-primary"
                    onClick={handleInstallDsh}
                    style={{ padding: '10px 24px', fontSize: '14px' }}
                  >
                    <Download size={16} />
                    <span>Install Official DSH Runtime</span>
                  </button>
                </div>
              )}

              {(installing || progress > 0) && !installError && (
                <div style={{ padding: '8px 0' }}>
                  {/* Header info with elapsed time and percentage */}
                  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '10px' }}>
                    <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                      <Loader2 size={16} color="var(--accent-cyan)" className="animate-spin" />
                      <span style={{ fontSize: '14px', fontWeight: 600 }}>
                        {progress >= 100 ? 'Installation Complete!' : 'Installing Official Runtime...'}
                      </span>
                    </div>
                    <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
                      <span style={{ fontSize: '12px', color: 'var(--text-muted)', display: 'flex', alignItems: 'center', gap: '4px' }}>
                        <Clock size={12} /> {elapsedSeconds}s elapsed
                      </span>
                      <span style={{ fontSize: '14px', fontWeight: 700, color: 'var(--accent-cyan)' }}>
                        {Math.round(progress)}%
                      </span>
                    </div>
                  </div>

                  {/* Visual Progress Bar Track */}
                  <div
                    style={{
                      width: '100%',
                      height: '10px',
                      backgroundColor: 'var(--bg-tertiary)',
                      borderRadius: 'var(--radius-full)',
                      overflow: 'hidden',
                      marginBottom: '16px',
                      border: '1px solid var(--border-subtle)',
                      position: 'relative'
                    }}
                  >
                    <div
                      style={{
                        height: '100%',
                        width: `${Math.min(100, Math.max(5, progress))}%`,
                        background: 'linear-gradient(90deg, var(--accent-blue) 0%, var(--accent-cyan) 100%)',
                        borderRadius: 'var(--radius-full)',
                        transition: 'width 300ms ease-out',
                        boxShadow: '0 0 12px rgba(6, 182, 212, 0.6)'
                      }}
                    />
                  </div>

                  {/* Installation Stages Checklist */}
                  <div
                    style={{
                      background: 'var(--bg-secondary)',
                      borderRadius: 'var(--radius-md)',
                      border: '1px solid var(--border-subtle)',
                      padding: '12px 14px',
                      display: 'flex',
                      flexDirection: 'column',
                      gap: '10px',
                    }}
                  >
                    {INSTALL_STAGES.map((stage, idx) => {
                      const isCompleted = idx < currentStageIndex || progress >= 100;
                      const isCurrent = idx === currentStageIndex && progress < 100;
                      return (
                        <div
                          key={stage.id}
                          style={{
                            display: 'flex',
                            alignItems: 'flex-start',
                            gap: '10px',
                            opacity: isCompleted ? 0.7 : isCurrent ? 1 : 0.4,
                            transition: 'opacity var(--transition-fast)',
                          }}
                        >
                          <div style={{ marginTop: '2px', flexShrink: 0 }}>
                            {isCompleted ? (
                              <CheckCircle2 size={16} color="var(--status-success)" />
                            ) : isCurrent ? (
                              <Loader2 size={16} color="var(--accent-cyan)" className="animate-spin" />
                            ) : (
                              <div
                                style={{
                                  width: '16px',
                                  height: '16px',
                                  borderRadius: '50%',
                                  border: '1.5px solid var(--border-medium)',
                                }}
                              />
                            )}
                          </div>
                          <div style={{ flex: 1 }}>
                            <div style={{ fontSize: '13px', fontWeight: isCurrent ? 600 : 500, color: isCurrent ? 'var(--text-primary)' : 'var(--text-secondary)' }}>
                              {stage.label}
                            </div>
                            <div style={{ fontSize: '11.5px', color: 'var(--text-muted)', marginTop: '1px' }}>
                              {stage.detail}
                            </div>
                          </div>
                        </div>
                      );
                    })}
                  </div>
                </div>
              )}

              {installError && (
                <div style={{ textAlign: 'center', padding: '16px 0' }}>
                  <div
                    style={{
                      width: '48px',
                      height: '48px',
                      borderRadius: '50%',
                      background: 'rgba(239, 68, 68, 0.15)',
                      display: 'flex',
                      alignItems: 'center',
                      justifyContent: 'center',
                      margin: '0 auto 12px',
                    }}
                  >
                    <AlertCircle size={24} color="var(--status-error)" />
                  </div>
                  <h4 style={{ fontSize: '16px', fontWeight: 600, color: 'var(--status-error)', marginBottom: '6px' }}>
                    Installation Encountered an Issue
                  </h4>
                  <div
                    className="code-block"
                    style={{ maxHeight: '120px', overflowY: 'auto', textAlign: 'left', marginBottom: '16px', fontSize: '12px' }}
                  >
                    {installError}
                  </div>
                  <button className="btn btn-primary" onClick={handleInstallDsh}>
                    <RotateCcw size={14} />
                    <span>Retry Installation</span>
                  </button>
                </div>
              )}
            </div>
          )}

          {step === 2 && (
            <form onSubmit={handleSaveProviderAndFinish}>
              <h4 style={{ fontSize: '16px', fontWeight: 600, marginBottom: '6px' }}>
                Configure Your First AI Provider
              </h4>
              <p style={{ color: 'var(--text-secondary)', fontSize: '13px', marginBottom: '18px' }}>
                Your API credentials will be encrypted directly into the Linux Secret Service / Keyring.
              </p>

              <div className="form-group">
                <label className="form-label">Provider</label>
                <select
                  className="form-select"
                  value={providerType}
                  onChange={(e) => {
                    setProviderType(e.target.value);
                    if (e.target.value === 'deepseek') {
                      setProviderName('DeepSeek');
                      setBaseUrl('https://api.deepseek.com');
                    } else if (e.target.value === 'openai') {
                      setProviderName('OpenAI');
                      setBaseUrl('https://api.openai.com/v1');
                    } else if (e.target.value === 'anthropic') {
                      setProviderName('Anthropic');
                      setBaseUrl('https://api.anthropic.com');
                    } else if (e.target.value === 'ollama') {
                      setProviderName('Local Ollama');
                      setBaseUrl('http://localhost:11434/v1');
                    }
                  }}
                >
                  <option value="deepseek">DeepSeek (Official)</option>
                  <option value="openai">OpenAI</option>
                  <option value="anthropic">Anthropic (Claude)</option>
                  <option value="ollama">Local Ollama</option>
                </select>
              </div>

              <div className="form-group">
                <label className="form-label">API Key</label>
                <input
                  type="password"
                  className="form-input"
                  value={apiKey}
                  onChange={(e) => setApiKey(e.target.value)}
                  placeholder={providerType === 'ollama' ? 'None needed for local Ollama' : 'sk-...'}
                />
                <div style={{ fontSize: '11px', color: 'var(--text-muted)', marginTop: '4px', display: 'flex', alignItems: 'center', gap: '4px' }}>
                  <Lock size={12} color="var(--status-success)" /> Encrypted directly into Linux Keyring.
                </div>
              </div>

              <div style={{ display: 'flex', justifyContent: 'flex-end', marginTop: '20px' }}>
                <button type="submit" className="btn btn-primary" disabled={savingProvider}>
                  <span>{savingProvider ? 'Saving...' : 'Save & Continue'}</span>
                  <ArrowRight size={14} />
                </button>
              </div>
            </form>
          )}

          {step === 3 && (
            <div style={{ textAlign: 'center', padding: '20px 0' }}>
              <div
                style={{
                  width: '56px',
                  height: '56px',
                  borderRadius: '50%',
                  background: 'rgba(16, 185, 129, 0.15)',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  margin: '0 auto 16px',
                }}
              >
                <CheckCircle2 size={32} color="var(--status-success)" />
              </div>

              <h4 style={{ fontSize: '18px', fontWeight: 600, marginBottom: '8px' }}>Setup Complete!</h4>
              <p style={{ color: 'var(--text-secondary)', fontSize: '13.5px', marginBottom: '24px' }}>
                DeepSeek Harness Linux is ready. Launch the official Web workspace or explore providers and sandbox features.
              </p>

              <button className="btn btn-primary" onClick={onComplete} style={{ padding: '10px 28px', fontSize: '14px' }}>
                <span>Launch Desktop Application</span>
              </button>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
