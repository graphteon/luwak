globalThis.Luwak = () => {
    return Deno;
}

globalThis.Luwak.version = "0.8.0";

globalThis.require = async (package_name, exports = null) => {
    const pkg = exports ? `npm:${package_name}?exports=${exports.join(",")}` : `npm:${package_name}`;
    const module = await import(pkg);
    if (exports) {
        return module;
    }
    return module.default;
}

globalThis.npmExports = (package_name, exports) => {
    const pkg = exports ? `npm:${package_name}?exports=${exports.join(",")}` : `npm:${package_name}`;
    return pkg;
}
