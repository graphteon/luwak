import { createRequire } from "https://deno.land/std@0.150.0/node/module.ts";

// import.meta.url will be the location of "this" module (like `__filename` in
// Node), and then serve as the root for your "package", where the
// `package.json` is expected to be, and where the `node_modules` will be used
// for resolution of packages.
const require = createRequire(import.meta.url);

// Loads the built-in module Deno "replacement":
const path = require("path");
console.log(path);

// // Loads a CommonJS module (without specifying the extension):
// const cjsModule = require("./my_mod");

// // Uses Node resolution in `node_modules` to load the package/module. The
// // package would need to be installed locally via a package management tool
// // like npm:
const leftPad = require("left-pad");