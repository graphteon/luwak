const { res, Server } = Luwak.http;
const server = new Server();

server.get(
    "/",
    res("html"),
    async (ctx, next) => {
        const decoder = new TextDecoder("utf-8");
        const html = await Luwak.readFile("./vue.html");
        ctx.res.body = decoder.decode(html);
        await next();
    },
);

console.log(`Listen port to http://localhost:4000`)
await server.listen({ port: 4000 });