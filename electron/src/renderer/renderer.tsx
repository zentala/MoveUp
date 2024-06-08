import React, { useEffect } from 'react';
import ReactDOM from 'react-dom';

const App = () => {
  useEffect(() => {
    window.api.receive('update-icon', (color: string) => {
      const iconElement = document.getElementById('icon');
      if (iconElement) {
        iconElement.src = `icon-${color}.png`;
      }
    });
  }, []);

  return (
    <div>
      <h1 className="text-3xl font-bold underline">Hello MoveUp!</h1>
      <img id="icon" src="icon-black.png" alt="Icon" width="16" height="16" />
      <button
        id="triggerFloatingWindow"
        className="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded"
        onClick={() => window.api.send('show-floating-window', null)}
      >
        Show Floating Window
      </button>
    </div>
  );
};

ReactDOM.render(<App />, document.getElementById('root'));
