/**
 * ARW Content Script
 * Runs on every page load to inspect for Agent-Ready Web features
 * Collects page size, machine view content, and protocol information
 */

(function() {
  'use strict';

  // Prevent multiple executions
  if (window.__arwInspectorLoaded) return;
  window.__arwInspectorLoaded = true;

  const currentOrigin = window.location.origin;
  const currentUrl = window.location.href;

  /**
   * Main inspection function
   */
  async function inspectPage() {
    const results = {
      url: currentUrl,
      origin: currentOrigin,
      timestamp: new Date().toISOString(),
      arwCompliant: false,
      pageSize: document.documentElement.outerHTML.length,
      machineViewSize: 0,
      machineViewContent: null,
      discoveries: {
        llmsTxt: null,
        machineViews: [],
        wellKnown: {},
        robotsTxt: null,
        sitemapXml: null,
        metaTags: []
      },
      aiHeaders: [],
      permissions: {
        training: null,
        inference: null,
        attribution: null,
        rateLimit: null
      },
      actions: [],
      auth: null,
      errors: []
    };

    try {
      // Run all checks in parallel for speed
      const [
        llmsTxt,
        wellKnown,
        robotsTxt,
        machineViewResult
      ] = await Promise.all([
        checkLlmsTxt(),
        checkWellKnownFiles(),
        checkRobotsTxt(),
        checkForMachineView()
      ]);

      results.discoveries.llmsTxt = llmsTxt;
      results.discoveries.wellKnown = wellKnown;
      results.discoveries.robotsTxt = robotsTxt;

      // Handle machine view - now includes content
      if (machineViewResult && machineViewResult.exists) {
        results.discoveries.machineViews.push({
          exists: true,
          url: machineViewResult.url,
          currentPage: currentUrl,
          source: machineViewResult.source
        });
        results.machineViewContent = machineViewResult.content;
        results.machineViewSize = machineViewResult.content?.length || 0;
      }

      // Scan for meta tags
      results.discoveries.metaTags = scanMetaTags();

      // Extract actions from manifest
      if (wellKnown.manifest?.data?.actions) {
        results.actions = wellKnown.manifest.data.actions;
      }

      // Extract auth info from manifest
      if (wellKnown.manifest?.data?.auth) {
        results.auth = wellKnown.manifest.data.auth;
      }

      // Check for AI headers in machine view
      if (machineViewResult?.url) {
        const headers = await checkAIHeaders(machineViewResult.url);
        results.aiHeaders = headers.headers;
        results.permissions = headers.permissions;
      }

      // Determine ARW compliance
      results.arwCompliant = !!(
        results.discoveries.llmsTxt?.exists ||
        results.discoveries.wellKnown.manifest?.exists ||
        results.discoveries.machineViews.length > 0
      );

    } catch (error) {
      results.errors.push({
        message: error.message,
        stack: error.stack
      });
    }

    return results;
  }

  /**
   * Check for llms.txt at site root
   */
  async function checkLlmsTxt() {
    try {
      const llmsTxtUrl = `${currentOrigin}/llms.txt`;
      const response = await fetch(llmsTxtUrl, {
        method: 'GET',
        mode: 'cors',
        cache: 'no-cache'
      });

      if (response.ok) {
        const content = await response.text();

        // Validate it's not HTML
        if (isHTML(content)) {
          return { exists: false, error: 'Response was HTML, not llms.txt' };
        }

        return {
          exists: true,
          url: llmsTxtUrl,
          size: content.length,
          preview: content.substring(0, 500),
          hasContent: content.includes('content:') || content.includes('site:')
        };
      }
    } catch (error) {
      return {
        exists: false,
        error: error.message
      };
    }
    return { exists: false };
  }

  /**
   * Check for .well-known ARW files
   */
  async function checkWellKnownFiles() {
    const wellKnownFiles = {
      manifest: '/.well-known/arw-manifest.json',
      contentIndex: '/.well-known/arw-content-index.json',
      policies: '/.well-known/arw-policies.json'
    };

    const results = {};

    const checks = Object.entries(wellKnownFiles).map(async ([key, path]) => {
      try {
        const url = `${currentOrigin}${path}`;
        const response = await fetch(url, {
          method: 'GET',
          mode: 'cors',
          cache: 'no-cache'
        });

        if (response.ok) {
          const text = await response.text();

          // Try to parse as JSON
          try {
            const data = JSON.parse(text);
            results[key] = {
              exists: true,
              url: url,
              data: data
            };
          } catch (e) {
            results[key] = { exists: false, error: 'Invalid JSON' };
          }
        } else {
          results[key] = { exists: false };
        }
      } catch (error) {
        results[key] = { exists: false, error: error.message };
      }
    });

    await Promise.all(checks);
    return results;
  }

  /**
   * Check robots.txt for ARW hints
   */
  async function checkRobotsTxt() {
    try {
      const robotsUrl = `${currentOrigin}/robots.txt`;
      const response = await fetch(robotsUrl, {
        method: 'GET',
        mode: 'cors',
        cache: 'no-cache'
      });

      if (response.ok) {
        const content = await response.text();

        // Validate it's not HTML
        if (isHTML(content)) {
          return { exists: false, error: 'Response was HTML' };
        }

        const hasArwHints = /llms\.txt|\.well-known\/arw|agent-ready/i.test(content);

        return {
          exists: true,
          url: robotsUrl,
          hasArwHints: hasArwHints,
          preview: content.substring(0, 300)
        };
      }
    } catch (error) {
      return {
        exists: false,
        error: error.message
      };
    }
    return { exists: false };
  }

  /**
   * Scan page for ARW-related meta tags
   */
  function scanMetaTags() {
    const metaTags = [];

    const arwMetaSelectors = [
      'meta[name*="arw"]',
      'meta[name*="llm"]',
      'meta[name*="agent"]',
      'meta[name*="ai-"]',
      'meta[property*="arw"]',
      'link[rel="llms"]',
      'link[rel="machine-view"]',
      'link[rel="alternate"][type*="llm"]'
    ];

    arwMetaSelectors.forEach(selector => {
      const elements = document.querySelectorAll(selector);
      elements.forEach(el => {
        metaTags.push({
          tag: el.tagName.toLowerCase(),
          name: el.getAttribute('name') || el.getAttribute('property') || el.getAttribute('rel'),
          content: el.getAttribute('content') || el.getAttribute('href')
        });
      });
    });

    return metaTags;
  }

  /**
   * Check if current page has a corresponding machine view
   * Returns the content directly if found
   */
  async function checkForMachineView() {
    const pathname = window.location.pathname;

    // First check link tags for machine view
    const machineViewLink = document.querySelector(
      'link[rel="machine-view"], link[rel="alternate"][type*="llm"], link[rel="alternate"][type="text/markdown"]'
    );

    if (machineViewLink) {
      const href = machineViewLink.getAttribute('href');
      if (href) {
        const url = new URL(href, currentOrigin).href;
        const result = await fetchAndValidateMachineView(url);
        if (result) {
          return { ...result, source: 'link-tag' };
        }
      }
    }

    // Try common patterns for machine views
    const potentialPaths = [
      pathname.replace(/\.html?$/, '.llm.md'),
      pathname.replace(/\/$/, '') + '.llm.md',
      pathname + '.llm.md',
      pathname.replace(/\/$/, '/index.llm.md'),
      // Also try without .llm extension
      pathname.replace(/\.html?$/, '.md'),
      pathname.replace(/\/$/, '') + '.md'
    ];

    // Remove duplicates and invalid paths
    const uniquePaths = [...new Set(potentialPaths)].filter(p =>
      p && p !== '.llm.md' && p !== '.md' && p.length > 1
    );

    for (const path of uniquePaths) {
      const machineViewUrl = `${currentOrigin}${path}`;
      const result = await fetchAndValidateMachineView(machineViewUrl);
      if (result) {
        return { ...result, source: 'path-convention' };
      }
    }

    return null;
  }

  /**
   * Fetch and validate that content is actually a machine view (markdown)
   */
  async function fetchAndValidateMachineView(url) {
    try {
      const response = await fetch(url, {
        method: 'GET',
        mode: 'cors',
        cache: 'no-cache'
      });

      if (!response.ok) {
        return null;
      }

      // Check content-type
      const contentType = response.headers.get('content-type') || '';
      const isMarkdownType = contentType.includes('markdown') ||
                             contentType.includes('text/plain') ||
                             contentType.includes('text/x-markdown');

      const content = await response.text();

      // Validate content is markdown, not HTML
      if (isHTML(content)) {
        return null;
      }

      // Check if it looks like markdown
      if (!isMarkdown(content)) {
        return null;
      }

      return {
        exists: true,
        url: url,
        content: content,
        contentType: contentType
      };
    } catch (error) {
      return null;
    }
  }

  /**
   * Check if content is HTML
   */
  function isHTML(content) {
    if (!content) return false;
    const trimmed = content.trim().toLowerCase();
    return trimmed.startsWith('<!doctype') ||
           trimmed.startsWith('<html') ||
           trimmed.startsWith('<head') ||
           trimmed.startsWith('<body') ||
           (trimmed.includes('<html') && trimmed.includes('</html>'));
  }

  /**
   * Check if content looks like markdown
   */
  function isMarkdown(content) {
    if (!content || content.length < 10) return false;

    // Check for common markdown patterns
    const markdownPatterns = [
      /^#+ /m,           // Headers
      /^\* /m,           // Unordered lists
      /^- /m,            // Unordered lists
      /^\d+\. /m,        // Ordered lists
      /\[.+\]\(.+\)/,    // Links
      /```/,             // Code blocks
      /^\>/m,            // Blockquotes
      /\*\*.+\*\*/,      // Bold
      /__.+__/,          // Bold
      /\*.+\*/,          // Italic
      /_.+_/,            // Italic
    ];

    // If it has any markdown patterns, consider it markdown
    const hasMarkdown = markdownPatterns.some(pattern => pattern.test(content));

    // Also accept plain text that doesn't look like HTML
    const isPlainText = !content.includes('<div') &&
                        !content.includes('<span') &&
                        !content.includes('<script');

    return hasMarkdown || isPlainText;
  }

  /**
   * Check for AI-specific HTTP headers
   */
  async function checkAIHeaders(url) {
    const headers = [];
    const permissions = {
      training: null,
      inference: null,
      attribution: null,
      rateLimit: null
    };

    try {
      const response = await fetch(url, {
        method: 'HEAD',
        mode: 'cors',
        cache: 'no-cache'
      });

      // Check for AI headers
      const aiHeaderNames = [
        'AI-Attribution',
        'AI-Training',
        'AI-Inference',
        'AI-Rate-Limit',
        'X-AI-Attribution',
        'X-AI-Training',
        'X-Robots-Tag'
      ];

      aiHeaderNames.forEach(headerName => {
        const value = response.headers.get(headerName);
        if (value) {
          headers.push({ name: headerName, value: value });

          // Parse permissions
          const lowerName = headerName.toLowerCase();
          if (lowerName.includes('training')) {
            permissions.training = value.includes('disallow') ? 'disallowed' : 'allowed';
          } else if (lowerName.includes('inference')) {
            permissions.inference = value.includes('disallow') ? 'disallowed' : 'allowed';
          } else if (lowerName.includes('attribution')) {
            permissions.attribution = value.includes('required') ? 'required' : value;
          } else if (lowerName.includes('rate-limit')) {
            permissions.rateLimit = value;
          }
        }
      });
    } catch (error) {
      console.log('Failed to check AI headers:', error.message);
    }

    return { headers, permissions };
  }

  /**
   * Send inspection results to background service worker
   */
  async function reportResults(results) {
    try {
      await chrome.runtime.sendMessage({
        type: 'ARW_INSPECTION_COMPLETE',
        data: results
      });
    } catch (error) {
      console.error('ARW Inspector: Failed to send results', error);
    }
  }

  /**
   * Update extension badge based on ARW compliance
   */
  function updateBadge(isCompliant) {
    chrome.runtime.sendMessage({
      type: 'UPDATE_BADGE',
      data: {
        text: isCompliant ? '✓' : '',
        color: isCompliant ? '#22c55e' : '#94a3b8'
      }
    });
  }

  // ============================================================================
  // GEO ANALYSIS FUNCTIONS (from geo-geo-bundle)
  // ============================================================================

  function getPageText() {
    const main = document.querySelector('main') || document.body;
    return main.innerText || '';
  }

  function getLinks() {
    return Array.from(document.querySelectorAll('a[href]'));
  }

  function analyzeCitations() {
    const links = getLinks();
    const externalLinks = links.filter((link) => {
      const href = link.getAttribute('href') || '';
      if (!href || href.startsWith('#') || href.startsWith('javascript:')) return false;
      try {
        const url = new URL(href, window.location.href);
        return url.hostname !== window.location.hostname;
      } catch {
        return false;
      }
    });

    const externalDomains = Array.from(
      new Set(
        externalLinks
          .map((l) => {
            try {
              return new URL(l.href).hostname;
            } catch {
              return null;
            }
          })
          .filter(Boolean)
      )
    );

    return {
      totalLinks: links.length,
      externalLinks: externalLinks.length,
      externalDomains
    };
  }

  function analyzeStatistics(text) {
    const percentRegex = /\b\d+(\.\d+)?\s*%/g;
    const currencyRegex = /(\$|€|£)\s?\d[\d,]*(\.\d+)?/g;
    const bigNumberRegex = /\b\d{4,}\b/g;

    const percentages = text.match(percentRegex) || [];
    const currencies = text.match(currencyRegex) || [];
    const bigNumbers = text.match(bigNumberRegex) || [];

    const total = percentages.length + currencies.length + bigNumbers.length;

    return {
      total,
      percentages,
      currencies,
      bigNumbers
    };
  }

  function analyzeQuotations() {
    const blockquotes = Array.from(document.querySelectorAll('blockquote'));
    const text = getPageText();
    const quoteRegex = /"([^"]+)"|"([^"]+)"/g;
    const inlineQuotes = [];
    let match;
    while ((match = quoteRegex.exec(text)) !== null) {
      inlineQuotes.push(match[1] || match[2]);
    }

    return {
      blockquoteCount: blockquotes.length,
      inlineQuoteCount: inlineQuotes.length,
      sampleQuotes: inlineQuotes.slice(0, 5)
    };
  }

  function analyzeEntities(text) {
    const words = text.split(/\s+/);
    const candidates = words.filter((w) => /^[A-Z][a-zA-Z0-9\-]+$/.test(w));
    const freq = {};
    for (const c of candidates) {
      freq[c] = (freq[c] || 0) + 1;
    }
    const entries = Object.entries(freq)
      .sort((a, b) => b[1] - a[1])
      .map(([name, count]) => ({ name, count }));
    return {
      totalEntities: entries.length,
      entities: entries.slice(0, 25)
    };
  }

  function analyzeMachineViews() {
    const links = getLinks();
    const llmMdLinks = links.filter((link) => link.href.endsWith('.llm.md'));
    const plannedChecks = ['/.well-known/llms.txt', '/sitemap-llm.xml'];
    return {
      llmMdCount: llmMdLinks.length,
      llmMdUrls: llmMdLinks.map((l) => l.href),
      plannedChecks
    };
  }

  function analyzeStructure(text) {
    const headings = document.querySelectorAll('h1, h2, h3, h4, h5, h6');
    const hasMainTag = !!document.querySelector('main');
    const wordCount = text.split(/\s+/).filter(Boolean).length;
    return {
      headingCount: headings.length,
      hasMainTag,
      wordCount
    };
  }

  // ============================================================================
  // GEO SCORING (from geoScoring.ts)
  // ============================================================================

  function computeAuthorityScore(m) {
    const extLinks = m.citations?.externalLinks ?? 0;
    const domainCount = m.citations?.externalDomains?.length ?? 0;
    const llmMdCount = m.machineViews?.llmMdCount ?? 0;

    const extLinksNorm = Math.min(extLinks, 10) / 10;
    const domainNorm = Math.min(domainCount, 5) / 5;
    const llmMdNorm = llmMdCount > 0 ? 1 : 0;

    const score = extLinksNorm * 0.4 + domainNorm * 0.3 + llmMdNorm * 0.3;
    return Math.round(score * 100);
  }

  function computeEvidenceScore(m) {
    const statsTotal = m.statistics?.total ?? 0;
    const quotesTotal =
      (m.quotations?.blockquoteCount ?? 0) +
      (m.quotations?.inlineQuoteCount ?? 0);

    const statsNorm = Math.min(statsTotal, 10) / 10;
    const quotesNorm = Math.min(quotesTotal, 8) / 8;

    const score = statsNorm * 0.6 + quotesNorm * 0.4;
    return Math.round(score * 100);
  }

  function computeSemanticClarityScore(m) {
    const totalEntities = m.entities?.totalEntities ?? 0;
    const wordCount = m.structure?.wordCount ?? 0;

    const wordNorm = Math.min(wordCount, 1500) / 1500;

    const idealMin = 8;
    const idealMax = 60;
    let entityNorm = 0;
    if (totalEntities <= 0) entityNorm = 0;
    else if (totalEntities < idealMin) entityNorm = totalEntities / idealMin;
    else if (totalEntities <= idealMax) entityNorm = 1;
    else entityNorm = Math.max(0.5, 1 - (totalEntities - idealMax) / 100);

    const score = entityNorm * 0.7 + wordNorm * 0.3;
    return Math.round(score * 100);
  }

  function computeArwReadinessScore(m) {
    const llmMdCount = m.machineViews?.llmMdCount ?? 0;
    const plannedChecks = m.machineViews?.plannedChecks?.length ?? 0;
    const hasMain = m.structure?.hasMainTag ?? false;
    const headingCount = m.structure?.headingCount ?? 0;

    const llmMdNorm = llmMdCount > 0 ? 1 : 0;
    const checksNorm = Math.min(plannedChecks, 3) / 3;
    const headingsNorm = Math.min(headingCount, 12) / 12;
    const mainNorm = hasMain ? 1 : 0;

    const score =
      llmMdNorm * 0.4 +
      checksNorm * 0.2 +
      headingsNorm * 0.25 +
      mainNorm * 0.15;

    return Math.round(score * 100);
  }

  function computeGeoScore(metrics) {
    const authority = computeAuthorityScore(metrics);
    const evidence = computeEvidenceScore(metrics);
    const semantic = computeSemanticClarityScore(metrics);
    const arw = computeArwReadinessScore(metrics);

    const finalScore =
      authority * 0.3 +
      evidence * 0.25 +
      semantic * 0.25 +
      arw * 0.2;

    return {
      geoScore: Math.round(finalScore),
      subscores: {
        authority,
        evidence,
        semanticClarity: semantic,
        arwReadiness: arw
      }
    };
  }

  // ============================================================================
  // INTEGRATED GEO ANALYSIS
  // ============================================================================

  function performGeoAnalysis() {
    const text = getPageText();
    const citations = analyzeCitations();
    const stats = analyzeStatistics(text);
    const quotes = analyzeQuotations();
    const entities = analyzeEntities(text);
    const machineViews = analyzeMachineViews();
    const structure = analyzeStructure(text);

    const metrics = {
      citations,
      statistics: stats,
      quotations: quotes,
      entities,
      machineViews,
      structure
    };

    const { geoScore, subscores } = computeGeoScore(metrics);

    return {
      geoScore,
      subscores,
      metrics,
      analyzedAt: new Date().toISOString()
    };
  }

  // Run inspection when page loads
  async function runInspection() {
    const results = await inspectPage();

    // Add automatic GEO analysis
    try {
      const geoAnalysis = performGeoAnalysis();
      results.geo = geoAnalysis;
    } catch (error) {
      console.error('GEO analysis failed:', error);
      results.geo = null;
    }

    updateBadge(results.arwCompliant);
    await reportResults(results);
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', runInspection);
  } else {
    runInspection();
  }

})();
