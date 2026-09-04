export {};

declare global {
  interface Window {
    api: {
      receive(channel: string, callback: (value: string) => void): void;
      send(channel: string, value: unknown): void;
    };
  }
}
