import { getContentEmbedding } from '../src/lib/vector-db';
import dotenv from 'dotenv';

dotenv.config();

async function testAgent() {
    console.log('🧪 Testing AI Agent (Embedding Generation)...');

    const text = "This is a test movie description about a space adventure.";
    const start = Date.now();

    try {
        const embedding = await getContentEmbedding(text);
        const duration = Date.now() - start;

        if (embedding && embedding.length > 0) {
            console.log(`✅ Success! Generated embedding in ${duration}ms`);
            console.log(`📊 Vector dimensions: ${embedding.length}`);
            console.log(`🔑 API Keys present:`);
            console.log(`   - OpenAI: ${process.env.OPENAI_API_KEY ? 'Yes' : 'No'}`);
            console.log(`   - Gemini: ${process.env.GOOGLE_GENERATIVE_AI_API_KEY ? 'Yes' : 'No'}`);
        } else {
            console.error('❌ Failed to generate embedding (returned null or empty)');
        }
    } catch (error) {
        console.error('❌ Error during testing:', error);
    }
}

testAgent();
