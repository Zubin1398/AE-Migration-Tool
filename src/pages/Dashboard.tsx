import React from 'react';
import { Button, Card } from '@fluentui/react-components';
import {
  ArrowSyncFilled,
  ChevronRightFilled
} from '@fluentui/react-icons';
import { invoke } from '@tauri-apps/api/core';
import { Installation } from '../types';
import { InstallationPathPicker } from '../components/InstallationPathPicker';

interface DashboardProps {
  installations: Installation[];
  onVersionSelect: (sourceId: string, targetId: string) => void;
  onRefresh: () => void;
  onInstallationAdded: (installation: Installation) => void;
}

export const Dashboard: React.FC<DashboardProps> = ({ 
  installations, 
  onVersionSelect, 
  onRefresh,
  onInstallationAdded,
}) => {
  const [selectedSource, setSelectedSource] = React.useState<string | null>(null);
  const [selectedTarget, setSelectedTarget] = React.useState<string | null>(null);
  const [sourcePath, setSourcePath] = React.useState('');
  const [targetPath, setTargetPath] = React.useState('');
  const [addingRole, setAddingRole] = React.useState<'source' | 'target' | null>(null);

  const handleAddManual = async (role: 'source' | 'target') => {
    const path = (role === 'source' ? sourcePath : targetPath).trim();
    if (!path) return;
    setAddingRole(role);
    try {
      const normalize = (value: string) => value.replace(/[\\/]+$/, '').toLowerCase();
      const existing = installations.find(item => item.is_complete && normalize(item.install_path) === normalize(path));
      const installation = existing ?? await invoke<Installation>('add_manual_installation', { path });
      if (!existing) onInstallationAdded(installation);
      if (role === 'source') {
        setSelectedSource(installation.id);
        setSourcePath('');
      } else {
        setSelectedTarget(installation.id);
        setTargetPath('');
      }
    } catch (e) {
      alert('Ошибка добавления: ' + e);
    } finally {
      setAddingRole(null);
    }
  };

  const canProceed = selectedSource && selectedTarget && selectedSource !== selectedTarget;

  return (
    <div className="main-content">
      <div className="page-header">
        <h1 className="page-title">Миграция</h1>
        <p className="page-subtitle">
          Укажите старую и новую установки After Effects.
        </p>
      </div>

      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '24px' }}>
        <Card className="card">
          <div className="card-header">
            <h3 className="card-title">Источник (Source)</h3>
            <Button 
              appearance="secondary" 
              icon={<ArrowSyncFilled />}
              onClick={onRefresh}
              aria-label="Обновить список"
            />
          </div>
          
          <InstallationPathPicker
            value={sourcePath}
            onChange={setSourcePath}
            onAdd={() => handleAddManual('source')}
            addLabel="Добавить источник"
            adding={addingRole === 'source'}
            placeholder="Вставьте путь к старой версии AE"
          />

          <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
            {installations.filter(i => i.is_complete).map(install => (
              <div
                key={install.id}
                className={`installation-card ${selectedSource === install.id ? 'selected' : ''}`}
                onClick={() => setSelectedSource(install.id)}
              >
                <div className="installation-info">
                  <div className="installation-name">
                    {install.install_path.split('\\').pop() || install.id}
                  </div>
                  <div className="installation-path">
                    {install.install_path} • v{install.version.display_name}
                  </div>
                </div>
                {selectedSource === install.id && (
                  <ChevronRightFilled style={{ color: '#0078d4' }} />
                )}
              </div>
            ))}
            {installations.filter(i => i.is_complete).length === 0 && (
              <div className="empty-state">
                <div className="empty-state-icon">🔍</div>
                <p>Установки не найдены</p>
                <p style={{ fontSize: '12px' }}>Нажмите "Обновить" или добавьте вручную</p>
              </div>
            )}
          </div>
        </Card>

        <Card className="card">
          <div className="card-header">
            <h3 className="card-title">Цель (Target)</h3>
          </div>

          <InstallationPathPicker
            value={targetPath}
            onChange={setTargetPath}
            onAdd={() => handleAddManual('target')}
            addLabel="Добавить цель"
            adding={addingRole === 'target'}
            placeholder="Вставьте путь к новой версии AE"
          />
          
          <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
            {installations.filter(i => i.is_complete).map(install => (
              <div
                key={install.id}
                className={`installation-card ${selectedTarget === install.id ? 'selected' : ''}`}
                onClick={() => setSelectedTarget(install.id)}
              >
                <div className="installation-info">
                  <div className="installation-name">
                    {install.install_path.split('\\').pop() || install.id}
                  </div>
                  <div className="installation-path">
                    {install.install_path} • v{install.version.display_name}
                  </div>
                </div>
                {selectedTarget === install.id && (
                  <ChevronRightFilled style={{ color: '#0078d4' }} />
                )}
              </div>
            ))}
            {installations.filter(i => i.is_complete).length === 0 && (
              <div className="empty-state">
                <div className="empty-state-icon">🎯</div>
                <p>Выберите целевую версию</p>
              </div>
            )}
          </div>
        </Card>
      </div>

      <div className="btn-group" style={{ justifyContent: 'center' }}>
        <Button 
          appearance="primary" 
          disabled={!canProceed}
          onClick={() => onVersionSelect(selectedSource!, selectedTarget!)}
        >
          Продолжить <ChevronRightFilled />
        </Button>
      </div>

    </div>
  );
};
