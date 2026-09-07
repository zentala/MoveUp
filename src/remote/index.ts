/**
 * Public surface of the remote-display client (E022).
 *
 * T01 ships the protocol only; transports (`transports/`), the pairing screen
 * and the stored-record helpers land in T08 and T09 behind this barrel.
 */
export * from "./protocol";
