// import a from './tes.js';
// import { serve } from "https://deno.land/std@0.140.0/http/server.ts";

// console.log(serve)
// window.luwak = Deno;
// import { createRequire } from 'https://deno.land/std@0.153.0/node/module.ts';
// import { compile } from './parser.js';

// window.require = createRequire(import.meta.url);
// window.lwk = {
//     __script_path : Deno.cwd()+"/"
// }

//console.log(lwk.__script_path)
//const luwak_script = await Deno.readTextFile(lwk.__script_path+"../../tests/webserver.luwak");
//const luwak_script = await Deno.readTextFile("./webserver.js");
//console.log(compile(luwak_script));
////eval(compile(luwak_script));

//const a = require(lwk.__script_path+'webserver.js');
// import a from '/Volumes/git/MAYAR/luwak/target/debug/webserver.js';
// a();
// console.log(lwk.__script_path+'webserver.js');

//console.log(compile(luwak_script));


import md5 from 'node://md5@2.3.0'
console.log("hasing result test123 : ",md5('tes123'));