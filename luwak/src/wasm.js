globalThis.Luwak.wasm = WebAssembly

globalThis.Luwak.wasm.instantiateFetch = async (url) => {
    return await WebAssembly.instantiateStreaming(
        fetch(url),
    );
}

globalThis.Luwak.wasm.compileFetch = async (url) => {
    return await WebAssembly.compileStreaming(
        fetch(url),
    );
}