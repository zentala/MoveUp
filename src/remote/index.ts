/**
 * Public surface of the remote-display client (E022).
 *
 * T01 shipped the protocol, T08 the transports and the stored pairing record;
 * the pairing screen and controls land in T09 behind this barrel.
 */
export * from "./protocol";
export * from "./storage";
export * from "./transports";
