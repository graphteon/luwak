const { increment } = await require("http://wpt.live/wasm/incrementer.wasm");

console.log(increment(41));