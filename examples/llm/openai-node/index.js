import OpenAI from 'npm:openai';

const openai = new OpenAI({
  apiKey: Luwak().env.get("OPENAI_API_KEY")
});

async function main() {
  const chatCompletion = await openai.chat.completions.create({
    messages: [{ role: 'user', content: 'Say this is a test' }],
    model: 'gpt-3.5-turbo',
  });

  console.log(chatCompletion.choices);
}

main();