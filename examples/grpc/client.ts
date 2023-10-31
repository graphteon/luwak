import { getClient } from "https://github.com/prohazko2/deno-grpc/blob/main/client.ts";
import { Greeter } from "./greeter.d.ts";

const protoPath = new URL("./greeter.proto", import.meta.url);
const protoFile = await Luwak().readTextFile(protoPath);

const client = getClient<Greeter>({
  port: 50051,
  root: protoFile,
  serviceName: "Greeter",
});

/* unary calls */
console.log(await client.SayHello({ name: "unary #1" }));
console.log(await client.SayHello({ name: "unary #2" }));

/* server stream */
for await (const reply of client.ShoutHello({ name: "streamed" })) {
  console.log(reply);
}

client.close();