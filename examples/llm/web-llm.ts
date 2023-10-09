//import * as webllm from "esm://npm@mlc-ai/web-llm@0.2.8";
import * as webllm from "https://esm.sh/@mlc-ai/web-llm@0.2.8";
// We use label to intentionally keep it simple
function setLabel(id: string, text: string) {
  const label = document.getElementById(id);
  if (label == null) {
    throw Error("Cannot find label " + id);
  }
  label.innerText = text;
}

// create a ChatModule,
const chat = new webllm.ChatModule();
// This callback allows us to report initialization progress
chat.setInitProgressCallback((report: webllm.InitProgressReport) => {
  setLabel("init-label", report.text);
});
// You can also try out "RedPajama-INCITE-Chat-3B-v1-q4f32_0"
await chat.reload("Llama-2-7b-chat-hf-q4f32_1");

const generateProgressCallback = (_step: number, message: string) => {
  setLabel("generate-label", message);
};

const prompt0 = "What is the capital of Canada?";
setLabel("prompt-label", prompt0);
const reply0 = await chat.generate(prompt0, generateProgressCallback);
console.log(reply0);

const prompt1 = "Can you write a poem about it?";
setLabel("prompt-label", prompt1);
const reply1 = await chat.generate(prompt1, generateProgressCallback);
console.log(reply1);

console.log(await chat.runtimeStatsText());
