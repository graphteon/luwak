import { GrpcServer } from "https://github.com/prohazko2/deno-grpc/blob/main/server.ts";
import { Greeter } from "./greeter.d.ts";

const port = 50051;
const server = new GrpcServer();

const protoPath = new URL("./greeter.proto", import.meta.url);
const protoFile = await Luwak().readTextFile(protoPath);

server.addService<Greeter>(protoFile, {
  
  async SayHello({ name }) {
    const message = `hello ${name || "stranger"}`;
    return { message };
  },

  async *ShoutHello({ name }) {
    for (const n of [0, 1, 2]) {
      const message = `hello ${name || "stranger"} #${n}`;
      yield { message };
    }
  }
});

console.log(`gonna listen on ${port} port`);
for await (const conn of Luwak().listen({ port })) {
  server.handle(conn);
}