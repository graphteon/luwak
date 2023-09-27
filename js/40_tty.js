// Copyright 2018-2023 the Deno authors. All rights reserved. MIT license.
const core = globalThis.Deno.core;
const ops = core.ops;
const primordials = globalThis.__bootstrap.primordials;
const {
  Uint32Array,
} = primordials;

const size = new Uint32Array(2);

function consoleSize() {
  ops.op_console_size(size);
  return { columns: size[0], rows: size[1] };
}

function isatty(rid) {
  return ops.op_isatty(rid);
}

export { consoleSize, isatty };
