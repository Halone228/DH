import { HashRouter, Routes, Route } from 'react-router-dom';
import { ErrorBoundary } from './components/ErrorBoundary';
import { OfflineBanner } from './components/OfflineBanner';
import { Layout } from './components/Layout';
import { ReminderList } from './pages/ReminderList';
import { CreateReminder } from './pages/CreateReminder';
import { Settings } from './pages/Settings';
import { Profile } from './pages/Profile';

function App() {
  return (
    <ErrorBoundary>
      <HashRouter>
        <OfflineBanner />
        <Layout>
          <Routes>
            <Route path="/" element={<ReminderList />} />
            <Route path="/create" element={<CreateReminder />} />
            <Route path="/settings" element={<Settings />} />
            <Route path="/profile" element={<Profile />} />
          </Routes>
        </Layout>
      </HashRouter>
    </ErrorBoundary>
  );
}

export default App;
