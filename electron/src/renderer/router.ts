import React from 'react';
import { Routes, Route } from 'react-router-dom'; 
import Dashboard from './views/Dashboard';
import Settings from './views/Settings';

const RoutesComponent: React.FC = () => {
  return (
    <Routes>
      <Route path="/" element={<Dashboard />} />
      <Route path="/about" element={<Settings />} />
    </Routes>
  );
};

export default RoutesComponent;