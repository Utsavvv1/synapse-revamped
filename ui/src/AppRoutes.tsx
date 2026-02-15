import { Routes, Route } from 'react-router-dom';
import SynapseApp from './pages/SynapseApp';
import BlockWarningPage from './pages/BlockWarningPage';
import StatisticsPage from './pages/StatisticsPage';

export default function AppRoutes() {
    return (
        <Routes>
            <Route path="/" element={<SynapseApp />} />
            <Route path="/block-warning" element={<BlockWarningPage />} />
            <Route path="/statistics" element={<StatisticsPage />} />
        </Routes>
    );
}
