// $ ANTHROPIC_API_KEY=xxxxxxx luwak index.js

import Anthropic from "npm:@anthropic-ai/sdk@0.8.1";
const anthropic = new Anthropic({
    apiKey: Luwak().env.get("ANTHROPIC_API_KEY"),
});

async function main() {
    const completion = await anthropic.completions.create({
        model: 'claude-2',
        max_tokens_to_sample: 300,
        prompt: `${Anthropic.HUMAN_PROMPT} How many toes do dogs have?${Anthropic.AI_PROMPT}`,
    });
    console.log(completion.completion);
}
main().catch(console.error);