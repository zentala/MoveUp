import React from 'react';

const Settings: React.FC = () => {
  return (
    <div className="p-4">
      <h1 className="text-3xl font-bold mb-4">Ustawienia</h1>

      <div className="bg-white shadow rounded-lg p-4">
        <h2 className="text-xl font-semibold">Preferencje przerw</h2>

        <label className="block mt-4">
          <span className="text-gray-700">Częstotliwość przerw</span>
          <select className="mt-1 block w-full">
            <option>Co 30 minut</option>
            <option>Co 45 minut</option>
            <option>Co 60 minut</option>
          </select>
        </label>

        <label className="block mt-4">
          <span className="text-gray-700">Czas trwania przerwy</span>
          <select className="mt-1 block w-full">
            <option>5 minut</option>
            <option>10 minut</option>
            <option>15 minut</option>
          </select>
        </label>

        <label className="block mt-4">
          <span className="text-gray-700">Rodzaj aktywności</span>
          <select className="mt-1 block w-full">
            <option>Ćwiczenia rozciągające</option>
            <option>Ćwiczenia oddechowe</option>
            <option>Aktywność fizyczna</option>
          </select>
        </label>

        <button className="mt-4 bg-blue-500 text-white py-2 px-4 rounded hover:bg-blue-700">
          Zapisz ustawienia
        </button>
      </div>
    </div>
  );
};

export default Settings;
