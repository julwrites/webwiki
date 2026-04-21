/* tslint:disable */
/* eslint-disable */

export function run_app(): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly run_app: () => void;
  readonly wasm_bindgen__convert__closures_____invoke__hb0fd9645aba06dd3: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__closure__destroy__h78b36ddcf98d2cc8: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__he8de7f5d44814975: (a: number, b: number) => void;
  readonly wasm_bindgen__closure__destroy__h24bb95eb630d0d83: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__h17fcd7d2f979b1da: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__closure__destroy__h64fb2488f2fd2c61: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures________invoke__h65e4e360001e3b5a: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__closure__destroy__h1f1196ed01ac55b4: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures________invoke__h41d094e7fab80de6: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__closure__destroy__h4424386b268fc354: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__h080a36dd223d0102: (a: number, b: number, c: number, d: number) => void;
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
