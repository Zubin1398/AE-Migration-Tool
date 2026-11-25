import React from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Button, Card } from '@fluentui/react-components';
import { ArrowSyncFilled, CheckmarkCircleFilled, WarningFilled } from '@fluentui/react-icons';
import { Installation } from '../types';
import { InstallationPathPicker } from '../components/InstallationPathPicker';

interface VersionSelectionProps {
  installations: Installation[];
  onRefresh: () => void;
  onInstallationAdded: (installation: Installation) => void;
}

export const VersionSelection: React.FC<VersionSelectionProps> = ({
  installations,
  onRefresh,
  onInstallationAdded,
}) => {
  const [manualPath, setManualPath] = React.useState('');
  const [adding, setAdding] = React.useState(false);

  const handleAddInstallation = async () => {
    const path = manualPath.trim();
    if (!path) return;
    setAdding(true);
    try {
      const normalize = (value: string) => value.replace(/[\\/]+$/, '').toLowerCase();
      const existing = installations.find(item => normalize(item.install_path) === normalize(path));
      if (existing) {
        alert('Эта установка уже есть в списке.');
        return;
      }
      const installation = await invoke<Installation>('add_manual_installation', { path });
      onInstallationAdded(installation);
      setManualPath('');
    } catch (error) {
      alert(`Не удалось добавить установку: ${error}`);
    } finally {
      setAdding(false);
    }
  };

  return (
    <div className="main-content">
      <div className="page-header version-page-header">
        <div>
          <h1 className="page-title">Установки After Effects</h1>
          <p className="page-subtitle">Управляйте путями к версиям, доступным для миграции.</p>
        </div>
        <Button appearance="secondary" icon={<ArrowSyncFilled />} onClick={onRefresh}>
          Обновить поиск
        </Button>
      </div>

      <Card className="card">
        <div className="card-header">
          <h2 className="card-title">Добавить установку</h2>
        </div>
        <InstallationPathPicker
          value={manualPath}
          onChange={setManualPath}
          onAdd={handleAddInstallation}
          addLabel="Добавить версию"
          adding={adding}
        />
      </Card>

      <div className="section-heading-row">
        <h2 className="section-title">Найденные папки</h2>
        <span className="installation-total">
          {installations.filter(item => item.is_complete).length} из {installations.length} готовы
        </span>
      </div>

      {installations.length === 0 ? (
        <Card className="card">
          <div className="empty-state">
            <div className="empty-state-icon"><WarningFilled /></div>
            <p>Установки не найдены</p>
            <p>Выберите папку After Effects вручную или обновите поиск.</p>
          </div>
        </Card>
      ) : (
        <div className="installation-list">
          {installations.map(installation => (
            <Card key={installation.id} className="installation-row">
              <span className={`installation-state ${installation.is_complete ? 'ready' : 'incomplete'}`}>
                {installation.is_complete ? <CheckmarkCircleFilled /> : <WarningFilled />}
              </span>
              <div className="installation-info">
                <div className="installation-name">
                  {installation.version.display_name || installation.install_path.split(/[\\/]/).pop()}
                  {!installation.is_complete && <span className="installation-warning">Неполная установка</span>}
                </div>
                <div className="installation-path" title={installation.install_path}>
                  {installation.install_path}
                </div>
              </div>
            </Card>
          ))}
        </div>
      )}
    </div>
  );
};
