// import a from './tes.js';
// import { serve } from "https://deno.land/std@0.140.0/http/server.ts";

// console.log(serve)
// window.luwak = Deno;
import { createRequire } from 'https://deno.land/std@0.153.0/node/module.ts';
import { compile } from './coffeescript.js';

window.require = createRequire(import.meta.url);
window.lwk = {
    __script_path : Deno.cwd()+"/"
}

//console.log(lwk.__script_path)
//const luwak_script = await Deno.readTextFile(lwk.__script_path+"../../tests/webserver.luwak");
//const luwak_script = await Deno.readTextFile("./webserver.js");
//console.log(compile(luwak_script));
////eval(compile(luwak_script));

require(lwk.__script_path+'webserver.js');
console.log(lwk.__script_path+'webserver.js');
