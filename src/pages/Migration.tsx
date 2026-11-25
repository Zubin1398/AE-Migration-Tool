import React from 'react';
import { invoke } from '@tauri-apps/api/core';
import { 
  Button, 
  Card, 
  ProgressBar, 
  Input,
  Checkbox,
  Radio,
  RadioGroup,
  Label
} from '@fluentui/react-components';
import { 
  PlayFilled, 
  ArrowLeftFilled,
  SearchFilled,
  CheckmarkFilled,
  WarningFilled,
  DismissFilled,
  InfoFilled,
  FolderFilled,
  DocumentFilled
} from '@fluentui/react-icons';
import { Category, MigrationPlan } from '../types';

interface MigrationProps {
  sourceVersion: any;
  targetVersion: any;
  plan: MigrationPlan | null;
  onScanComplete: (plan: MigrationPlan) => void;
  onMigrationComplete: (result: any) => void;
  onBack: () => void;
}

const CATEGORIES: Category[] = [
  'Plugins', 'Scripts', 'ScriptUIPanels', 'CEPExtensions', 
  'Presets', 'Workspaces', 'KeyboardShortcuts', 'Preferences', 'SharedResource', 'Unknown'
];

const categoryLabels: Record<Category, string> = {
  Plugins: 'Плагины (.aex, .dll)',
  Scripts: 'Скрипты (.jsx, .js)',
  ScriptUIPanels: 'UI Панели (.jsx)',
  CEPExtensions: 'CEP Расширения (.zxp)',
  Presets: 'Пресеты (.ffx, .prfpset)',
  Workspaces: 'Рабочие пространства (.xml)',
  KeyboardShortcuts: 'Горячие клавиши (.txt)',
  Preferences: 'Настройки',
  SharedResource: 'Общие ресурсы',
  Unknown: 'Неизвестные файлы',
};

const categoryHints: Record<Category, string> = {
  Plugins: 'Нативные плагины After Effects',
  Scripts: 'JSX-скрипты для автоматизации',
  ScriptUIPanels: 'Панели интерфейса пользователя',
  CEPExtensions: 'HTML/JS расширения (CEP)',
  Presets: 'Эффекты и анимационные пресеты',
  Workspaces: 'Раскладки панелей и окон',
  KeyboardShortcuts: 'Назначения клавиш',
  Preferences: 'Файлы настроек и преференсов',
  SharedResource: 'Шаблоны, медиа и общие файлы',
  Unknown: 'Файлы без определенной категории',
};

const categoryIcons: Record<Category, React.ReactNode> = {
  Plugins: '🔌',
  Scripts: '📜',
  ScriptUIPanels: '🖼️',
  CEPExtensions: '🧩',
  Presets: '✨',
  Workspaces: '🖥️',
  KeyboardShortcuts: '⌨️',
  Preferences: '⚙️',
  SharedResource: '📦',
  Unknown: '❓',
};

