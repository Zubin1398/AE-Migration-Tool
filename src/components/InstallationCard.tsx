import React from 'react';
import { Badge } from '@fluentui/react-components';
import { Installation } from '../types';

interface InstallationCardProps {
  installation: Installation;
  isSelected?: boolean;
  onSelect?: () => void;
  showDetails?: boolean;
}

export const InstallationCard: React.FC<InstallationCardProps> = ({
  installation,
  isSelected = false,
  onSelect,
  showDetails = false,
}) => {
  return (
    <div 
      className={`installation-card ${isSelected ? 'selected' : ''}`}
      onClick={onSelect}
    >
      <div>
        <div className="installation-name">
          {installation.install_path.split('\\').pop() || installation.id}
          <Badge appearance="outline" className="status-badge success">
            {installation.is_complete ? 'Готов' : 'Не готов'}
          </Badge>
        </div>
        <div className="installation-path">{installation.install_path}</div>
        {showDetails && (
          <>
            <div style={{ fontSize: '12px', color: '#616161' }}>
              Версия: {installation.version.display_name}
            </div>
            <div style={{ fontSize: '12px', color: '#616161' }}>
              Метод обнаружения: {installation.detection_method}
            </div>
          </>
        )}
      </div>
    </div>
  );
};