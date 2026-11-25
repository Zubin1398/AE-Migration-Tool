import React from 'react';
import { Button, Card } from '@fluentui/react-components';
import { PlayFilled, ArrowLeftFilled, InfoFilled, WarningFilled, CheckmarkFilled, DismissFilled, ChevronDownFilled, ChevronRightFilled, FolderFilled, DocumentFilled } from '@fluentui/react-icons';
import { MigrationPlan } from '../types';

interface ScanAnalysisProps {
  plan: MigrationPlan | null;
  onExecute: () => void;
  onBack: () => void;
}

// Group files by their top-level folder (e.g., "!!!_MOLOB Effects_!!!")
function groupByTopFolder(steps: any[]) {
  const groups = new Map<string, any[]>();
  
  for (const step of steps) {
    const relPath = step.relative_path;
    // Get the first folder after "Support Files\" or similar
    const parts = relPath.split(/[\\/]/);
    let topFolder = parts[0];
    
    // If path starts with "Support Files", use the next part as top folder
    if (topFolder.toLowerCase() === 'support files' && parts.length > 1) {
      topFolder = parts[1];
    }
    
    // For presets, try to get the actual preset pack folder
    if (parts.length > 2 && parts[1].toLowerCase() === 'presets') {
      topFolder = parts[2];
    }
    
    if (!groups.has(topFolder)) {
      groups.set(topFolder, []);
    }
    groups.get(topFolder)!.push(step);
  }
  
  return groups;
}

const recommendationLabels: Record<string, { label: string; icon: React.ReactNode; color: string; desc: string }> = {
  KeepSource: { 
    label: 'Копировать из источника', 
    icon: <CheckmarkFilled style={{color: '#107c10'}}/>, 
    color: '#107c10',
    desc: 'Файла нет в новой версии — он будет скопирован' 
  },
  KeepTarget: { 
    label: 'Оставить в цели', 
    icon: <InfoFilled style={{color: '#616161'}}/>, 
    color: '#616161',
    desc: 'Файл идентичен в обеих версиях — оставляем как есть' 
  },
  Skip: { 
    label: 'Пропустить (только в цели)', 
    icon: <DismissFilled style={{color: '#ff8c00'}}/>, 
    color: '#ff8c00',
    desc: 'Файл есть только в новой версии — не трогаем' 
  },
  BackupAndReplace: { 
    label: 'Заменить с бэкапом', 
    icon: <WarningFilled style={{color: '#d13438'}}/>, 
    color: '#d13438',
    desc: 'Файл отличается — можно заменить, старый сохранится в бэкап' 
  },
};

