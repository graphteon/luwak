import { OpenAI } from "https://github.com/load1n9/openai/blob/main/mod.ts";
const KEY = Luwak().env.get("OPENAI_API_KEY");

const openAI = new OpenAI(KEY);

const transcription = await openAI.createTranscription({
  model: "whisper-1",
  file: '/path/to/your/audio/file.mp3',
});

console.log(transcription);