export const Migration: React.FC<MigrationProps> = ({ 
  sourceVersion, 
  targetVersion, 
  plan: initialPlan,
  onScanComplete,
  onMigrationComplete,
  onBack 
}) => {
  const [selectedCategories, setSelectedCategories] = React.useState<Category[]>(CATEGORIES);
  const [isScanning, setIsScanning] = React.useState(false);
  const [isMigrating, setIsMigrating] = React.useState(false);
  const [scanProgress, setScanProgress] = React.useState(0);
  const [migrationProgress, setMigrationProgress] = React.useState(0);
  const [plan, setPlan] = React.useState<MigrationPlan | null>(initialPlan);
  const [result, setResult] = React.useState<any>(null);
  const [logs, setLogs] = React.useState<string[]>([]);
  const [backupPath, setBackupPath] = React.useState('');
  const [conflictStrategy, setConflictStrategy] = React.useState<'keep_target' | 'backup_replace'>('keep_target');
  const [backupKeptTarget, setBackupKeptTarget] = React.useState(false);

  // Sync plan from props (when coming from ScanAnalysis)
  React.useEffect(() => {
    if (initialPlan) {
      setPlan(initialPlan);
    }
  }, [initialPlan]);

  // Use plan from props if available, otherwise local state
  const currentPlan = plan || initialPlan;

  const addLog = (message: string, level: 'info' | 'success' | 'warning' | 'error' = 'info') => {
    const timestamp = new Date().toLocaleTimeString();
    setLogs(prev => [...prev, `[${timestamp}] [${level.toUpperCase()}] ${message}`]);
  };

  const handleScan = async () => {
    if (!sourceVersion || !targetVersion) return;
    
    setIsScanning(true);
    setScanProgress(0);
    setLogs([]);
    addLog('Начинаем сканирование...');
    
    try {
      addLog('Сканирование источника...');
      setScanProgress(25);
      const sourceFiles = await invoke<any[]>('scan_directory_cmd', { 
        path: sourceVersion.install_path,
        categories: selectedCategories
      });
      addLog(`Найдено файлов в источнике: ${sourceFiles.length}`, 'success');
      
      setScanProgress(50);
      addLog('Сканирование цели...');
      const targetFiles = await invoke<any[]>('scan_directory_cmd', { 
        path: targetVersion.install_path,
        categories: selectedCategories
      });
      addLog(`Найдено файлов в цели: ${targetFiles.length}`, 'success');
      
      setScanProgress(75);
      addLog('Создание плана миграции...');
      const migrationPlan = await invoke<MigrationPlan>('create_migration_plan', {
        sourcePath: sourceVersion.install_path,
        targetPath: targetVersion.install_path,
        sourceVersion: sourceVersion.version,
        targetVersion: targetVersion.version,
        categories: selectedCategories,
      });
      
      setScanProgress(100);
      addLog('План создан успешно!', 'success');
      
      setPlan(migrationPlan);
      onScanComplete(migrationPlan);
    } catch (e) {
      addLog(`Ошибка: ${e}`, 'error');
    } finally {
      setIsScanning(false);
    }
  };

  const handleExecute = async () => {
    if (!currentPlan) return;
    
    setIsMigrating(true);
    setMigrationProgress(0);
    addLog('Начинаем миграцию...');
    
    try {
      const backupRoot = backupPath || `${targetVersion.install_path}\\..\\AE-Migration-Tool-Backups`;
      
      // Apply conflict strategy to the plan
      const isConflict = (step: MigrationPlan['steps'][number]) =>
        step.recommendation === 'KeepTarget' && step.reason.toLowerCase().includes('содержимое разное');
      const planToExecute: MigrationPlan = {
        ...currentPlan,
        steps: currentPlan.steps.map(step => {
          if (!isConflict(step)) return step;
          if (conflictStrategy === 'backup_replace') {
            return { ...step, recommendation: 'BackupAndReplace' };
          }
          if (backupKeptTarget) {
            return { ...step, recommendation: 'BackupAndKeepTarget' };
          }
          return step;
        }),
      };
      
      const migrationResult = await invoke<any>('execute_migration', {
        plan: planToExecute,
        sourcePath: sourceVersion.install_path,
        targetPath: targetVersion.install_path,
        backupPath: backupRoot,
      });
      
      setMigrationProgress(100);
      addLog('Миграция завершена!', 'success');
      addLog(`Выполнено: ${migrationResult.completed_steps}, Пропущено: ${migrationResult.skipped_steps}, Ошибок: ${migrationResult.failed_steps}`);
      
      if (migrationResult.errors.length > 0) {
        migrationResult.errors.forEach((err: string) => addLog(err, 'error'));
      }
      
      setResult(migrationResult);
      onMigrationComplete(migrationResult);
    } catch (e) {
      addLog(`Ошибка миграции: ${e}`, 'error');
    } finally {
      setIsMigrating(false);
    }
  };

  const toggleCategory = (cat: Category) => {
    setSelectedCategories(prev => 
      prev.includes(cat) ? prev.filter(c => c !== cat) : [...prev, cat]
    );
  };

  return (
    <div className="main-content">
      <div className="page-header" style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start' }}>
        <div>
          <Button appearance="secondary" icon={<ArrowLeftFilled />} onClick={onBack}>
            Назад
          </Button>
          <h1 className="page-title" style={{ marginTop: '16px' }}>Миграция</h1>
          <p className="page-subtitle">
            {sourceVersion?.version?.display_name} → {targetVersion?.version?.display_name}
          </p>
        </div>
      </div>

      {currentPlan ? (
        <>
          <Card className="card">
            <div className="card-header">
              <h3 className="card-title">План миграции</h3>
            </div>
            
            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(180px, 1fr))', gap: '16px', marginBottom: '24px' }}>
              <div style={{ padding: '16px', background: 'rgba(16, 124, 16, 0.1)', borderRadius: '8px', borderLeft: '4px solid #107c10' }}>
                <div style={{ fontSize: '12px', color: '#616161', marginBottom: '4px' }}>Будет скопировано</div>
                <div style={{ fontSize: '28px', fontWeight: '600', color: '#107c10' }}>
                  {currentPlan.steps.filter(s => s.recommendation === 'KeepSource').length}
                </div>
                <div style={{ fontSize: '11px', color: '#107c10', marginTop: '4px' }}>Новых файлов из старой версии</div>
              </div>
              <div style={{ padding: '16px', background: 'rgba(209, 52, 56, 0.1)', borderRadius: '8px', borderLeft: '4px solid #d13438' }}>
                <div style={{ fontSize: '12px', color: '#616161', marginBottom: '4px' }}>Конфликты (разные файлы)</div>
                <div style={{ fontSize: '28px', fontWeight: '600', color: '#d13438' }}>
                  {currentPlan.steps.filter(s => s.recommendation === 'KeepTarget' && s.reason.toLowerCase().includes('содержимое разное')).length}
                </div>
                <div style={{ fontSize: '11px', color: '#d13438', marginTop: '4px' }}>Нужно выбрать действие</div>
              </div>
              <div style={{ padding: '16px', background: 'rgba(97, 97, 97, 0.1)', borderRadius: '8px', borderLeft: '4px solid #616161' }}>
                <div style={{ fontSize: '12px', color: '#616161', marginBottom: '4px' }}>Идентичны</div>
                <div style={{ fontSize: '28px', fontWeight: '600', color: '#616161' }}>
                  {currentPlan.steps.filter(s => s.recommendation === 'KeepTarget' && s.reason.toLowerCase().includes('идентич')).length}
                </div>
                <div style={{ fontSize: '11px', color: '#616161', marginTop: '4px' }}>Уже одинаковые — пропуск</div>
              </div>
              <div style={{ padding: '16px', background: 'rgba(255, 140, 0, 0.1)', borderRadius: '8px', borderLeft: '4px solid #ff8c00' }}>
                <div style={{ fontSize: '12px', color: '#616161', marginBottom: '4px' }}>Только в новой версии</div>
                <div style={{ fontSize: '28px', fontWeight: '600', color: '#ff8c00' }}>
                  {currentPlan.steps.filter(s => s.recommendation === 'Skip').length}
                </div>
                <div style={{ fontSize: '11px', color: '#ff8c00', marginTop: '4px' }}>Сохраняем как есть</div>
              </div>
            </div>

            {/* Legend */}
            <div style={{ marginBottom: '24px', padding: '12px', background: 'rgba(0,120,212,0.05)', borderRadius: '6px' }}>
              <div style={{ display: 'flex', flexWrap: 'wrap', gap: '16px', alignItems: 'center' }}>
                <span style={{ fontWeight: 600, fontSize: '13px' }}>Легенда:</span>
                <span style={{ display: 'flex', alignItems: 'center', gap: '6px', fontSize: '12px', color: '#107c10' }}>
                  <CheckmarkFilled style={{fontSize: '14px'}} /> Копировать из источника
                </span>
                <span style={{ display: 'flex', alignItems: 'center', gap: '6px', fontSize: '12px', color: '#d13438' }}>
                  <WarningFilled style={{fontSize: '14px'}} /> Конфликт (разные файлы)
                </span>
                <span style={{ display: 'flex', alignItems: 'center', gap: '6px', fontSize: '12px', color: '#616161' }}>
                  <InfoFilled style={{fontSize: '14px'}} /> Идентичны
                </span>
                <span style={{ display: 'flex', alignItems: 'center', gap: '6px', fontSize: '12px', color: '#ff8c00' }}>
                  <DismissFilled style={{fontSize: '14px'}} /> Только в цели
                </span>
                <span style={{ display: 'flex', alignItems: 'center', gap: '6px', fontSize: '12px', color: '#ff8c00' }}>
                  <InfoFilled style={{fontSize: '14px'}} /> Заменить с бэкапом
                </span>
              </div>
            </div>

            {/* Detailed Steps by Action - like ScanAnalysis */}
            <div style={{ maxHeight: '400px', overflow: 'auto' }}>
              {(() => {
                const stepsByRec = {
                  KeepSource: currentPlan.steps.filter(s => s.recommendation === 'KeepSource'),
                  Identical: currentPlan.steps.filter(s => s.recommendation === 'KeepTarget' && s.reason.toLowerCase().includes('идентич')),
                  Conflicts: currentPlan.steps.filter(s => s.recommendation === 'KeepTarget' && s.reason.toLowerCase().includes('содержимое разное')),
                  BackupAndKeepTarget: currentPlan.steps.filter(s => s.recommendation === 'BackupAndKeepTarget'),
                  Skip: currentPlan.steps.filter(s => s.recommendation === 'Skip'),
                  BackupAndReplace: currentPlan.steps.filter(s => s.recommendation === 'BackupAndReplace'),
                };

                const recLabels = {
                  KeepSource: { label: 'Копировать из источника', icon: <CheckmarkFilled style={{color: '#107c10'}}/>, color: '#107c10' },
                  Identical: { label: 'Идентичны', icon: <InfoFilled style={{color: '#616161'}}/>, color: '#616161' },
                  Conflicts: { label: 'Конфликты (файлы отличаются)', icon: <WarningFilled style={{color: '#d13438'}}/>, color: '#d13438' },
                  BackupAndKeepTarget: { label: 'Оставить целевой с резервной копией', icon: <InfoFilled style={{color: '#0078d4'}}/>, color: '#0078d4' },
                  Skip: { label: 'Только в новой версии (пропуск)', icon: <DismissFilled style={{color: '#ff8c00'}}/>, color: '#ff8c00' },
                  BackupAndReplace: { label: 'Заменить с бэкапом', icon: <InfoFilled style={{color: '#ff8c00'}}/>, color: '#ff8c00' },
                };

                const groupByTopFolder = (steps: any[]) => {
                  const groups = new Map<string, any[]>();
                  for (const step of steps) {
                    const relPath = step.relative_path;
                    const parts = relPath.split(/[\\/]/);
                    let topFolder = parts[0];
                    if (topFolder.toLowerCase() === 'support files' && parts.length > 1) {
                      topFolder = parts[1];
                    }
                    if (parts.length > 2 && parts[1].toLowerCase() === 'presets') {
                      topFolder = parts[2];
                    }
                    if (!groups.has(topFolder)) groups.set(topFolder, []);
                    groups.get(topFolder)!.push(step);
                  }
                  return groups;
                };

                return Object.entries(stepsByRec).map(([recKey, steps]) => {
                  if (steps.length === 0) return null;
                  const info = recLabels[recKey as keyof typeof recLabels];
                  const shouldGroup = recKey === 'KeepSource' || recKey === 'Conflicts' || recKey === 'BackupAndKeepTarget';
                  const groups = shouldGroup ? groupByTopFolder(steps) : new Map([[`Все файлы (${steps.length})`, steps]]);

                  return (
                    <div key={recKey} style={{ marginBottom: '16px' }}>
                      <div style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '8px', padding: '8px 12px', background: `${info.color}15`, borderRadius: '6px', borderLeft: `3px solid ${info.color}` }}>
                        {info.icon}
                        <strong style={{ color: info.color }}>{info.label}</strong>
                        <span style={{ color: '#616161', fontSize: '12px', marginLeft: '8px' }}>({steps.length} файлов)</span>
                      </div>
                      <div style={{ paddingLeft: '16px', paddingRight: '16px' }}>
                        {Array.from(groups.entries()).map(([folderName, folderSteps]) => (
                          <div key={folderName} style={{ marginBottom: '8px' }}>
                            <details style={{ width: '100%' }}>
                              <summary style={{
                                display: 'flex', alignItems: 'center', gap: '8px',
                                padding: '6px 10px', background: 'rgba(0,0,0,0.03)',
                                border: '1px solid var(--border)', borderRadius: '4px',
                                cursor: 'pointer', fontSize: '12px', fontFamily: 'monospace',
                                width: '100%', textAlign: 'left', listStyle: 'none'
                              }}>
                                <FolderFilled style={{fontSize: '14px', color: '#ffc107'}} />
                                <span style={{ fontWeight: 500, color: '#0078d4' }}>{folderName}</span>
                                <span style={{ color: '#616161', marginLeft: '8px' }}>({folderSteps.length} файлов)</span>
                              </summary>
                              <div style={{ paddingLeft: '24px', marginTop: '4px', borderLeft: '2px solid var(--border)' }}>
                                {folderSteps.map((step) => (
                                  <div key={step.id} style={{ fontSize: '11px', fontFamily: 'monospace', padding: '2px 0', display: 'flex', gap: '8px', alignItems: 'center' }}>
                                    <DocumentFilled style={{fontSize: '12px', color: 'var(--neutral-secondary)'}} />
                                    <span style={{ flex: 1, wordBreak: 'break-all', color: 'var(--neutral-primary)' }}>{step.relative_path}</span>
                                    <span style={{ color: 'var(--neutral-tertiary)', fontSize: '10px' }}>{step.category}</span>
                                    <span style={{ color: info.color, fontSize: '10px', fontWeight: 500 }}>{step.reason}</span>
                                  </div>
                                ))}
                              </div>
                            </details>
                          </div>
                        ))}
                      </div>
                    </div>
                  );
                });
              })()}
            </div>
          </Card>

          <Card className="card">
            <div className="card-header">
              <h3 className="card-title">Настройки конфликтов</h3>
            </div>
            <RadioGroup value={conflictStrategy} onChange={(e: any) => setConflictStrategy(e.target.value as 'keep_target' | 'backup_replace')}>
              <Label>
                <Radio value="keep_target" />
                Сохранить целевой (безопасно) — при конфликте оставляем файл в целевой версии
              </Label>
              {conflictStrategy === 'keep_target' && (
                <Checkbox
                  checked={backupKeptTarget}
                  onChange={(_, data) => setBackupKeptTarget(data.checked === true)}
                  label="Создавать резервную копию целевого файла для возможного отката"
                  style={{ margin: '8px 0 4px 28px' }}
                />
              )}
              <Label style={{ marginTop: '8px', display: 'block' }}>
                <Radio value="backup_replace" />
                Резервная копия + заменить — делаем бэкап целевого файла, копируем из источника
              </Label>
            </RadioGroup>
          </Card>

          <Card className="card">
            <div className="card-header">
              <h3 className="card-title">Папка для бэкапов</h3>
            </div>
            <Input
              placeholder="Автоматически: рядом с целевой версией"
              value={backupPath}
              onChange={(e: React.ChangeEvent<HTMLInputElement>) => setBackupPath(e.target.value)}
              style={{ width: '100%' }}
            />
          </Card>

          <div className="btn-group" style={{ justifyContent: 'flex-end' }}>
            <Button appearance="primary" icon={<PlayFilled />} onClick={handleExecute} disabled={isMigrating}>
              {isMigrating ? 'Выполняется...' : 'Выполнить миграцию'}
            </Button>
          </div>

          {isMigrating && (
            <ProgressBar value={migrationProgress} style={{ marginTop: '16px' }} />
          )}

          {result && (
            <Card className="card" style={{ marginTop: '16px', borderColor: result.failed_steps > 0 ? '#d13438' : '#107c10' }}>
              <div className="card-header">
                <h3 className="card-title">Результат миграции</h3>
              </div>
              <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '16px' }}>
                <div style={{ textAlign: 'center' }}>
                  <div style={{ fontSize: '32px', fontWeight: '600', color: '#107c10' }}>{result.completed_steps}</div>
                  <div style={{ fontSize: '12px', color: '#616161' }}>Выполнено</div>
                </div>
                <div style={{ textAlign: 'center' }}>
                  <div style={{ fontSize: '32px', fontWeight: '600', color: '#616161' }}>{result.skipped_steps}</div>
                  <div style={{ fontSize: '12px', color: '#616161' }}>Пропущено</div>
                </div>
                <div style={{ textAlign: 'center' }}>
                  <div style={{ fontSize: '32px', fontWeight: '600', color: '#d13438' }}>{result.failed_steps}</div>
                  <div style={{ fontSize: '12px', color: '#616161' }}>Ошибок</div>
                </div>
              </div>
            </Card>
          )}
        </>
      ) : (
        <>
          <Card className="card">
            <div className="card-header">
              <h3 className="card-title">Категории для миграции</h3>
            </div>
            <p style={{ color: '#616161', marginBottom: '16px' }}>
              Выберите типы файлов, которые нужно включить в миграцию
            </p>
            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(280px, 1fr))', gap: '12px' }}>
              {CATEGORIES.map(cat => (
                <label key={cat} className="category-checkbox">
                  <input
                    type="checkbox"
                    checked={selectedCategories.includes(cat)}
                    onChange={() => toggleCategory(cat)}
                  />
                  <div className="category-info">
                    <div className="category-name">
                      <span style={{ fontSize: '18px', marginRight: '8px' }}>{categoryIcons[cat]}</span>
                      {categoryLabels[cat]}
                    </div>
                    <div className="category-hint">{categoryHints[cat]}</div>
                  </div>
                </label>
              ))}
            </div>
          </Card>

          <div className="btn-group" style={{ justifyContent: 'center' }}>
            <Button 
              appearance="primary" 
              icon={<SearchFilled />}
              onClick={handleScan}
              disabled={isScanning || selectedCategories.length === 0}
            >
              {isScanning ? 'Сканирование...' : 'Сканировать и создать план'}
            </Button>
          </div>

          {isScanning && (
            <ProgressBar value={scanProgress} style={{ marginTop: '16px' }} />
          )}
        </>
      )}

      {(logs.length > 0 || isScanning || isMigrating) && (
        <Card className="card" style={{ marginTop: '24px' }}>
          <div className="card-header">
            <h3 className="card-title">Лог</h3>
          </div>
          <div style={{ maxHeight: '200px', overflow: 'auto', fontFamily: 'monospace', fontSize: '12px' }}>
            {logs.map((log, i) => (
              <div key={i} className="log-entry">
                <span className="log-time">{log.match(/\[(.*?)\]/)?.[1] || ''}</span>
                <span className={`log-level ${log.match(/\[(INFO|SUCCESS|WARNING|ERROR)\]/i)?.[1]?.toLowerCase() || 'info'}`}>
                  {log.match(/\[(INFO|SUCCESS|WARNING|ERROR)\]/i)?.[1] || ''}
                </span>
                <span className="log-message">{log.replace(/\[.*?\]\s*\[.*?\]\s*/, '')}</span>
              </div>
            ))}
          </div>
        </Card>
      )}
    </div>
  );
};
