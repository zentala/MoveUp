import React from 'react';
import { Link } from 'react-router-dom';

const MainPanel: React.FC = ({ children }) => {
  return (
    <div className="min-h-screen bg-gray-100">
      <header className="bg-white shadow">
        <div className="container mx-auto px-4 py-4 flex justify-between items-center">
          <div className="text-2xl font-bold text-blue-600">MoveUp</div>
          <nav className="space-x-4">
            <Link to="/" className="text-gray-700 hover:text-blue-600">
              Dashboard
            </Link>
            <Link
              to="/achievements"
              className="text-gray-700 hover:text-blue-600"
            >
              Achievements
            </Link>
            <Link to="/rewards" className="text-gray-700 hover:text-blue-600">
              Rewards
            </Link>
            <Link
              to="/challenges"
              className="text-gray-700 hover:text-blue-600"
            >
              Challenges
            </Link>
            <Link to="/community" className="text-gray-700 hover:text-blue-600">
              Community
            </Link>
            <Link
              to="/statistics"
              className="text-gray-700 hover:text-blue-600"
            >
              Statistics
            </Link>
            <Link to="/settings" className="text-gray-700 hover:text-blue-600">
              Settings
            </Link>
          </nav>
        </div>
      </header>
      <main className="container mx-auto px-4 py-8">{children}</main>
    </div>
  );
};

export default MainPanel;
