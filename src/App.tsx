import React from 'react';
import { invoke } from '@tauri-apps/api/core';
import { 
  HomeFilled, 
  SearchFilled, 
  SettingsFilled, 
  HistoryFilled,
  DarkThemeFilled,
  CommentLightningFilled
} from '@fluentui/react-icons';
import { Dashboard } from './pages/Dashboard';
import { VersionSelection } from './pages/VersionSelection';
import { ScanAnalysis } from './pages/ScanAnalysis';
import { MigrationReport } from './pages/MigrationReport';
import { Migration } from './pages/Migration';
import { History } from './pages/History';
import { Page, Installation, MigrationPlan } from './types';
import { useTheme } from './context/ThemeContext';

function App() {
  const [currentPage, setCurrentPage] = React.useState<Page>('dashboard');
  const [installations, setInstallations] = React.useState<Installation[]>([]);
  const [currentSourceVersion, setCurrentSourceVersion] = React.useState<Installation | null>(null);
  const [currentTargetVersion, setCurrentTargetVersion] = React.useState<Installation | null>(null);
  const [currentPlan, setCurrentPlan] = React.useState<MigrationPlan | null>(null);
  const { theme, toggleTheme } = useTheme();
  const installationPaths = React.useMemo(
    () => installations.map(installation => installation.install_path),
    [installations],
  );

  React.useEffect(() => {
    loadInstallations();
  }, []);

  const loadInstallations = async () => {
    try {
      const installs = await invoke<Installation[]>('detect_installations_cmd');
      setInstallations(installs);
    } catch (e) {
      console.error('Failed to load installations:', e);
    }
  };

  const handleInstallationAdded = (installation: Installation) => {
    setInstallations(current => current.some(item => item.id === installation.id)
      ? current
      : [...current, installation]);
  };

  const handleVersionSelect = (sourceId: string, targetId: string) => {
    const source = installations.find(i => i.id === sourceId);
    const target = installations.find(i => i.id === targetId);
    if (source && target) {
      setCurrentSourceVersion(source);
      setCurrentTargetVersion(target);
      setCurrentPage('migration');
    }
  };

  const handleScanComplete = (plan: MigrationPlan) => {
    setCurrentPlan(plan);
    setCurrentPage('scan-analysis');
  };

  const handleMigrationComplete = () => {
    setCurrentPage('history');
  };

  const renderPage = () => {
    switch (currentPage) {
      case 'dashboard':
        return <Dashboard 
          installations={installations}
          onVersionSelect={handleVersionSelect}
          onRefresh={loadInstallations}
          onInstallationAdded={handleInstallationAdded}
        />;
      case 'version-selection':
        return <VersionSelection 
          installations={installations}
          onRefresh={loadInstallations}
          onInstallationAdded={handleInstallationAdded}
        />;
      case 'scan-analysis':
        return <ScanAnalysis 
          plan={currentPlan}
          onExecute={() => setCurrentPage('migration')}
          onBack={() => setCurrentPage('migration-options')}
        />;
      case 'migration-report':
        return <MigrationReport plan={currentPlan} />;
      case 'migration':
        return <Migration 
          sourceVersion={currentSourceVersion}
          targetVersion={currentTargetVersion}
          plan={currentPlan}
          onScanComplete={handleScanComplete}
          onMigrationComplete={handleMigrationComplete}
          onBack={() => setCurrentPage('scan-analysis')}
        />;
      case 'history':
        return <History installationPaths={installationPaths} />;
      default:
        return <Dashboard 
          installations={installations}
          onVersionSelect={handleVersionSelect}
          onRefresh={loadInstallations}
          onInstallationAdded={handleInstallationAdded}
        />;
    }
  };

  return (
    <div className="app-layout">
      <nav className="sidebar">
        <div className="sidebar-header">AE Migration Tool</div>
        <div className="sidebar-nav">
          <button 
            className={`nav-item ${currentPage === 'dashboard' ? 'active' : ''}`}
            onClick={() => setCurrentPage('dashboard')}
          >
            <span className="nav-icon"><HomeFilled /></span>
            Главная
          </button>
          <button 
            className={`nav-item ${currentPage === 'version-selection' ? 'active' : ''}`}
            onClick={() => setCurrentPage('version-selection')}
          >
            <span className="nav-icon"><SearchFilled /></span>
            Версии
          </button>
          <button 
            className={`nav-item ${currentPage === 'history' ? 'active' : ''}`}
            onClick={() => setCurrentPage('history')}
          >
            <span className="nav-icon"><HistoryFilled /></span>
            История
          </button>
          <button 
            className={`nav-item ${currentPage === 'migration' || currentPage === 'scan-analysis' || currentPage === 'migration-report' ? 'active' : ''}`}
            onClick={() => setCurrentPage('migration')}
          >
            <span className="nav-icon"><SettingsFilled /></span>
            Миграция
          </button>
        </div>
        <div style={{ padding: '8px 12px', borderTop: '1px solid var(--border)' }}>
          <button 
            className="nav-item"
            onClick={toggleTheme}
            style={{ justifyContent: 'center' }}
            aria-label={theme === 'dark' ? 'Switch to light theme' : 'Switch to dark theme'}
          >
            <span className="nav-icon">
              {theme === 'dark' ? <CommentLightningFilled /> : <DarkThemeFilled />}
            </span>
            {theme === 'dark' ? 'Светлая тема' : 'Тёмная тема'}
          </button>
        </div>
      </nav>
      <main className="main-content">
        {renderPage()}
      </main>
    </div>
  );
}

export default App;
