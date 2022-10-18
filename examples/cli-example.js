// luwak cli.js Luwak Bandeng --version=1.0.2 --no-color

const name = Deno.args[0];
const food = Deno.args[1];
console.log(`Hello ${name}, I like ${food}!`);
import { parse } from "https://deno.land/std@0.119.0/flags/mod.ts";
const flags = parse(Deno.args, {
    boolean: ["help", "color"],
    string: ["version"],
    default: { color: true },
});
console.log("Wants help?", flags.help);
console.log("Version:", flags.version);
console.log("Wants color?:", flags.color);
console.log("Other:", flags._);