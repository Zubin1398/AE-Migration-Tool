import React from 'react';
import { Button, Card } from '@fluentui/react-components';
import { ArrowLeftFilled, ArrowDownload20Filled, CheckmarkFilled, InfoFilled, WarningFilled, DismissFilled } from '@fluentui/react-icons';
import { MigrationPlan } from '../types';

interface MigrationReportProps {
  plan: MigrationPlan | null;
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

export const MigrationReport: React.FC<MigrationReportProps> = ({ plan }) => {
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

  return (
    <div className="main-content">
      <div className="page-header" style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start' }}>
        <div>
          <Button appearance="secondary" icon={<ArrowLeftFilled />} onClick={() => window.history.back()}>
            Назад
          </Button>
          <h1 className="page-title" style={{ marginTop: '16px' }}>Отчёт миграции</h1>
          <p className="page-subtitle">
            План ID: {plan.id}
          </p>
        </div>
        <Button appearance="primary" icon={<ArrowDownload20Filled />}>
          Экспорт JSON
        </Button>
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
      <Card className="card">
        <div className="card-header">
          <h3 className="card-title">Полный список действий</h3>
        </div>
        
        {Object.entries(stepsByRec).map(([recKey, steps]) => {
          if (steps.length === 0) return null;
          const info = recommendationLabels[recKey] || { label: recKey, icon: null, color: '#616161', desc: '' };
          return (
            <div key={recKey} style={{ marginBottom: '16px' }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '8px', padding: '8px 12px', background: `${info.color}15`, borderRadius: '6px', borderLeft: `3px solid ${info.color}` }}>
                {info.icon}
                <strong style={{ color: info.color }}>{info.label}</strong>
                <span style={{ color: '#616161', fontSize: '12px', marginLeft: '8px' }}>({steps.length} файлов)</span>
                <div style={{ marginLeft: 'auto', color: '#616161', fontSize: '11px' }}>{info.desc}</div>
              </div>
              <div style={{ maxHeight: '250px', overflow: 'auto', paddingLeft: '28px' }}>
                {steps.map((step) => (
                  <div key={step.id} style={{ fontSize: '12px', fontFamily: 'monospace', padding: '4px 0', borderBottom: '1px solid var(--border)', display: 'flex', gap: '12px', alignItems: 'flex-start' }}>
                    <span style={{ color: '#616161', minWidth: '100px' }}>{step.category}</span>
                    <span style={{ flex: 1, wordBreak: 'break-all' }}>{step.relative_path}</span>
                    <span style={{ color: '#616161', fontSize: '11px', whiteSpace: 'nowrap' }}>{step.reason}</span>
                  </div>
                ))}
              </div>
            </div>
          );
        })}
      </Card>
    </div>
  );
};