export const ScanAnalysis: React.FC<ScanAnalysisProps> = ({ plan, onExecute, onBack }) => {
  if (!plan) return null;

  const stats = {
    total: plan.total_files,
    conflicts: plan.conflicts,
    toCopy: plan.steps.filter(s => s.recommendation === 'KeepSource').length,
    identical: plan.steps.filter(s => s.recommendation === 'KeepTarget' && s.reason.includes('идентичен')).length,
    conflictsReal: plan.steps.filter(s => s.recommendation === 'KeepTarget' && s.reason.includes('РАЗНОЕ')).length,
    targetOnly: plan.steps.filter(s => s.recommendation === 'Skip').length,
  };

  const stepsByRec = {
    KeepSource: plan.steps.filter(s => s.recommendation === 'KeepSource'),
    KeepTarget: plan.steps.filter(s => s.recommendation === 'KeepTarget' && s.reason.includes('идентичен')),
    Conflicts: plan.steps.filter(s => s.recommendation === 'KeepTarget' && s.reason.includes('РАЗНОЕ')),
    Skip: plan.steps.filter(s => s.recommendation === 'Skip'),
  };

  const [expandedFolders, setExpandedFolders] = React.useState<Set<string>>(new Set());

  const toggleFolder = (folderName: string) => {
    setExpandedFolders(prev => {
      const next = new Set(prev);
      if (next.has(folderName)) next.delete(folderName);
      else next.add(folderName);
      return next;
    });
  };

  return (
    <div className="main-content">
      <div className="page-header" style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start' }}>
        <div>
          <Button appearance="secondary" icon={<ArrowLeftFilled />} onClick={onBack}>
            Назад к настройкам
          </Button>
          <h1 className="page-title" style={{ marginTop: '16px' }}>Анализ и план миграции</h1>
          <p className="page-subtitle">
            Проверьте, что будет перенесено. <strong>Зелёное = копируется</strong>, <strong>Серое = оставляется</strong>, <strong>Красное = конфликт (нужно решить)</strong>
          </p>
        </div>
      </div>

      {/* Summary Cards */}
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(180px, 1fr))', gap: '16px', marginBottom: '24px' }}>
        <Card className="card" style={{ textAlign: 'center', borderLeft: '4px solid #107c10' }}>
          <div style={{ fontSize: '12px', color: '#616161', marginBottom: '4px' }}>Будет скопировано</div>
          <div style={{ fontSize: '32px', fontWeight: '600', color: '#107c10' }}>{stats.toCopy}</div>
          <div style={{ fontSize: '11px', color: '#107c10', marginTop: '4px' }}>Новых файлов из старой версии</div>
        </Card>
        <Card className="card" style={{ textAlign: 'center', borderLeft: '4px solid #d13438' }}>
          <div style={{ fontSize: '12px', color: '#616161', marginBottom: '4px' }}>Конфликты</div>
          <div style={{ fontSize: '32px', fontWeight: '600', color: '#d13438' }}>{stats.conflictsReal}</div>
          <div style={{ fontSize: '11px', color: '#d13438', marginTop: '4px' }}>Файлы отличаются — нужно выбрать</div>
        </Card>
        <Card className="card" style={{ textAlign: 'center', borderLeft: '4px solid #616161' }}>
          <div style={{ fontSize: '12px', color: '#616161', marginBottom: '4px' }}>Идентичны</div>
          <div style={{ fontSize: '32px', fontWeight: '600', color: '#616161' }}>{stats.identical}</div>
          <div style={{ fontSize: '11px', color: '#616161', marginTop: '4px' }}>Уже одинаковые — пропуск</div>
        </Card>
        <Card className="card" style={{ textAlign: 'center', borderLeft: '4px solid #ff8c00' }}>
          <div style={{ fontSize: '12px', color: '#616161', marginBottom: '4px' }}>Только в новой версии</div>
          <div style={{ fontSize: '32px', fontWeight: '600', color: '#ff8c00' }}>{stats.targetOnly}</div>
          <div style={{ fontSize: '11px', color: '#ff8c00', marginTop: '4px' }}>Сохраняем как есть</div>
        </Card>
      </div>

      {/* Legend */}
      <Card className="card" style={{ marginBottom: '24px', background: 'rgba(0,120,212,0.05)' }}>
        <div style={{ display: 'flex', flexWrap: 'wrap', gap: '16px', alignItems: 'center' }}>
          <span style={{ fontWeight: 600, fontSize: '13px' }}>Легенда:</span>
          {Object.entries(recommendationLabels).map(([key, v]) => (
            <span key={key} style={{ display: 'flex', alignItems: 'center', gap: '6px', fontSize: '12px', color: v.color }}>
              {v.icon} {v.label}
            </span>
          ))}
        </div>
      </Card>

      {/* Detailed Steps by Action */}
      <Card className="card" style={{ marginBottom: '24px' }}>
        <div className="card-header">
          <h3 className="card-title">Детали по действиям</h3>
        </div>
        
        {Object.entries(stepsByRec).map(([recKey, steps]) => {
          if (steps.length === 0) return null;
          const info = recommendationLabels[recKey] || { label: recKey, icon: null, color: '#616161', desc: '' };
          
          // Group by top folder for KeepSource and Conflicts
          const shouldGroup = recKey === 'KeepSource' || recKey === 'Conflicts';
          const groups = shouldGroup ? groupByTopFolder(steps) : new Map([[`Все файлы (${steps.length})`, steps]]);
          
          return (
            <div key={recKey} style={{ marginBottom: '16px' }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '8px', padding: '8px 12px', background: `${info.color}15`, borderRadius: '6px', borderLeft: `3px solid ${info.color}` }}>
                {info.icon}
                <strong style={{ color: info.color }}>{info.label}</strong>
                <span style={{ color: '#616161', fontSize: '12px', marginLeft: '8px' }}>({steps.length} файлов)</span>
                <div style={{ marginLeft: 'auto', color: '#616161', fontSize: '11px' }}>{info.desc}</div>
              </div>
              <div style={{ maxHeight: '300px', overflow: 'auto', paddingLeft: '16px', paddingRight: '16px' }}>
                {Array.from(groups.entries()).map(([folderName, folderSteps]) => (
                  <div key={folderName} style={{ marginBottom: '8px' }}>
                    <button
                      onClick={() => toggleFolder(`${recKey}-${folderName}`)}
                      style={{
                        display: 'flex',
                        alignItems: 'center',
                        gap: '8px',
                        padding: '6px 10px',
                        background: 'rgba(0,0,0,0.03)',
                        border: '1px solid var(--border)',
                        borderRadius: '4px',
                        cursor: 'pointer',
                        fontSize: '12px',
                        fontFamily: 'monospace',
                        width: '100%',
                        textAlign: 'left'
                      }}
                    >
                      {expandedFolders.has(`${recKey}-${folderName}`) ? <ChevronDownFilled style={{fontSize: '12px', color: 'var(--neutral-secondary)'}} /> : <ChevronRightFilled style={{fontSize: '12px', color: 'var(--neutral-secondary)' }} />}
                      <FolderFilled style={{fontSize: '14px', color: '#ffc107'}} />
                      <span style={{ fontWeight: 500, color: '#0078d4' }}>{folderName}</span>
                      <span style={{ color: '#616161', marginLeft: '8px' }}>({folderSteps.length} файлов)</span>
                    </button>
                    {expandedFolders.has(`${recKey}-${folderName}`) && (
                      <div style={{ paddingLeft: '24px', marginTop: '4px', borderLeft: '2px solid var(--border)', paddingRight: '16px' }}>
                        {folderSteps.map((step) => (
                          <div key={step.id} style={{ fontSize: '11px', fontFamily: 'monospace', padding: '2px 0', display: 'flex', gap: '8px', alignItems: 'center' }}>
                            <DocumentFilled style={{fontSize: '12px', color: 'var(--neutral-secondary)'}} />
                            <span style={{ flex: 1, wordBreak: 'break-all', color: 'var(--neutral-primary)' }}>{step.relative_path}</span>
                            <span style={{ color: 'var(--neutral-tertiary)', fontSize: '10px' }}>{step.category}</span>
                          </div>
                        ))}
                      </div>
                    )}
                  </div>
                ))}
              </div>
            </div>
          );
        })}
      </Card>

      <div className="btn-group" style={{ justifyContent: 'flex-end' }}>
        <Button appearance="primary" icon={<PlayFilled />} onClick={onExecute} style={{ minWidth: '200px' }}>
          Выполнить миграцию
        </Button>
      </div>
    </div>
  );
};