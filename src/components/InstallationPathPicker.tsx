import React from 'react';
import { Button, Input } from '@fluentui/react-components';
import { FolderOpenRegular } from '@fluentui/react-icons';
import { open } from '@tauri-apps/plugin-dialog';

interface InstallationPathPickerProps {
  value: string;
  placeholder?: string;
  addLabel: string;
  adding?: boolean;
  onChange: (path: string) => void;
  onAdd: () => void;
}

export const InstallationPathPicker: React.FC<InstallationPathPickerProps> = ({
  value,
  placeholder = 'Путь к папке After Effects',
  addLabel,
  adding = false,
  onChange,
  onAdd,
}) => {
  const chooseFolder = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: 'Выберите папку Adobe After Effects',
      });
      if (typeof selected === 'string') onChange(selected);
    } catch (error) {
      alert(`Не удалось открыть выбор папки: ${error}`);
    }
  };

  return (
    <div className="installation-path-picker">
      <Input
        aria-label={addLabel}
        placeholder={placeholder}
        value={value}
        onChange={event => onChange(event.target.value)}
        onKeyDown={event => {
          if (event.key === 'Enter' && value.trim() && !adding) onAdd();
        }}
      />
      <Button
        appearance="secondary"
        icon={<FolderOpenRegular />}
        onClick={chooseFolder}
        aria-label="Обзор папок"
        title="Выбрать папку"
      >
        Обзор
      </Button>
      <Button appearance="primary" onClick={onAdd} disabled={!value.trim() || adding}>
        {adding ? 'Добавление…' : addLabel}
      </Button>
    </div>
  );
};
