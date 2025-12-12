
import { localDB } from '../src/lib/local-db';

async function testSorting() {
    console.log('Testing recommendation sorting...');

    const recommendations = localDB.recommend({
        type: 'movie',
        limit: 10
    });

    console.log(`Got ${recommendations.length} recommendations.`);

    let isSorted = true;
    for (let i = 0; i < recommendations.length - 1; i++) {
        const current = recommendations[i];
        const next = recommendations[i + 1];

        console.log(`#${i + 1}: ${current.title} - Vote Count: ${current.voteCount}`);

        if ((current.voteCount || 0) < (next.voteCount || 0)) {
            isSorted = false;
            console.error(`❌ Sorting violation at index ${i}: ${current.voteCount} < ${next.voteCount}`);
        }
    }
    console.log(`#${recommendations.length}: ${recommendations[recommendations.length - 1].title} - Vote Count: ${recommendations[recommendations.length - 1].voteCount}`);

    if (isSorted) {
        console.log('✅ Recommendations are correctly sorted by voteCount (descending).');
    } else {
        console.error('❌ Recommendations are NOT sorted correctly.');
        process.exit(1);
    }
}

testSorting();
