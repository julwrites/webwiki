/* tslint:disable */
/* eslint-disable */

export function run_app(): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly run_app: () => void;
  readonly wasm_bindgen__convert__closures________invoke__h1bb4a704bd98f2a7: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__closure__destroy__h584e270892e68b01: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures________invoke__h5a02b94f75529f50: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__closure__destroy__h77083a61e335ca33: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__h2d7e93e509660f56: (a: number, b: number, c: number, d: number) => void;
  readonly wasm_bindgen__closure__destroy__h55395d7854e982be: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__h512dfaa8e4142f50: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__convert__closures_____invoke__h6985d67e3f7f79ac: (a: number, b: number) => void;
  readonly wasm_bindgen__closure__destroy__hba4e46a3f1e2656d: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__hff39e61eab70f7f9: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__closure__destroy__h36e275eb0ef83d33: (a: number, b: number) => void;
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
