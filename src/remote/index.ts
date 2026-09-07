/**
 * Public surface of the remote-display client (E022).
 *
 * T01 shipped the protocol, T08 the transports and the stored pairing record,
 * T09 the pairing screen and the controls.
 */
export * from "./protocol";
export * from "./storage";
export * from "./transports";
export * from "./activeTransport";
export * from "./pairing";
export * from "./useRemoteCommand";
export { PairScreen } from "./PairScreen";
export type { PairScreenProps } from "./PairScreen";
export { RemoteControls } from "./RemoteControls";
export type { RemoteControlsProps } from "./RemoteControls";
