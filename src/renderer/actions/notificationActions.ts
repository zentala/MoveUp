// actions/notificationActions.ts
export const SEND_NOTIFICATION = 'SEND_NOTIFICATION';
export const POSTPONE_BREAK = 'POSTPONE_BREAK';
export const SWITCH_TO_STANDING = 'SWITCH_TO_STANDING';

export interface SendNotificationAction {
  type: typeof SEND_NOTIFICATION;
  payload: { message: string };
}

export interface PostponeBreakAction {
  type: typeof POSTPONE_BREAK;
}

export interface SwitchToStandingAction {
  type: typeof SWITCH_TO_STANDING;
}

export type NotificationActions =
  | SendNotificationAction
  | PostponeBreakAction
  | SwitchToStandingAction;

export const sendNotification = (message: string): SendNotificationAction => ({
  type: SEND_NOTIFICATION,
  payload: { message },
});

export const postponeBreak = (): PostponeBreakAction => ({
  type: POSTPONE_BREAK,
});

export const switchToStanding = (): SwitchToStandingAction => ({
  type: SWITCH_TO_STANDING,
});
