import type { Engine } from "./engine";

interface DylinkInfo {
  memorySize: number;
  memoryAlign: number;
  tableSize: number;
  tableAlign: number;
}

const decoder = new TextDecoder();

// A relocatable grammar records the data and table space it needs in its `dylink.0` section.
function dylinkInfo(bytes: Uint8Array): DylinkInfo {
  let offset = 8;
  const leb = () => {
    let result = 0;
    let shift = 0;
    let byte: number;
    do {
      byte = bytes[offset++];
      result |= (byte & 0x7f) << shift;
      shift += 7;
    } while (byte & 0x80);
    return result >>> 0;
  };
  while (offset < bytes.length) {
    const id = bytes[offset++];
    const end = leb() + offset;
    if (id === 0) {
      const length = leb();
      const name = decoder.decode(bytes.subarray(offset, offset + length));
      offset += length;
      while (name === "dylink.0" && offset < end) {
        const kind = bytes[offset++];
        const size = leb();
        if (kind === 1)
          return { memorySize: leb(), memoryAlign: leb(), tableSize: leb(), tableAlign: leb() };
        offset += size;
      }
    }
    offset = end;
  }
  throw new Error("Grammar is not a relocatable WebAssembly module");
}

// A JavaScript exception unwinding through Wasm skips its cleanup, so a grammar's abort counts as a trap.
function trap(message: string) {
  return () => {
    throw new WebAssembly.RuntimeError(message);
  };
}

const linkage = new Map<string, WebAssembly.ImportExportKind>([
  ["memory", "memory"],
  ["__indirect_function_table", "table"],
  ["__stack_pointer", "global"],
  ["__memory_base", "global"],
  ["__table_base", "global"],
]);
const builtins = new Map([
  ["abort", trap("Grammar aborted")],
  ["__assert_fail", trap("Grammar assertion failed")],
]);

function own(exports: object, name: string): unknown {
  return Object.hasOwn(exports, name) ? (exports as Record<string, unknown>)[name] : undefined;
}

export interface CompiledGrammar {
  module: WebAssembly.Module;
  info: DylinkInfo;
  language: string;
}

export async function compileGrammar(bytes: ArrayBuffer): Promise<CompiledGrammar> {
  const module = await WebAssembly.compile(bytes);
  const info = dylinkInfo(new Uint8Array(bytes));
  const languages = WebAssembly.Module.exports(module).filter(({ name }) =>
    name.startsWith("tree_sitter_"),
  );
  if (languages.length !== 1 || languages[0].kind !== "function")
    throw new Error("Grammar does not export exactly one language function");
  return { module, info, language: languages[0].name };
}

// Links a grammar into the engine's memory and function table as tree-sitter's own Wasm store does, returning the table index of its language function; everything is checked before the engine allocates anything.
export function linkGrammar(
  engine: Engine,
  { module, info, language }: CompiledGrammar,
  libc: readonly string[],
): number {
  const exports: unknown = engine.instance_exports();
  const table: unknown = engine.function_table();
  if (typeof exports !== "object" || exports === null || !(table instanceof WebAssembly.Table))
    throw new Error("The engine does not export its instance and function table");
  const memory = own(exports, "memory");
  const stackPointer = own(exports, "__stack_pointer");
  const calloc = own(exports, "calloc");
  if (
    !(memory instanceof WebAssembly.Memory) ||
    !(stackPointer instanceof WebAssembly.Global) ||
    typeof calloc !== "function"
  )
    throw new Error("The engine does not export its memory, stack pointer, and allocator");
  const functions = new Map<string, unknown>(builtins);
  for (const name of libc) functions.set(name, own(exports, name));
  const env: Record<string, WebAssembly.ImportValue> = Object.create(null);
  for (const { module: from, name, kind } of WebAssembly.Module.imports(module)) {
    const value = functions.get(name);
    if (from === "env" && kind === "function" && typeof value === "function") env[name] = value;
    else if (from !== "env" || kind !== linkage.get(name))
      throw new Error(`Grammar imports unsupported ${kind} ${from}.${name}`);
  }

  const memoryAlign = 2 ** info.memoryAlign;
  const block: unknown = calloc(info.memorySize + memoryAlign, 1);
  if (typeof block !== "number" || !block)
    throw new Error(`Could not allocate ${info.memorySize} bytes for a grammar`);
  const tableAlign = 2 ** info.tableAlign;
  const padding = (tableAlign - (table.length % tableAlign)) % tableAlign;
  const tableBase = table.grow(padding + info.tableSize) + padding;
  const constant = (value: number) =>
    new WebAssembly.Global({ value: "i32", mutable: false }, value);
  Object.assign(env, {
    memory,
    __indirect_function_table: table,
    __stack_pointer: stackPointer,
    __memory_base: constant(Math.ceil(block / memoryAlign) * memoryAlign),
    __table_base: constant(tableBase),
  });
  const grammar = new WebAssembly.Instance(module, { env }).exports;
  for (const initializer of ["__wasm_apply_data_relocs", "_initialize", "__wasm_call_ctors"]) {
    const run = own(grammar, initializer);
    if (typeof run === "function") run();
  }
  const languageFn = own(grammar, language);
  if (typeof languageFn !== "function") throw new Error("Grammar language function is missing");
  return table.grow(1, languageFn);
}
