import React, { useState, useEffect } from 'react';

const BreakNotification: React.FC = () => {
  const [showPopup, setShowPopup] = useState(false);
  const [timeToBreak, setTimeToBreak] = useState(5); // Minutes until break

  useEffect(() => {
    const timer = setInterval(() => {
      if (timeToBreak > 0) {
        setTimeToBreak(timeToBreak - 1);
      } else {
        setShowPopup(true);
      }
    }, 60000);

    return () => clearInterval(timer);
  }, [timeToBreak]);

  const handleStartBreak = () => {
    // Logic for starting break and awarding points
    setShowPopup(false);
  };

  const handlePostponeBreak = () => {
    // Logic for postponing break
    setShowPopup(false);
  };

  return (
    showPopup && (
      <div className="fixed bottom-0 left-0 right-0 bg-white shadow-lg p-4 m-4 rounded-lg">
        <h2 className="text-xl font-bold">Zbliżająca się przerwa</h2>
        <p>Czas na przerwę! Możesz rozpocząć teraz i zdobyć dodatkowe punkty.</p>
        <div className="mt-4 flex justify-end">
          <button
            onClick={handleStartBreak}
            className="bg-green-500 text-white py-2 px-4 rounded mr-2 hover:bg-green-700"
          >
            Rozpocznij teraz
          </button>
          <button
            onClick={handlePostponeBreak}
            className="bg-gray-500 text-white py-2 px-4 rounded hover:bg-gray-700"
          >
            Przypomnij później
          </button>
        </div>
      </div>
    )
  );
};

export default BreakNotification;
