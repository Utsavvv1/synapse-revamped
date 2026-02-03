
import { Routes, Route } from 'react-router-dom';
import SynapseApp from './pages/SynapseApp';
import BlockWarningPage from './pages/BlockWarningPage';

export default function AppRoutes() {
    return (
        <Routes>
            <Route path="/" element={<SynapseApp />} />
            <Route path="/block-warning" element={<BlockWarningPage />} />
        </Routes>
    );
}
