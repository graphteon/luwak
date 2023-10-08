const { req, res, Server } = Luwak.http;
const server = new Server();
server.post(
    "/example_json",
    res("json"),
    req("json"),
    async (ctx: any, next: any) => {
        console.log(ctx.body);
        ctx.res.body = { msg: "json response example" };
        await next();
    },
);
server.get(
    "/",
    res("html"),
    async (ctx: any, next: any) => {
        ctx.res.body = `
        <!DOCTYPE html>
        <html>
          <head>
            <meta charset="utf-8">
            <title>title example</title>
          </head>
          </body>
            HTML body example
          <body>
        </html>
      `;
        await next();
    },
);

console.log(`Listen port to 4000`)
await server.listen({ port: 4000 });