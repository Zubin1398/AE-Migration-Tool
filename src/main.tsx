import React from 'react';
import ReactDOM from 'react-dom/client';
import { FluentProvider, webLightTheme, webDarkTheme } from '@fluentui/react-components';
import { ThemeProvider } from './context/ThemeContext';
import { useTheme } from './context/ThemeContext';
import App from './App';
import './styles.css';

const ThemedApp = () => {
  const { theme } = useTheme();
  return (
    <FluentProvider theme={theme === 'dark' ? webDarkTheme : webLightTheme}>
      <App />
    </FluentProvider>
  );
};

const root = ReactDOM.createRoot(
  document.getElementById('root') as HTMLElement
);

root.render(
  <React.StrictMode>
    <ThemeProvider>
      <ThemedApp />
    </ThemeProvider>
  </React.StrictMode>
);