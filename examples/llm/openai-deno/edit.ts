import { OpenAI } from "https://github.com/load1n9/openai/blob/main/mod.ts";
const KEY = Luwak().env.get("OPENAI_API_KEY");

const openAI = new OpenAI(KEY);

const edit = await openAI.createEdit({
  model: "text-davinci-edit-001",
  input: "What day of the wek is it?",
  instruction: "Fix the spelling mistakes",
});

console.log(edit);