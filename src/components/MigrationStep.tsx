import React from 'react';
import { Badge } from '@fluentui/react-components';
import { MigrationStep } from '../types';

interface StepCardProps {
  step: MigrationStep;
  index: number;
  showDetails?: boolean;
}

const recommendationLabels: Record<string, string> = {
  KeepTarget: 'Сохранить целевой',
  KeepSource: 'Скопировать из источника',
  Merge: 'Объединить',
  Skip: 'Пропустить',
  BackupAndReplace: 'Резервная копия + Заменить',
};

const statusLabels: Record<string, string> = {
  Pending: 'Ожидает',
  Running: 'Выполняется',
  Completed: 'Завершено',
  Failed: 'Ошибка',
  RolledBack: 'Отменено',
  Cancelled: 'Отменено',
};

const recommendationAppearances: Record<string, 'outline' | 'filled' | 'ghost' | 'tint'> = {
  KeepTarget: 'outline',
  KeepSource: 'filled',
  Merge: 'tint',
  Skip: 'outline',
  BackupAndReplace: 'filled',
};

const recommendationColors: Record<string, string> = {
  KeepTarget: '#107c10',
  KeepSource: '#0078d4',
  Merge: '#ff8c00',
  Skip: '#616161',
  BackupAndReplace: '#d13438',
};

export const StepCard: React.FC<StepCardProps> = ({ step, index, showDetails = false }) => {
  return (
    <div className="card">
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start' }}>
        <div>
          <h4 style={{ margin: '0 0 8px 0' }}>
            Шаг {index + 1}: {step.relative_path}
          </h4>
          <Badge 
            appearance={recommendationAppearances[step.recommendation]} 
            color={recommendationColors[step.recommendation] as any}
            className="status-badge"
          >
            {recommendationLabels[step.recommendation] || step.recommendation}
          </Badge>
        </div>
        {showDetails && (
          <Badge appearance="outline" className="status-badge info">
            {statusLabels[step.status] || step.status}
          </Badge>
        )}
      </div>
      
      {showDetails && (
        <>
          <div style={{ marginTop: '12px', fontSize: '12px' }}>
            <div><strong>Источник:</strong> {step.source_path}</div>
            <div><strong>Цель:</strong> {step.target_path}</div>
            <div><strong>Причина:</strong> {step.reason}</div>
          </div>
          <div style={{ height: '8px', background: '#e1e1e1', borderRadius: '4px', overflow: 'hidden', marginTop: '12px' }}>
            <div style={{ 
              height: '100%', 
              width: `${step.status === 'Completed' ? 100 : step.status === 'Running' ? 50 : 0}%`, 
              background: '#0078d4', 
              borderRadius: '4px',
              transition: 'width 0.3s ease'
            }} />
          </div>
        </>
      )}
    </div>
  );
};