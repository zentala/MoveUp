import React from 'react';

const Statistics: React.FC = () => {
  return (
    <div className="p-4">
      <h1 className="text-3xl font-bold mb-4">Statystyki</h1>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        <div className="bg-white shadow rounded-lg p-4">
          <h2 className="text-xl font-semibold">Czas przy biurku</h2>
          <p>Dziś: 5 godzin</p>
          <p>W tym tygodniu: 25 godzin</p>
          <p>W tym miesiącu: 100 godzin</p>
        </div>

        <div className="bg-white shadow rounded-lg p-4">
          <h2 className="text-xl font-semibold">Liczba przerw</h2>
          <p>Dziś: 3 przerwy</p>
          <p>W tym tygodniu: 15 przerw</p>
          <p>W tym miesiącu: 60 przerw</p>
        </div>

        <div className="bg-white shadow rounded-lg p-4">
          <h2 className="text-xl font-semibold">Punkty zdobyte</h2>
          <p>Dziś: 30 punktów</p>
          <p>W tym tygodniu: 150 punktów</p>
          <p>W tym miesiącu: 600 punktów</p>
        </div>

        <div className="bg-white shadow rounded-lg p-4">
          <h2 className="text-xl font-semibold">Aktywność fizyczna</h2>
          <p>Dziś: 20 minut</p>
          <p>W tym tygodniu: 100 minut</p>
          <p>W tym miesiącu: 400 minut</p>
        </div>

        <div className="bg-white shadow rounded-lg p-4">
          <h2 className="text-xl font-semibold">Postawa ciała</h2>
          <p>Czas spędzony na siedząco: 4 godziny</p>
          <p>Czas spędzony na stojąco: 1 godzina</p>
          <p>Alerty postawy: 2 razy</p>
        </div>

        <div className="bg-white shadow rounded-lg p-4">
          <h2 className="text-xl font-semibold">Odznaki zdobyte</h2>
          <p>Zdobyte odznaki: 3</p>
          <ul className="list-disc pl-5">
            <li>Odznaka regularnych przerw</li>
            <li>Odznaka aktywności fizycznej</li>
            <li>Odznaka postawy</li>
          </ul>
        </div>
      </div>
    </div>
  );
};

export default Statistics;
