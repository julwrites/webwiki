/* tslint:disable */
/* eslint-disable */

export function run_app(): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly run_app: () => void;
  readonly wasm_bindgen__convert__closures_____invoke__h15b2444e7b8e56c7: (a: number, b: number, c: number, d: number) => void;
  readonly wasm_bindgen__closure__destroy__h180c9f57214c293d: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures________invoke__h9ae29008818b72a8: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__closure__destroy__h463abdf7ef87ba73: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures________invoke__h5e5f7c3d1adb0ebe: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__closure__destroy__h43a3fcf5afbf121c: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__h20d3225642fb054c: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__closure__destroy__h6319c344c083c653: (a: number, b: number) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __externref_table_alloc: () => number;
  readonly __wbindgen_externrefs: WebAssembly.Table;
  readonly __externref_drop_slice: (a: number, b: number) => void;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
