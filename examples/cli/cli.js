// luwak cli.js Luwak Bandeng --version=1.0.2 --no-color

const { flags, args } = Luwak.cli;
console.log("Print Arguments :", args());

const cli = flags({
    boolean: ["help", "color"],
    string: ["version"],
    default: { color: true },
});

console.log("Wants help?", cli.help);
console.log("Version:", cli.version);
console.log("Wants color?:", cli.color);
console.log("Other:", cli._);