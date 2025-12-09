#!/usr/bin/env tsx

/**
 * Seed Database Script
 *
 * Loads synthetic entertainment metadata into Firestore for testing and development.
 * Uses batch operations for efficient writes and includes progress logging.
 */

import * as admin from 'firebase-admin';
import * as fs from 'fs';
import * as path from 'path';

// Initialize Firebase Admin SDK
const initializeFirebase = (): admin.firestore.Firestore => {
  const serviceAccountPath = process.env.GOOGLE_APPLICATION_CREDENTIALS ||
    path.join(__dirname, '../../service-account-key.json');

  if (!admin.apps.length) {
    if (fs.existsSync(serviceAccountPath)) {
      const serviceAccount = JSON.parse(fs.readFileSync(serviceAccountPath, 'utf8'));
      admin.initializeApp({
        credential: admin.credential.cert(serviceAccount)
      });
    } else {
      console.warn('⚠️  Service account key not found. Using default credentials.');
      admin.initializeApp();
    }
  }

  return admin.firestore();
};

// Load JSON data from file
const loadJsonData = <T>(filename: string): T[] => {
  const dataPath = path.join(__dirname, '../data', filename);
  const rawData = fs.readFileSync(dataPath, 'utf8');
  return JSON.parse(rawData) as T[];
};

// Batch write data to Firestore
const batchWriteToCollection = async (
  db: admin.firestore.Firestore,
  collectionName: string,
  data: any[],
  idField: string = 'id'
): Promise<void> => {
  const batchSize = 500; // Firestore batch limit
  const batches: admin.firestore.WriteBatch[] = [];
  let currentBatch = db.batch();
  let operationCount = 0;
  let batchCount = 0;

  console.log(`\n📝 Writing ${data.length} documents to '${collectionName}' collection...`);

  for (const item of data) {
    const docId = item[idField] || db.collection(collectionName).doc().id;
    const docRef = db.collection(collectionName).doc(docId);

    currentBatch.set(docRef, {
      ...item,
      createdAt: admin.firestore.FieldValue.serverTimestamp(),
      updatedAt: admin.firestore.FieldValue.serverTimestamp()
    });

    operationCount++;

    // Create new batch if we hit the limit
    if (operationCount === batchSize) {
      batches.push(currentBatch);
      currentBatch = db.batch();
      operationCount = 0;
      batchCount++;
    }
  }

  // Add remaining operations
  if (operationCount > 0) {
    batches.push(currentBatch);
    batchCount++;
  }

  // Commit all batches
  console.log(`   Committing ${batchCount} batch(es)...`);
  let completedBatches = 0;

  for (const batch of batches) {
    await batch.commit();
    completedBatches++;
    process.stdout.write(`\r   Progress: ${completedBatches}/${batchCount} batches completed`);
  }

  console.log(`\n✅ Successfully wrote ${data.length} documents to '${collectionName}'`);
};

// Main seeding function
const seedDatabase = async (): Promise<void> => {
  console.log('🌱 Starting database seeding process...\n');
  console.log('═'.repeat(60));

  try {
    const db = initializeFirebase();

    // Load synthetic data
    console.log('\n📂 Loading synthetic data files...');
    const movies = loadJsonData('synthetic-movies.json');
    const series = loadJsonData('synthetic-series.json');
    const users = loadJsonData('synthetic-users.json');

    console.log(`   ✓ Loaded ${movies.length} movies`);
    console.log(`   ✓ Loaded ${series.length} TV series`);
    console.log(`   ✓ Loaded ${users.length} user profiles`);

    // Seed collections
    await batchWriteToCollection(db, 'movies', movies, 'id');
    await batchWriteToCollection(db, 'series', series, 'id');
    await batchWriteToCollection(db, 'users', users, 'userId');

    // Create indexes (informational - actual indexes must be created in Firestore console)
    console.log('\n📊 Recommended Firestore indexes:');
    console.log('   • movies: genres (array), rating (desc), releaseYear (desc)');
    console.log('   • series: genres (array), rating (desc), releaseYear (desc)');
    console.log('   • users: preferredGenres (array)');
    console.log('\n   Note: Create these composite indexes in the Firebase Console');

    console.log('\n═'.repeat(60));
    console.log('✨ Database seeding completed successfully!');
    console.log('\n📈 Summary:');
    console.log(`   • Movies seeded: ${movies.length}`);
    console.log(`   • Series seeded: ${series.length}`);
    console.log(`   • Users seeded: ${users.length}`);
    console.log(`   • Total documents: ${movies.length + series.length + users.length}`);
    console.log('═'.repeat(60));

  } catch (error) {
    console.error('\n❌ Error seeding database:', error);
    if (error instanceof Error) {
      console.error('   Error message:', error.message);
      console.error('   Stack trace:', error.stack);
    }
    process.exit(1);
  }
};

// Run seeding if this file is executed directly
if (require.main === module) {
  seedDatabase()
    .then(() => {
      console.log('\n👋 Exiting...\n');
      process.exit(0);
    })
    .catch((error) => {
      console.error('Fatal error:', error);
      process.exit(1);
    });
}

export { seedDatabase, batchWriteToCollection, loadJsonData };
