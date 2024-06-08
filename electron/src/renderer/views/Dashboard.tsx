import React from 'react';

const Dashboard: React.FC = () => {
  return (
    <div className="p-4">
      <h1 className="text-3xl font-bold mb-4">Dashboard</h1>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        <div className="bg-white shadow rounded-lg p-4">
          <h2 className="text-xl font-semibold">Aktywność</h2>
          <p>Czas przy biurku: 40 minut</p>
          <p>Zbliżająca się przerwa za: 5 minut</p>
          <p>Punkty: 120</p>
        </div>

        <div className="bg-white shadow rounded-lg p-4">
          <h2 className="text-xl font-semibold">Gamifikacja</h2>
          <p>Punkty za regularne przerwy: 15</p>
          <p>Punkty za aktywność fizyczną: 30</p>
          <p>Odznaki zdobyte: 3</p>
        </div>

        <div className="bg-white shadow rounded-lg p-4">
          <h2 className="text-xl font-semibold">Następna przerwa</h2>
          <p>Typ: Ćwiczenia rozciągające</p>
          <p>Czas trwania: 15 minut</p>
          <button className="mt-2 bg-blue-500 text-white py-2 px-4 rounded hover:bg-blue-700">
            Rozpocznij teraz
          </button>
        </div>
      </div>
    </div>
  );
};

export default Dashboard;
