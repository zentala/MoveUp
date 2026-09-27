import { describe, expect, it } from 'vitest';
import { IconSpecSchema } from '@iconforge/schema';
import { iconSpecJsonSchema, HAND_WRITTEN_ICON_SPEC_JSON_SCHEMA } from '../src/jsonSchema.ts';
import { validateMiniSchema } from './mini-json-schema.ts';
import { validSpec, invalidSpec } from './fixtures.ts';

describe('iconSpecJsonSchema', () => {
  it('produces a JSON schema object', () => {
    const schema = iconSpecJsonSchema();
    expect(typeof schema).toBe('object');
  });
});

describe('hand-written schema is equivalent to the zod schema on sample specs', () => {
  const validateHandWritten = (value: unknown): boolean =>
    validateMiniSchema(HAND_WRITTEN_ICON_SPEC_JSON_SCHEMA as unknown as Record<string, unknown>, value);

  it('accepts everything the zod schema accepts', () => {
    const zodOk = IconSpecSchema.safeParse(validSpec).success;
    const handWrittenOk = validateHandWritten(validSpec);
    expect(zodOk).toBe(true);
    expect(handWrittenOk).toBe(true);
  });

  it('rejects what the zod schema rejects (negative radius)', () => {
    const zodOk = IconSpecSchema.safeParse(invalidSpec).success;
    const handWrittenOk = validateHandWritten(invalidSpec);
    expect(zodOk).toBe(false);
    expect(handWrittenOk).toBe(false);
  });

  it('accepts a nested group', () => {
    const withGroup = {
      version: 1,
      name: 'grouped',
      profile: 'outline-24-v1',
      shapes: [
        {
          id: 'g1',
          type: 'group',
          children: [{ id: 'c1', type: 'circle', center: [1, 1], radius: 2 }],
          translate: [1, 1],
        },
      ],
    };
    expect(IconSpecSchema.safeParse(withGroup).success).toBe(true);
    expect(validateHandWritten(withGroup)).toBe(true);
  });

  it('rejects an unknown property (strict)', () => {
    const withExtra = { ...validSpec, extra: 'nope' };
    expect(IconSpecSchema.safeParse(withExtra).success).toBe(false);
    expect(validateHandWritten(withExtra)).toBe(false);
  });
});
