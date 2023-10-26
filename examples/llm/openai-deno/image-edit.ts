import { OpenAI } from "https://github.com/load1n9/openai/blob/main/mod.ts";
const KEY = Luwak().env.get("OPENAI_API_KEY");

const openAI = new OpenAI(KEY);

const imageEdit = await openAI.createImageEdit({
  image: "@otter.png",
  mask: "@mask.png",
  prompt: "A cute baby sea otter wearing a beret",
  n: 2,
  size: "1024x1024",
});

console.log(imageEdit);