import { describe, expect, it } from 'vitest';
import { createOpenRouterProvider, DEFAULT_MODEL } from '../src/openrouter.ts';
import { ProviderError } from '../src/provider.ts';

function jsonResponse(body: unknown, status = 200): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { 'content-type': 'application/json' },
  });
}

describe('createOpenRouterProvider', () => {
  it('builds an OpenAI-compatible request and never leaks the key outside the Authorization header', async () => {
    let capturedUrl = '';
    let capturedInit: RequestInit | undefined;
    const fetchImpl = async (url: string | URL | Request, init?: RequestInit) => {
      capturedUrl = String(url);
      capturedInit = init;
      return jsonResponse({
        choices: [{ message: { content: '{"ok":true}' } }],
        usage: { prompt_tokens: 10, completion_tokens: 5, cost: 0.001 },
      });
    };

    const provider = createOpenRouterProvider({ apiKey: 'sk-secret-value', fetchImpl });
    const result = await provider.complete({
      system: 'sys',
      messages: [{ role: 'user', content: 'hello' }],
      jsonSchema: { name: 'Test', schema: { type: 'object' } },
    });

    expect(capturedUrl).toBe('https://openrouter.ai/api/v1/chat/completions');
    expect(provider.model).toBe(DEFAULT_MODEL);
    expect(result.text).toBe('{"ok":true}');
    expect(result.usage).toEqual({ inputTokens: 10, outputTokens: 5, costUsd: 0.001 });

    const headers = capturedInit?.headers as Record<string, string>;
    expect(headers.authorization).toBe('Bearer sk-secret-value');

    const bodyText = capturedInit?.body as string;
    expect(bodyText).not.toContain('sk-secret-value');
    const body = JSON.parse(bodyText) as Record<string, unknown>;
    expect(body.model).toBe(DEFAULT_MODEL);
    expect(body.messages).toEqual([
      { role: 'system', content: 'sys' },
      { role: 'user', content: 'hello' },
    ]);
    expect(body.response_format).toEqual({
      type: 'json_schema',
      json_schema: { name: 'Test', strict: true, schema: { type: 'object' } },
    });
  });

  it('encodes image content parts as data URLs', async () => {
    let capturedBody = '';
    const fetchImpl = async (_url: string | URL | Request, init?: RequestInit) => {
      capturedBody = init?.body as string;
      return jsonResponse({ choices: [{ message: { content: '{}' } }], usage: {} });
    };
    const provider = createOpenRouterProvider({ apiKey: 'k', fetchImpl });
    await provider.complete({
      system: 'sys',
      messages: [
        {
          role: 'user',
          content: [
            { type: 'text', text: 'look' },
            { type: 'image', pngBase64: 'AAAA' },
          ],
        },
      ],
    });
    const body = JSON.parse(capturedBody) as { messages: Array<{ content: unknown }> };
    expect(body.messages[1]?.content).toEqual([
      { type: 'text', text: 'look' },
      { type: 'image_url', image_url: { url: 'data:image/png;base64,AAAA' } },
    ]);
  });

  it('throws a typed ProviderError on a non-2xx response', async () => {
    const fetchImpl = async () => jsonResponse({ error: 'bad key' }, 401);
    const provider = createOpenRouterProvider({ apiKey: 'k', fetchImpl });
    await expect(provider.complete({ system: 's', messages: [{ role: 'user', content: 'hi' }] })).rejects.toThrow(
      ProviderError,
    );
  });

  it('throws a typed ProviderError when the response has no message content', async () => {
    const fetchImpl = async () => jsonResponse({ choices: [{ message: {} }] });
    const provider = createOpenRouterProvider({ apiKey: 'k', fetchImpl });
    await expect(provider.complete({ system: 's', messages: [{ role: 'user', content: 'hi' }] })).rejects.toThrow(
      ProviderError,
    );
  });

  it('uses a custom baseUrl and model when given', async () => {
    let capturedUrl = '';
    const fetchImpl = async (url: string | URL | Request) => {
      capturedUrl = String(url);
      return jsonResponse({ choices: [{ message: { content: '{}' } }], usage: {} });
    };
    const provider = createOpenRouterProvider({
      apiKey: 'k',
      fetchImpl,
      baseUrl: 'https://example.internal/v1',
      model: 'some/model',
    });
    expect(provider.model).toBe('some/model');
    await provider.complete({ system: 's', messages: [{ role: 'user', content: 'hi' }] });
    expect(capturedUrl).toBe('https://example.internal/v1/chat/completions');
  });
});
