/* tslint:disable */
/* eslint-disable */

export function add_media(file: Uint8Array): string;

export function add_template(file_data: Uint8Array, is_decode: boolean): Promise<number>;

export function extract_medias(files: Uint8Array[], is_decode: boolean): Promise<any>;

export function extract_one_file_medias(data: Uint8Array, is_decode: boolean): Promise<any>;

export function extract_one_file_variable_names(data: Uint8Array, is_decode: boolean): Promise<string[]>;

export function extract_variable_names(files: Uint8Array[], is_decode: boolean): Promise<string[]>;

export function file_encrypt(file: Uint8Array): Uint8Array;

export function files_encrypt(files: Uint8Array[]): Uint8Array[];

export function main(): void;

export function replace(params: any, medias: Uint8Array[]): Promise<Uint8Array[]>;

export function replace_batch(params: any, medias: Uint8Array[], files: Uint8Array[], is_decode: boolean): Promise<Uint8Array[]>;

export function replace_batch_multiple_params(params: any, medias: Uint8Array[], files: Uint8Array[], is_decode: boolean): Promise<Uint8Array[]>;

export function replace_batch_multiple_params_to_zip(params: any, medias: Uint8Array[], files: Uint8Array[], file_names: string[], is_decode: boolean): Promise<Uint8Array>;

export function replace_batch_to_zip(params: any, medias: Uint8Array[], files: Uint8Array[], file_names: string[], is_decode: boolean): Promise<Uint8Array>;

export function replace_item(variables: any, medias: Uint8Array[], file: Uint8Array, is_decode: boolean): Promise<Uint8Array>;

export function replace_multiple_params(params: any, medias: Uint8Array[]): Promise<Uint8Array[]>;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly add_media: (a: any) => [number, number];
  readonly add_template: (a: any, b: number) => any;
  readonly extract_medias: (a: number, b: number, c: number) => any;
  readonly extract_one_file_medias: (a: any, b: number) => any;
  readonly extract_one_file_variable_names: (a: any, b: number) => any;
  readonly extract_variable_names: (a: number, b: number, c: number) => any;
  readonly file_encrypt: (a: any) => [number, number];
  readonly files_encrypt: (a: number, b: number) => [number, number];
  readonly main: () => void;
  readonly replace: (a: any, b: number, c: number) => any;
  readonly replace_batch: (a: any, b: number, c: number, d: number, e: number, f: number) => any;
  readonly replace_batch_multiple_params: (a: any, b: number, c: number, d: number, e: number, f: number) => any;
  readonly replace_batch_multiple_params_to_zip: (a: any, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => any;
  readonly replace_batch_to_zip: (a: any, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => any;
  readonly replace_item: (a: any, b: number, c: number, d: any, e: number) => any;
  readonly replace_multiple_params: (a: any, b: number, c: number) => any;
  readonly wasm_bindgen__convert__closures_____invoke__h5c763851625a2d40: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__closure__destroy__hd716590baeba421e: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__h19e80470befc6792: (a: number, b: number, c: any, d: any) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __externref_table_alloc: () => number;
  readonly __wbindgen_externrefs: WebAssembly.Table;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __externref_drop_slice: (a: number, b: number) => void;
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
