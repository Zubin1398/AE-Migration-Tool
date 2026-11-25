import React from 'react';
import { Card, Button } from '@fluentui/react-components';
import { ArrowSyncFilled } from '@fluentui/react-icons';
import { invoke } from '@tauri-apps/api/core';
import { confirm } from '@tauri-apps/plugin-dialog';
import { BackupRecord, MigrationHistoryEntry } from '../types';

interface HistoryProps {
  installationPaths: string[];
}

export const History: React.FC<HistoryProps> = ({ installationPaths }) => {
  const [history, setHistory] = React.useState<MigrationHistoryEntry[]>([]);
  const [loading, setLoading] = React.useState(false);
  const [error, setError] = React.useState<string | null>(null);
  const [rollbackAvailable, setRollbackAvailable] = React.useState<Set<string>>(new Set());
  const [rollbackCompleted, setRollbackCompleted] = React.useState<Set<string>>(new Set());
  const [rollingBackId, setRollingBackId] = React.useState<string | null>(null);
  const [rollbackFeedback, setRollbackFeedback] = React.useState<{ id: string; type: 'success' | 'error'; text: string } | null>(null);

  const loadHistory = async () => {
    setLoading(true);
    setError(null);
    try {
      const records = await invoke<MigrationHistoryEntry[]>('get_migration_history', { installationPaths });
      setHistory(records.sort((a, b) => b.timestamp - a.timestamp));
      const rollbackStates = await Promise.all(records.map(async entry => {
        try {
          const backups = await invoke<BackupRecord[]>('list_backups_cmd', { backupPath: entry.backup_root });
          const migrationBackups = backups.filter(record => record.migration_id === entry.migration_id);
          return {
            id: entry.migration_id,
            available: migrationBackups.length > 0 && !migrationBackups.every(record => record.rolled_back),
            completed: migrationBackups.length > 0 && migrationBackups.every(record => record.rolled_back),
          };
        } catch {
          return { id: entry.migration_id, available: false, completed: false };
        }
      }));
      setRollbackAvailable(new Set(rollbackStates.filter(state => state.available).map(state => state.id)));
      setRollbackCompleted(new Set(rollbackStates.filter(state => state.completed).map(state => state.id)));
    } catch (e) {
      console.error('Failed to load migration history:', e);
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  React.useEffect(() => {
    loadHistory();
  }, [installationPaths]);

  const handleRollback = async (migrationId: string) => {
    setRollbackFeedback(null);
    setRollingBackId(migrationId);
    try {
      const confirmed = await confirm(
        'Будут восстановлены файлы из резервной копии. Продолжить?',
        { title: 'Откат миграции', kind: 'warning' },
      );
      if (!confirmed) return;

      const entry = history.find(item => item.migration_id === migrationId);
      if (!entry) throw new Error('Запись миграции не найдена');
      await invoke<void>('rollback_migration', { planId: migrationId, backupPath: entry.backup_root });
      setRollbackFeedback({ id: migrationId, type: 'success', text: 'Откат выполнен. Файлы восстановлены из резервной копии.' });
      await loadHistory();
    } catch (e) {
      setRollbackFeedback({ id: migrationId, type: 'error', text: `Не удалось выполнить откат: ${String(e)}` });
    } finally {
      setRollingBackId(null);
    }
  };

  return (
    <div className="main-content">
      <div className="page-header" style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <div>
          <h1 className="page-title">История миграций</h1>
          <p className="page-subtitle">Резервные копии и выполненные миграции</p>
        </div>
        <Button appearance="secondary" icon={<ArrowSyncFilled />} onClick={loadHistory} disabled={loading}>
          {loading ? 'Обновление...' : 'Обновить'}
        </Button>
      </div>

      {error && <Card className="card"><p role="alert">Не удалось загрузить историю: {error}</p></Card>}

      {loading && history.length === 0 ? (
        <Card className="card"><p>Загружаем историю…</p></Card>
      ) : !error && history.length === 0 ? (
        <Card className="card">
          <div className="empty-state">
            <div className="empty-state-icon">📋</div>
            <p>История пуста</p>
            <p style={{ fontSize: '14px', color: '#616161' }}>
              Выполненные миграции и их бэкапы будут отображаться здесь
            </p>
          </div>
        </Card>
      ) : (
        <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
          {history.map(entry => {
            const rolledBack = rollbackCompleted.has(entry.migration_id) ||
              (rollbackFeedback?.id === entry.migration_id && rollbackFeedback.type === 'success');
            return (
            <Card key={entry.migration_id} className="card">
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', flexWrap: 'wrap', gap: '16px' }}>
                <div>
                  <div style={{ fontWeight: 600, fontSize: '14px', marginBottom: '4px' }}>
                    {entry.source_version.display_name} → {entry.target_version.display_name}
                  </div>
                  <div style={{ fontSize: '12px', color: '#616161' }}>
                    Завершено: {entry.completed_steps} · Пропущено: {entry.skipped_steps} · Ошибок: {entry.failed_steps}
                  </div>
                  <div style={{ fontSize: '12px', color: '#616161' }}>
                    Дата: {new Date(entry.timestamp * 1000).toLocaleString()}
                  </div>
                  {entry.errors.length > 0 && <div style={{ fontSize: '12px', color: '#d13438' }}>{entry.errors.join('; ')}</div>}
                  {rollbackFeedback?.id === entry.migration_id && (
                    <div
                      role={rollbackFeedback.type === 'error' ? 'alert' : 'status'}
                      style={{ marginTop: '8px', color: rollbackFeedback.type === 'error' ? '#d13438' : '#107c10' }}
                    >
                      {rollbackFeedback.text}
                    </div>
                  )}
                </div>
                <div style={{ display: 'flex', gap: '8px' }}>
                  <Button
                    appearance="secondary"
                    onClick={() => handleRollback(entry.migration_id)}
                    disabled={!rollbackAvailable.has(entry.migration_id) || rollingBackId === entry.migration_id || rolledBack}
                    title={rollbackAvailable.has(entry.migration_id) ? 'Восстановить файлы из резервной копии' : 'Для этой миграции нет данных для отката'}
                  >
                    {rollingBackId === entry.migration_id ? 'Выполняется...' : rolledBack ? 'Откат выполнен' : rollbackAvailable.has(entry.migration_id) ? 'Откатить' : 'Бэкапа нет'}
                  </Button>
                </div>
              </div>
            </Card>
            );
          })}
        </div>
      )}
    </div>
  );
};
