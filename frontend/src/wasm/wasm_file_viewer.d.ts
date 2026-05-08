/* tslint:disable */
/* eslint-disable */

export class FileViewer {
    free(): void;
    [Symbol.dispose](): void;
    get_file_info(): string;
    load_content(data: Uint8Array): void;
    constructor(file_type: string);
    render(): string;
}

export function detect_file_type(filename: string): string;

export function greet(name: string): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_fileviewer_free: (a: number, b: number) => void;
    readonly detect_file_type: (a: number, b: number) => [number, number];
    readonly fileviewer_get_file_info: (a: number) => [number, number];
    readonly fileviewer_load_content: (a: number, b: number, c: number) => void;
    readonly fileviewer_new: (a: number, b: number) => number;
    readonly fileviewer_render: (a: number) => [number, number];
    readonly greet: (a: number, b: number) => [number, number];
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
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
