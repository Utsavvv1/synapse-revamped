import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import './globals.css';
import SynapseApp from './SynapseApp';

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <SynapseApp />
  </StrictMode>,
);
