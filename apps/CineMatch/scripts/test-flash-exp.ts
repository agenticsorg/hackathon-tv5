import { google } from '@ai-sdk/google';
import { generateText } from 'ai';
import dotenv from 'dotenv';

dotenv.config();

async function testChat() {
    console.log('🧪 Testing AI Chat (Gemini Flash Exp)...');

    try {
        const model = google('gemini-2.0-flash-exp');

        const start = Date.now();
        const { text } = await generateText({
            model,
            prompt: 'Hello, are you working?',
        });
        const duration = Date.now() - start;

        console.log(`✅ Success! Response in ${duration}ms`);
        console.log(`🤖 Response: ${text}`);
    } catch (error) {
        console.error('❌ Chat generation failed:', error);
    }
}

testChat();
