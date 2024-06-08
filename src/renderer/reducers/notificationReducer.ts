// reducers/notificationReducer.ts
import { createSlice, PayloadAction } from '@reduxjs/toolkit';

interface NotificationState {
  message: string;
  postponed: boolean;
  standing: boolean;
}

const initialState: NotificationState = {
  message: '',
  postponed: false,
  standing: false,
};

const notificationSlice = createSlice({
  name: 'notifications',
  initialState,
  reducers: {
    sendNotification: (state, action: PayloadAction<{ message: string }>) => {
      state.message = action.payload.message;
      state.postponed = false;
      state.standing = false;
    },
    postponeBreak: (state) => {
      state.postponed = true;
    },
    switchToStanding: (state) => {
      state.standing = true;
    },
  },
});

export const { sendNotification, postponeBreak, switchToStanding } =
  notificationSlice.actions;
export default notificationSlice.reducer;
