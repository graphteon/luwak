import { OpenAI } from "https://github.com/load1n9/openai/blob/main/mod.ts";
const KEY = Luwak().env.get("OPENAI_API_KEY");

const openAI = new OpenAI(KEY);

const image = await openAI.createImage({
  prompt: "A unicorn in space",
});

console.log(JSON.stringify(image));