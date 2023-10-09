const module = await WebAssembly.compileStreaming(
    fetch("http://wpt.live/wasm/incrementer.wasm"),
  );
  
  /* do some more stuff */
  
  const instance = await WebAssembly.instantiate(module);
  instance.exports.increment as (input: number) => number;