/* tslint:disable */
/* eslint-disable */

export function run_app(): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly run_app: () => void;
  readonly wasm_bindgen__convert__closures_____invoke__h7eb13b628d6f135c: (a: number, b: number, c: number, d: number) => void;
  readonly wasm_bindgen__closure__destroy__h4b3401d425942968: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures________invoke__h9998accb5ccd4ea4: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__closure__destroy__hb11f0c3b24c0911d: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__h8fe8a4667a793b67: (a: number, b: number) => void;
  readonly wasm_bindgen__closure__destroy__h08783ef05dcd2154: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__h17fcd7d2f979b1da: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__closure__destroy__h64fb2488f2fd2c61: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__hc2b9222b6d70a84e: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__convert__closures________invoke__h5163ee76bf6765aa: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__closure__destroy__h3690d7a922754f03: (a: number, b: number) => void;
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
