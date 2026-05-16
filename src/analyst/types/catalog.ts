/**
 * catalog.ts — TypeScript types mirroring the Rust `DataCatalog` payload
 * returned by the `get_data_catalog` Tauri command (E012-T02).
 *
 * Wire shape matches `commands_catalog.rs`:
 * - `FieldMeta.ty` is serialized as `type` (serde rename).
 */

/** One field on a data source row. */
export interface FieldMeta {
  name: string;
  /** Field type label, e.g. `"u16"`, `"ISO 8601"`, `"INTEGER PK"`. */
  type: string;
  description: string;
}

/** Static description of a single data source. */
export interface DataSource {
  id: string;
  name: string;
  kind: string;
  location: string;
  retention: string;
  fields: FieldMeta[];
  sample_row: string | null;
  description: string;
}

/** Top-level catalog payload returned by `get_data_catalog`. */
export interface DataCatalog {
  sources: DataSource[];
  generated_at: string;
}
