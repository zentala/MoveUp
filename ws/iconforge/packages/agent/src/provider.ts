/** Provider-agnostic LLM adapter contract. Never send raw model text anywhere but through parseIconSpec. */

export type ContentPart = { type: 'text'; text: string } | { type: 'image'; pngBase64: string };

export type ChatMessage = {
  role: 'user' | 'assistant';
  content: string | ContentPart[];
};

export type JsonSchemaRequest = {
  name: string;
  schema: object;
};

export type CompleteRequest = {
  system: string;
  messages: ChatMessage[];
  jsonSchema?: JsonSchemaRequest;
  maxTokens?: number;
  signal?: AbortSignal;
};

export type Usage = {
  inputTokens: number;
  outputTokens: number;
  costUsd?: number;
};

export type CompleteResult = {
  text: string;
  usage: Usage;
};

export interface LlmProvider {
  id: string;
  model: string;
  complete(req: CompleteRequest): Promise<CompleteResult>;
}

export type ProviderErrorKind = 'network' | 'http' | 'timeout' | 'abort' | 'invalid-response';

export class ProviderError extends Error {
  readonly kind: ProviderErrorKind;
  readonly status?: number;

  constructor(kind: ProviderErrorKind, message: string, opts?: { status?: number; cause?: unknown }) {
    super(message, opts?.cause !== undefined ? { cause: opts.cause } : undefined);
    this.name = 'ProviderError';
    this.kind = kind;
    this.status = opts?.status;
  }
}
