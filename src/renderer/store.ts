import { configureStore } from '@reduxjs/toolkit';
import notificationReducer from './reducers/notificationReducer';
import userPreferencesReducer from './reducers/userPreferencesReducer';

const store = configureStore({
  reducer: {
    notifications: notificationReducer,
    userPreferences: userPreferencesReducer,
  },
  devTools: process.env.NODE_ENV !== 'production', // turn on Redux DevTools in dev mode only
});

export default store;
export type RootState = ReturnType<typeof store.getState>;
export type AppDispatch = typeof store.dispatch;
