import type { ChatMessage, CompleteRequest, CompleteResult, ContentPart, LlmProvider } from './provider.ts';
import { ProviderError } from './provider.ts';

export const DEFAULT_MODEL = 'anthropic/claude-sonnet-5';
const DEFAULT_BASE_URL = 'https://openrouter.ai/api/v1';

export type FetchImpl = typeof fetch;

export type CreateOpenRouterProviderOptions = {
  /** Never logged, never placed anywhere but the `Authorization` header. */
  apiKey: string;
  model?: string;
  baseUrl?: string;
  fetchImpl?: FetchImpl;
};

/** OpenAI-compatible chat completions client for OpenRouter. */
export function createOpenRouterProvider(opts: CreateOpenRouterProviderOptions): LlmProvider {
  const model = opts.model ?? DEFAULT_MODEL;
  const baseUrl = opts.baseUrl ?? DEFAULT_BASE_URL;
  const fetchImpl = opts.fetchImpl ?? fetch;
  const apiKey = opts.apiKey;

  return {
    id: 'openrouter',
    model,
    async complete(req: CompleteRequest): Promise<CompleteResult> {
      const body: Record<string, unknown> = {
        model,
        messages: [{ role: 'system', content: req.system }, ...req.messages.map(toOpenAiMessage)],
        usage: { include: true },
      };
      if (req.maxTokens !== undefined) body.max_tokens = req.maxTokens;
      if (req.jsonSchema) {
        body.response_format = {
          type: 'json_schema',
          json_schema: { name: req.jsonSchema.name, strict: true, schema: req.jsonSchema.schema },
        };
      }

      let response: Response;
      try {
        response = await fetchImpl(`${baseUrl}/chat/completions`, {
          method: 'POST',
          headers: {
            'content-type': 'application/json',
            authorization: `Bearer ${apiKey}`,
          },
          body: JSON.stringify(body),
          signal: req.signal,
        });
      } catch (err) {
        if (req.signal?.aborted) {
          throw new ProviderError('abort', 'request aborted', { cause: err });
        }
        throw new ProviderError('network', 'openrouter request failed', { cause: err });
      }

      if (!response.ok) {
        const errorBody = await safeText(response);
        throw new ProviderError('http', `openrouter responded ${response.status}: ${errorBody}`, {
          status: response.status,
        });
      }

      let json: unknown;
      try {
        json = await response.json();
      } catch (err) {
        throw new ProviderError('invalid-response', 'openrouter returned non-JSON body', { cause: err });
      }

      return parseCompletion(json);
    },
  };
}

function toOpenAiMessage(message: ChatMessage): { role: string; content: unknown } {
  if (typeof message.content === 'string') {
    return { role: message.role, content: message.content };
  }
  return { role: message.role, content: message.content.map(toOpenAiContentPart) };
}

function toOpenAiContentPart(part: ContentPart): Record<string, unknown> {
  if (part.type === 'text') return { type: 'text', text: part.text };
  return { type: 'image_url', image_url: { url: `data:image/png;base64,${part.pngBase64}` } };
}

type OpenRouterResponse = {
  choices?: Array<{ message?: { content?: unknown } }>;
  usage?: { prompt_tokens?: number; completion_tokens?: number; cost?: number };
};

function parseCompletion(json: unknown): CompleteResult {
  const data = json as OpenRouterResponse;
  const content = data.choices?.[0]?.message?.content;
  if (typeof content !== 'string') {
    throw new ProviderError('invalid-response', 'openrouter response missing choices[0].message.content');
  }
  return {
    text: content,
    usage: {
      inputTokens: data.usage?.prompt_tokens ?? 0,
      outputTokens: data.usage?.completion_tokens ?? 0,
      costUsd: data.usage?.cost,
    },
  };
}

async function safeText(response: Response): Promise<string> {
  try {
    return await response.text();
  } catch {
    return '<unreadable body>';
  }
}
