const { res, Server } = Luwak.http;
import { serveStatic } from "https://deno.land/x/kilat@1.2.0/middlewares/serve_static.ts";
const server = new Server();

server.get(
    "/*",
    serveStatic("./assets"),
);

server.get(
    "/",
    res("html"),
    async (ctx, next) => {
        const decoder = new TextDecoder("utf-8");
        const html = await Luwak.readFile("./assets/index.html");
        ctx.res.body = decoder.decode(html);
        await next();
    },
);

console.log(`Listen port to http://localhost:4000`)
await server.listen({ port: 4000 });