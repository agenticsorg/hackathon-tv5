import { NextRequest, NextResponse } from 'next/server';
import { Pool } from 'pg';

// Force dynamic rendering - don't try to pre-render during build
export const dynamic = 'force-dynamic';

const pool = new Pool({
  connectionString: process.env.DATABASE_URL ||
    'postgresql://tvdb:tvdb_recommender_2024@localhost:5432/tvdb_recommender',
  max: 10,
});

export async function GET(
  request: NextRequest,
  { params }: { params: { id: string } }
) {
  const { id } = params;

  if (!id) {
    return NextResponse.json({ error: 'Content ID required' }, { status: 400 });
  }

  const client = await pool.connect();

  try {
    const result = await client.query(`
      SELECT
        id, title, year, content_type, genres, overview,
        rating, network_name, original_language, image_url
      FROM content
      WHERE id = $1
    `, [id]);

    if (result.rows.length === 0) {
      return NextResponse.json({ error: 'Content not found' }, { status: 404 });
    }

    return NextResponse.json({
      content: result.rows[0],
      timestamp: new Date().toISOString(),
    });
  } catch (error: any) {
    console.error('Content fetch error:', error);
    return NextResponse.json({ error: error.message }, { status: 500 });
  } finally {
    client.release();
  }
}
