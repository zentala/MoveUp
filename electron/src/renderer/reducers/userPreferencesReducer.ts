// reducers/userPreferencesReducer.ts
import { createSlice, PayloadAction } from '@reduxjs/toolkit';

interface UserPreferencesState {
  breakInterval: number;
  deskPosition: 'sitting' | 'standing';
}

const initialState: UserPreferencesState = {
  breakInterval: 45, // default interval of break in minutes
  deskPosition: 'sitting',
};

const userPreferencesSlice = createSlice({
  name: 'userPreferences',
  initialState,
  reducers: {
    setUserPreferences: (
      state,
      action: PayloadAction<{ breakInterval: number }>
    ) => {
      state.breakInterval = action.payload.breakInterval;
    },
    updateDeskPosition: (
      state,
      action: PayloadAction<{ position: 'sitting' | 'standing' }>
    ) => {
      state.deskPosition = action.payload.position;
    },
  },
});

export const { setUserPreferences, updateDeskPosition } =
  userPreferencesSlice.actions;
export default userPreferencesSlice.reducer;
