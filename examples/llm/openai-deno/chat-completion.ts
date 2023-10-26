import { OpenAI } from "https://github.com/load1n9/openai/blob/main/mod.ts";
const KEY = Luwak().env.get("OPENAI_API_KEY");

const openAI = new OpenAI(KEY);

const completion = await openAI.createCompletion({
  model: "davinci",
  prompt: "The meaning of life is",
});

const chatCompletion = await openAI.createChatCompletion({
  model: "gpt-3.5-turbo",
  messages: [
    { "role": "system", "content": "You are a helpful assistant." },
    { "role": "user", "content": "Who won the world series in 2020?" },
    {
      "role": "assistant",
      "content": "The Los Angeles Dodgers won the World Series in 2020.",
    },
    { "role": "user", "content": "Where was it played?" },
  ],
});

console.log(chatCompletion);