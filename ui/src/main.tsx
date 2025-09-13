import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import './styles/globals.css';
import SynapseApp from './pages/SynapseApp';

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <SynapseApp />
  </StrictMode>,
);
