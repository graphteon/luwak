globalThis.Luwak = () => {
    return Deno;
}

globalThis.Luwak.version = "0.8.0";

globalThis.require = async (package_name, exports = null) => {
    const pkg = !exports ? `npm:${package_name}` : exports.length == 0 ? `esm:${package_name}` : `esm:${package_name}?exports=${exports.join(",")}`;
    const module = await import(pkg);
    if (exports) {
        return module;
    }
    return module.default;
}
