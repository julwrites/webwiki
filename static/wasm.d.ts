/* tslint:disable */
/* eslint-disable */

export function run_app(): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly run_app: () => void;
  readonly wasm_bindgen__convert__closures_____invoke__h971ec49a09f9498a: (a: number, b: number) => void;
  readonly wasm_bindgen__closure__destroy__hed65fadbfbafbb37: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__hd0a29bbbd816fc41: (a: number, b: number, c: number, d: number) => void;
  readonly wasm_bindgen__closure__destroy__h153ccc5aad8100b0: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__hbfb304059146b5e4: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__closure__destroy__hbe11236d12c79034: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__h0d087d75486c44eb: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__closure__destroy__h0ca519d0e23f4498: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures________invoke__h9a5dd0421f429784: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__closure__destroy__h51e38b399e44457b: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures________invoke__h68b605264039272b: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__closure__destroy__h2e50f12bedbee129: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__ha3d30e5f1f4be086: (a: number, b: number) => number;
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
