import { OpenAI } from "https://github.com/load1n9/openai/blob/main/mod.ts";
const KEY = Luwak().env.get("OPENAI_API_KEY");

const openAI = new OpenAI(KEY);

const completion = await openAI.createCompletion({
  model: "davinci",
  prompt: "The meaning of life is",
});

console.log(completion.choices);