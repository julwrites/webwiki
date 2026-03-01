/* tslint:disable */
/* eslint-disable */

export function run_app(): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly run_app: () => void;
  readonly wasm_bindgen__convert__closures_____invoke__hfb8e9ff15e66888b: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__closure__destroy__h2d156ade04bd47e8: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures________invoke__h1b15310a5b8bba89: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__closure__destroy__h46fed59730eeebeb: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__hcf6a9c3fa96216ae: (a: number, b: number) => void;
  readonly wasm_bindgen__closure__destroy__hb4c5d395d9842517: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__hb4349d2e4aac8739: (a: number, b: number, c: number, d: number) => void;
  readonly wasm_bindgen__convert__closures________invoke__hf6567d3ca51ffc05: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__closure__destroy__h3d82f797e97a8bd6: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__h3956896e0e16973c: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__closure__destroy__h5b486963ff1a3b4d: (a: number, b: number) => void;
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
