const { req, res, Server } = Luwak.http;
const server = new Server();
const msg = new TextEncoder().encode("data: hello\r\n\r\n");
server.post(
    "/",
    async (ctx: any, next: any) => {
        let timerId: number | undefined;
        ctx.res.headers.set(
            "Content-Type", "text/event-stream",
        );
        ctx.res.body = new ReadableStream({
            start(controller) {
                timerId = setInterval(() => {
                    controller.enqueue(msg);
                }, 1000);
            },
            cancel() {
                if (typeof timerId === "number") {
                    clearInterval(timerId);
                }
            },
        });
        await next();
    },
);
console.log(`Listen port to 4000`)
await server.listen({ port: 4000 });