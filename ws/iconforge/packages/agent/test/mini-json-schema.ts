/**
 * Minimal JSON Schema (draft-7 subset) validator, just enough to exercise
 * `HAND_WRITTEN_ICON_SPEC_JSON_SCHEMA` in tests without adding an ajv
 * dependency. Supports: $ref/$defs, const, type, properties/required/
 * additionalProperties, items (array or tuple), minItems/maxItems,
 * minLength/maxLength/pattern, minimum/maximum/exclusiveMinimum, not, anyOf.
 */
export function validateMiniSchema(rootSchema: Record<string, unknown>, value: unknown): boolean {
  const defs = (rootSchema.$defs ?? {}) as Record<string, Record<string, unknown>>;

  function resolve(schema: Record<string, unknown>): Record<string, unknown> {
    if (typeof schema.$ref === 'string') {
      const name = schema.$ref.replace('#/$defs/', '');
      const target = defs[name];
      if (!target) throw new Error(`unresolved $ref: ${schema.$ref}`);
      return target;
    }
    return schema;
  }

  function check(schemaIn: Record<string, unknown>, val: unknown): boolean {
    const schema = resolve(schemaIn);

    if ('const' in schema) return val === schema.const;
    if ('not' in schema) return !check(schema.not as Record<string, unknown>, val);
    if (Array.isArray(schema.anyOf)) {
      return (schema.anyOf as Record<string, unknown>[]).some((s) => check(s, val));
    }

    if (schema.type === 'object') {
      if (typeof val !== 'object' || val === null || Array.isArray(val)) return false;
      const obj = val as Record<string, unknown>;
      const props = (schema.properties ?? {}) as Record<string, Record<string, unknown>>;
      const required = (schema.required ?? []) as string[];
      for (const key of required) {
        if (!(key in obj)) return false;
      }
      if (schema.additionalProperties === false) {
        for (const key of Object.keys(obj)) {
          if (!(key in props)) return false;
        }
      }
      for (const [key, propSchema] of Object.entries(props)) {
        if (key in obj && !check(propSchema, obj[key])) return false;
      }
      return true;
    }

    if (schema.type === 'array') {
      if (!Array.isArray(val)) return false;
      if (typeof schema.minItems === 'number' && val.length < schema.minItems) return false;
      if (typeof schema.maxItems === 'number' && val.length > schema.maxItems) return false;
      if (Array.isArray(schema.items)) {
        const tupleItems = schema.items as Record<string, unknown>[];
        return val.every((item, i) => {
          const itemSchema = tupleItems[i];
          return itemSchema ? check(itemSchema, item) : true;
        });
      }
      if (schema.items) {
        return val.every((item) => check(schema.items as Record<string, unknown>, item));
      }
      return true;
    }

    if (schema.type === 'number') {
      if (typeof val !== 'number') return false;
      if (typeof schema.minimum === 'number' && val < schema.minimum) return false;
      if (typeof schema.maximum === 'number' && val > schema.maximum) return false;
      if (typeof schema.exclusiveMinimum === 'number' && val <= schema.exclusiveMinimum) return false;
      return true;
    }

    if (schema.type === 'string') {
      if (typeof val !== 'string') return false;
      if (typeof schema.minLength === 'number' && val.length < schema.minLength) return false;
      if (typeof schema.maxLength === 'number' && val.length > schema.maxLength) return false;
      if (typeof schema.pattern === 'string' && !new RegExp(schema.pattern).test(val)) return false;
      return true;
    }

    if (schema.type === 'boolean') return typeof val === 'boolean';

    return true;
  }

  return check(rootSchema, value);
}
