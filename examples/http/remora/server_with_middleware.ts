import { Remora } from 'https://github.com/graphteon/remora/blob/main/mod.ts'

async function middleware(ctx: any, next: any) {
    console.log("midleware print method :", ctx.req.method);
    console.log("midleware print headers :", ctx.req.headers);
    await next();
}

const server = new Remora("./lambdas");
await server.use(middleware);
await server.listen();