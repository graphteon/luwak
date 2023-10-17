globalThis.Luwak = {
    readFile : async (path) => {
        return await Deno.readFile(path);
    },
    version : "0.8.0"
}

