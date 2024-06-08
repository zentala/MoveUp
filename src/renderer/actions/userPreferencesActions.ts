// actions/userPreferencesActions.ts
export const SET_USER_PREFERENCES = 'SET_USER_PREFERENCES';
export const UPDATE_DESK_POSITION = 'UPDATE_DESK_POSITION';

export interface SetUserPreferencesAction {
  type: typeof SET_USER_PREFERENCES;
  payload: { breakInterval: number };
}

export interface UpdateDeskPositionAction {
  type: typeof UPDATE_DESK_POSITION;
  payload: { position: 'sitting' | 'standing' };
}

export type UserPreferencesActions = SetUserPreferencesAction | UpdateDeskPositionAction;

export const setUserPreferences = (breakInterval: number): SetUserPreferencesAction => ({
  type: SET_USER_PREFERENCES,
  payload: { breakInterval },
});

export const updateDeskPosition = (position: 'sitting' | 'standing'): UpdateDeskPositionAction => ({
  type: UPDATE_DESK_POSITION,
  payload: { position },
});
