/**
 * HTML test fixtures for crawler testing
 * Provides sample HTML content for various test scenarios
 */

export const BASIC_HTML = `
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>Test Page</title>
</head>
<body>
  <h1>Welcome to Test Page</h1>
  <p>This is a simple test page for crawler testing.</p>
  <a href="/page1">Link 1</a>
  <a href="/page2">Link 2</a>
</body>
</html>
`;

export const ARW_COMPLIANT_HTML = `
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>ARW Compliant Page</title>
  <link rel="alternate" type="text/x-llm+markdown" href="/docs/page.llm.md" />
  <link rel="alternate" type="application/json" href="/.well-known/arw-manifest.json" />
</head>
<body>
  <section data-chunk-id="overview">
    <h1>Product Overview</h1>
    <p>This is an ARW-compliant page with machine-readable metadata.</p>
  </section>
  <section data-chunk-id="features">
    <h2>Features</h2>
    <ul>
      <li>Feature 1</li>
      <li>Feature 2</li>
    </ul>
  </section>
</body>
</html>
`;

export const COMPLEX_HTML = `
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>Complex Page</title>
</head>
<body>
  <nav>
    <a href="/">Home</a>
    <a href="/about">About</a>
  </nav>
  <main>
    <article>
      <h1>Article Title</h1>
      <p>Article content with <strong>bold</strong> and <em>italic</em> text.</p>
      <pre><code>const example = "code block";</code></pre>
    </article>
    <aside>
      <h3>Related</h3>
      <ul>
        <li><a href="/related1">Related 1</a></li>
        <li><a href="/related2">Related 2</a></li>
      </ul>
    </aside>
  </main>
  <footer>
    <p>&copy; 2024 Test Site</p>
  </footer>
</body>
</html>
`;

export const MALFORMED_HTML = `
<!DOCTYPE html>
<html>
<head>
  <title>Malformed Page
</head>
<body>
  <h1>Missing closing tags
  <p>Unclosed paragraph
  <div>
    <span>Nested unclosed
  </div>
</body>
`;

export const EMPTY_HTML = `
<!DOCTYPE html>
<html>
<head><title>Empty</title></head>
<body></body>
</html>
`;

export const HTML_WITH_SCRIPTS = `
<!DOCTYPE html>
<html>
<head>
  <title>Page with Scripts</title>
  <script>
    var sensitive = "should be removed";
    console.log("test");
  </script>
</head>
<body>
  <h1>Content</h1>
  <script src="/app.js"></script>
  <p>Text content</p>
</body>
</html>
`;

export const HTML_WITH_METADATA = `
<!DOCTYPE html>
<html>
<head>
  <meta charset="UTF-8">
  <meta name="description" content="Test description">
  <meta name="keywords" content="test, crawler, arw">
  <meta property="og:title" content="OG Title">
  <meta property="og:description" content="OG Description">
  <meta name="robots" content="index, follow">
  <title>Metadata Test</title>
</head>
<body>
  <h1>Page with Rich Metadata</h1>
</body>
</html>
`;

export const LARGE_HTML = `
<!DOCTYPE html>
<html>
<head><title>Large Page</title></head>
<body>
  ${Array(1000).fill('<p>Repeated content paragraph.</p>').join('\n')}
  ${Array(100).fill('<a href="/link">Link</a>').join(' ')}
</body>
</html>
`;
