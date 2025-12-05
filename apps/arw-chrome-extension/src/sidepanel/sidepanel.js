/**
 * ARW Inspector Side Panel
 * Displays machine view, token costs, and protocols
 */

(function() {
  'use strict';

  // Token cost constants (GPT-4 pricing as reference)
  const TOKEN_COST_PER_1K = 0.03; // $0.03 per 1K input tokens
  const CHARS_PER_TOKEN = 4; // Approximate characters per token

  // State
  let currentData = null;
  let activeTabId = 'machine-view'; // Track which tab is active
  let retryCount = 0;
  const MAX_RETRIES = 10; // Stop retrying after 10 seconds
  let lastDataReceivedTime = 0; // Track when we last received data
  let currentTabId = null; // Track current tab ID

  // Initialize when DOM loads
  document.addEventListener('DOMContentLoaded', async () => {
    setupTabs();
    setupEventListeners();
    await loadInspectionData();
  });

  /**
   * Setup tab navigation
   */
  function setupTabs() {
    const tabBtns = document.querySelectorAll('.tab-btn');
    const tabContents = document.querySelectorAll('.tab-content');

    tabBtns.forEach(btn => {
      btn.addEventListener('click', () => {
        const tabId = btn.dataset.tab;

        // Track active tab
        activeTabId = tabId;

        // Update buttons
        tabBtns.forEach(b => b.classList.remove('active'));
        btn.classList.add('active');

        // Update content
        tabContents.forEach(content => {
          content.classList.remove('active');
          content.style.display = 'none';
        });

        const activeContent = document.getElementById(`${tabId}-tab`);
        if (activeContent) {
          activeContent.classList.add('active');
          activeContent.style.display = 'block';
        }
      });
    });
  }

  /**
   * Switch to a specific tab by ID
   */
  function switchToTab(tabId) {
    const btn = document.querySelector(`.tab-btn[data-tab="${tabId}"]`);
    if (btn) {
      btn.click();
    }
  }

  /**
   * Setup event listeners
   */
  function setupEventListeners() {
    // Retry button
    document.getElementById('retry-btn')?.addEventListener('click', loadInspectionData);

    // Copy machine view
    document.getElementById('copy-machine-view')?.addEventListener('click', copyMachineView);

    // Open machine view
    document.getElementById('open-machine-view')?.addEventListener('click', openMachineView);

    // Monthly requests input
    document.getElementById('monthly-requests')?.addEventListener('input', updateProjections);

    // GEO Analysis button
    document.getElementById('open-full-analysis')?.addEventListener('click', openFullAnalysis);

    // Listen for messages from background script (inspection updates)
    chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
      if (message.type === 'INSPECTION_UPDATE' && message.data) {
        console.log('[ARW Sidepanel] Received INSPECTION_UPDATE:', {
          tabId: message.data.tabId,
          url: message.data.url,
          arwCompliant: message.data.arwCompliant
        });
        currentData = message.data;
        currentTabId = message.data.tabId;
        retryCount = 0; // Reset retry counter since we got data
        lastDataReceivedTime = Date.now(); // Track when we received data
        displayResults(currentData);
      }
      sendResponse({ received: true });
    });

    // Listen for tab updates - reload when active tab finishes loading
    chrome.tabs.onUpdated.addListener(async (tabId, changeInfo, tab) => {
      if (changeInfo.status === 'complete' && tab.active) {
        // Small delay to let content script finish
        setTimeout(async () => {
          const [activeTab] = await chrome.tabs.query({ active: true, currentWindow: true });

          // Only reload if:
          // 1. We don't have data for this tab, OR
          // 2. The last data received was more than 2 seconds ago (indicates a real navigation)
          const timeSinceLastData = Date.now() - lastDataReceivedTime;
          const needsReload = !currentData ||
                             currentData.tabId !== activeTab.id ||
                             timeSinceLastData > 2000;

          console.log('[ARW Sidepanel] Tab updated:', {
            tabId: activeTab?.id,
            currentDataTabId: currentData?.tabId,
            timeSinceLastData,
            needsReload
          });

          if (activeTab && needsReload) {
            currentTabId = activeTab.id;
            loadInspectionData();
          }
        }, 500);
      }
    });

    // Listen for tab activation - switch data when changing tabs
    chrome.tabs.onActivated.addListener(async () => {
      const [activeTab] = await chrome.tabs.query({ active: true, currentWindow: true });

      // Only reload if we don't have data for this tab
      if (activeTab && (!currentData || currentData.tabId !== activeTab.id)) {
        currentTabId = activeTab.id;
        loadInspectionData();
      }
    });
  }

  /**
   * Load inspection data from background script
   */
  async function loadInspectionData() {
    // If we recently received data (within last 500ms), don't reload
    const timeSinceLastData = Date.now() - lastDataReceivedTime;
    if (currentData && timeSinceLastData < 500) {
      console.log('Skipping reload - data received recently');
      return;
    }

    showLoading();
    retryCount = 0; // Reset retry counter

    try {
      const response = await chrome.runtime.sendMessage({
        type: 'GET_INSPECTION_DATA'
      });

      if (response.status === 'success' && response.data) {
        currentData = response.data;
        currentTabId = response.data.tabId;
        retryCount = 0;
        lastDataReceivedTime = Date.now();
        displayResults(currentData);
      } else if (response.status === 'no_data' && response.message?.includes('progress')) {
        // Inspection in progress, wait and retry with exponential backoff
        if (retryCount < MAX_RETRIES) {
          retryCount++;
          const delay = Math.min(1000 * retryCount, 3000); // Max 3 second delay
          setTimeout(() => loadInspectionData(), delay);
        } else {
          showError('Inspection is taking longer than expected. Try refreshing the page.');
        }
      } else {
        showError(response.message || 'No inspection data available');
      }
    } catch (error) {
      showError(`Failed to load data: ${error.message}`);
    }
  }

  /**
   * Display inspection results
   */
  function displayResults(data) {
    hideLoading();

    // Update header
    updateHeader(data);

    // Update all tabs
    updateMachineViewTab(data);
    updateTokenCostTab(data);
    updateProtocolsTab(data);
    updateDiscoveryTab(data);
    updateGEOTab(data);

    // Restore the previously active tab (or default to machine-view)
    switchToTab(activeTabId || 'machine-view');
  }

  /**
   * Update header with page info and compliance
   */
  function updateHeader(data) {
    document.getElementById('page-url').textContent = data.url;
    document.getElementById('inspection-time').textContent =
      new Date(data.timestamp).toLocaleTimeString();

    // Update ARW compliance badge
    const badge = document.getElementById('compliance-badge');
    const badgeText = document.getElementById('badge-text');

    badge.classList.remove('badge-unknown', 'badge-compliant', 'badge-non-compliant');

    if (data.arwCompliant) {
      badge.classList.add('badge-compliant');
      badgeText.textContent = 'ARW Compliant';
    } else {
      badge.classList.add('badge-non-compliant');
      badgeText.textContent = 'Not Compliant';
    }

    // Update GEO score badge
    const geoBadge = document.getElementById('geo-badge');
    const geoBadgeText = document.getElementById('geo-badge-text');

    geoBadge.classList.remove('badge-unknown', 'badge-excellent', 'badge-good', 'badge-average', 'badge-poor');

    if (data.geo?.geoScore !== undefined) {
      const score = data.geo.geoScore;
      geoBadgeText.textContent = `GEO: ${score}`;

      if (score >= 80) {
        geoBadge.classList.add('badge-excellent');
      } else if (score >= 60) {
        geoBadge.classList.add('badge-good');
      } else if (score >= 40) {
        geoBadge.classList.add('badge-average');
      } else {
        geoBadge.classList.add('badge-poor');
      }
    } else {
      geoBadge.classList.add('badge-unknown');
      geoBadgeText.textContent = 'GEO: --';
    }
  }

  /**
   * Update Machine View tab
   */
  function updateMachineViewTab(data) {
    const statusIcon = document.getElementById('mv-status-icon');
    const statusText = document.getElementById('mv-status-text');
    const content = document.getElementById('machine-view-content');

    // Validate machine view content is actually markdown, not HTML
    const machineViewContent = data.machineViewContent;
    const isValidContent = machineViewContent &&
                           !isHTMLContent(machineViewContent) &&
                           machineViewContent.length > 0;

    if (isValidContent) {
      statusIcon.textContent = '✓';
      const mv = data.discoveries?.machineViews?.[0];
      statusText.textContent = mv?.url ? `Found: ${mv.url}` : 'Machine view available';
      content.textContent = machineViewContent;
      content.classList.remove('placeholder');
    } else if (data.discoveries?.machineViews?.length > 0) {
      const mv = data.discoveries.machineViews[0];
      statusIcon.textContent = '⚠️';
      statusText.textContent = `URL: ${mv.url} (content may be invalid)`;
      content.innerHTML = `<p class="placeholder">Machine view found but content appears to be HTML, not markdown.</p>`;
    } else {
      statusIcon.textContent = '✗';
      statusText.textContent = 'No machine view found';
      content.innerHTML = `<p class="placeholder">No machine view (.llm.md) available for this page.\n\nThe extension looks for:\n• &lt;link rel="machine-view" href="..."&gt;\n• /path/to/page.llm.md\n• /path/to/page.md</p>`;
    }
  }

  /**
   * Check if content looks like HTML
   */
  function isHTMLContent(content) {
    if (!content) return false;
    const trimmed = content.trim().toLowerCase();
    return trimmed.startsWith('<!doctype') ||
           trimmed.startsWith('<html') ||
           trimmed.startsWith('<head') ||
           trimmed.startsWith('<body') ||
           (trimmed.includes('<html') && trimmed.includes('</html>')) ||
           (trimmed.includes('<div') && trimmed.includes('</div>'));
  }

  /**
   * Update Token Cost tab
   */
  function updateTokenCostTab(data) {
    const htmlSize = data.pageSize || 0;
    const mvSize = data.machineViewContent?.length || data.machineViewSize || 0;

    // Calculate tokens
    const htmlTokens = Math.ceil(htmlSize / CHARS_PER_TOKEN);
    const mvTokens = Math.ceil(mvSize / CHARS_PER_TOKEN);

    // Calculate costs
    const htmlCost = (htmlTokens / 1000) * TOKEN_COST_PER_1K;
    const mvCost = (mvTokens / 1000) * TOKEN_COST_PER_1K;

    // Update HTML metrics
    document.getElementById('html-size').textContent = formatBytes(htmlSize);
    document.getElementById('html-tokens').textContent = formatNumber(htmlTokens);
    document.getElementById('html-cost').textContent = formatCurrency(htmlCost);

    // Update Machine View metrics
    document.getElementById('mv-size').textContent = mvSize > 0 ? formatBytes(mvSize) : 'N/A';
    document.getElementById('mv-tokens').textContent = mvSize > 0 ? formatNumber(mvTokens) : 'N/A';
    document.getElementById('mv-cost').textContent = mvSize > 0 ? formatCurrency(mvCost) : 'N/A';

    // Calculate savings
    if (mvSize > 0 && htmlSize > 0) {
      const sizeReduction = Math.round((1 - mvSize / htmlSize) * 100);
      const tokenReduction = Math.round((1 - mvTokens / htmlTokens) * 100);
      const costSavings = htmlCost - mvCost;

      document.getElementById('size-reduction').textContent = `${sizeReduction}%`;
      document.getElementById('token-reduction').textContent = `${tokenReduction}%`;
      document.getElementById('cost-savings').textContent = formatCurrency(costSavings);

      document.getElementById('savings-summary').style.display = 'block';
    } else {
      document.getElementById('savings-summary').style.display = 'none';
    }

    // Update projections
    updateProjections();
  }

  /**
   * Update monthly projections
   */
  function updateProjections() {
    const requests = parseInt(document.getElementById('monthly-requests')?.value) || 10000;

    if (!currentData) return;

    const htmlSize = currentData.pageSize || 0;
    const mvSize = currentData.machineViewContent?.length || currentData.machineViewSize || 0;

    if (mvSize > 0 && htmlSize > 0) {
      const htmlTokens = Math.ceil(htmlSize / CHARS_PER_TOKEN);
      const mvTokens = Math.ceil(mvSize / CHARS_PER_TOKEN);

      const bandwidthSaved = (htmlSize - mvSize) * requests;
      const tokensSaved = (htmlTokens - mvTokens) * requests;
      const costSaved = (tokensSaved / 1000) * TOKEN_COST_PER_1K;

      document.getElementById('bandwidth-saved').textContent = formatBytes(bandwidthSaved);
      document.getElementById('tokens-saved').textContent = formatNumber(tokensSaved);
      document.getElementById('cost-saved').textContent = formatCurrency(costSaved);
    } else {
      document.getElementById('bandwidth-saved').textContent = 'N/A';
      document.getElementById('tokens-saved').textContent = 'N/A';
      document.getElementById('cost-saved').textContent = 'N/A';
    }
  }

  /**
   * Update Protocols tab
   */
  function updateProtocolsTab(data) {
    // Update AI Headers
    const headersList = document.getElementById('ai-headers-list');
    if (data.aiHeaders && data.aiHeaders.length > 0) {
      headersList.innerHTML = data.aiHeaders.map(header => `
        <div class="protocol-item">
          <span class="protocol-name">${header.name}</span>
          <span class="protocol-value">${header.value}</span>
        </div>
      `).join('');
    } else {
      headersList.innerHTML = '<p class="placeholder">No AI headers detected</p>';
    }

    // Update permissions
    updatePermission('training', data.permissions?.training);
    updatePermission('inference', data.permissions?.inference);
    updatePermission('attribution', data.permissions?.attribution);
    updatePermission('ratelimit', data.permissions?.rateLimit);

    // Update actions
    const actionsList = document.getElementById('actions-list');
    if (data.actions && data.actions.length > 0) {
      actionsList.innerHTML = data.actions.map(action => `
        <div class="action-item">
          <div class="action-name">${action.name}</div>
          <div class="action-desc">${action.description || ''}</div>
          <div class="action-endpoint">${action.method || 'GET'} ${action.endpoint}</div>
        </div>
      `).join('');
    } else {
      actionsList.innerHTML = '<p class="placeholder">No actions available</p>';
    }

    // Update auth section
    const authSection = document.getElementById('auth-section');
    if (data.auth) {
      authSection.innerHTML = `
        <div class="auth-item">
          <span class="permission-icon">🔐</span>
          <span class="permission-label">Type</span>
          <span class="permission-value">${data.auth.type || 'Unknown'}</span>
        </div>
        ${data.auth.endpoint ? `
          <div class="auth-item">
            <span class="permission-icon">🔗</span>
            <span class="permission-label">Endpoint</span>
            <span class="permission-value">${data.auth.endpoint}</span>
          </div>
        ` : ''}
      `;
    } else {
      authSection.innerHTML = '<p class="placeholder">No authentication protocols detected</p>';
    }
  }

  /**
   * Update individual permission display
   */
  function updatePermission(type, value) {
    const icon = document.getElementById(`${type}-icon`);
    const valueEl = document.getElementById(`${type}-value`);

    if (value === true || value === 'allowed') {
      icon.textContent = '✓';
      valueEl.textContent = 'Allowed';
      valueEl.style.color = '#22c55e';
    } else if (value === false || value === 'disallowed') {
      icon.textContent = '✗';
      valueEl.textContent = 'Disallowed';
      valueEl.style.color = '#ef4444';
    } else if (value === 'required') {
      icon.textContent = '⚠️';
      valueEl.textContent = 'Required';
      valueEl.style.color = '#f59e0b';
    } else if (value) {
      icon.textContent = 'ℹ️';
      valueEl.textContent = value;
      valueEl.style.color = '#64748b';
    } else {
      icon.textContent = '⭕';
      valueEl.textContent = 'Unknown';
      valueEl.style.color = '#94a3b8';
    }
  }

  /**
   * Update Discovery tab
   */
  function updateDiscoveryTab(data) {
    // llms.txt
    updateDiscoveryItem('llms', data.discoveries?.llmsTxt);

    // .well-known
    const wellKnownFound = data.discoveries?.wellKnown?.manifest?.exists;
    updateDiscoveryItem('wellknown', {
      exists: wellKnownFound,
      url: data.discoveries?.wellKnown?.manifest?.url
    });

    // robots.txt
    updateDiscoveryItem('robots', data.discoveries?.robotsTxt);

    // Machine views
    const machineViews = data.discoveries?.machineViews || [];
    updateDiscoveryItem('machineview', {
      exists: machineViews.length > 0,
      count: machineViews.length,
      urls: machineViews.map(mv => mv.url)
    });

    // Meta tags
    const metaTags = data.discoveries?.metaTags || [];
    updateDiscoveryItem('meta', {
      exists: metaTags.length > 0,
      count: metaTags.length,
      tags: metaTags
    });

    // Content structure
    if (data.discoveries?.wellKnown?.manifest?.data?.content) {
      const contentStructure = document.getElementById('content-structure');
      const contentTree = document.getElementById('content-tree');

      contentStructure.style.display = 'block';
      contentTree.textContent = JSON.stringify(
        data.discoveries.wellKnown.manifest.data.content,
        null,
        2
      );
    }
  }

  /**
   * Update individual discovery item
   */
  function updateDiscoveryItem(id, data) {
    const icon = document.getElementById(`${id}-icon`);
    const status = document.getElementById(`${id}-status`);
    const details = document.getElementById(`${id}-details`);

    if (data?.exists) {
      icon.textContent = '✓';
      status.textContent = data.count ? `${data.count} Found` : 'Found';
      status.classList.add('found');
      status.classList.remove('not-found');

      // Build details HTML
      let detailsHtml = '';
      if (data.url) {
        detailsHtml += `<strong>URL:</strong> <a href="${data.url}" target="_blank">${data.url}</a><br>`;
      }
      if (data.urls) {
        detailsHtml += data.urls.map(url =>
          `<a href="${url}" target="_blank">${url}</a>`
        ).join('<br>');
      }
      if (data.size) {
        detailsHtml += `<strong>Size:</strong> ${formatBytes(data.size)}<br>`;
      }
      if (data.hasArwHints !== undefined) {
        detailsHtml += `<strong>ARW Hints:</strong> ${data.hasArwHints ? 'Yes' : 'No'}<br>`;
      }
      if (data.tags) {
        detailsHtml += data.tags.map(tag =>
          `<code>&lt;${tag.tag} ${tag.name}="${tag.content}"&gt;</code>`
        ).join('<br>');
      }

      details.innerHTML = detailsHtml || '';
      details.style.display = detailsHtml ? 'block' : 'none';
    } else {
      icon.textContent = '✗';
      status.textContent = 'Not Found';
      status.classList.add('not-found');
      status.classList.remove('found');

      if (data?.error) {
        details.innerHTML = `<span style="color: #ef4444;">Error: ${data.error}</span>`;
        details.style.display = 'block';
      } else {
        details.style.display = 'none';
      }
    }
  }

  /**
   * Copy machine view content to clipboard
   */
  async function copyMachineView() {
    const content = document.getElementById('machine-view-content').textContent;
    if (content && !content.includes('No machine view')) {
      await navigator.clipboard.writeText(content);
      const btn = document.getElementById('copy-machine-view');
      btn.textContent = 'Copied!';
      setTimeout(() => btn.textContent = 'Copy', 2000);
    }
  }

  /**
   * Open machine view in new tab
   */
  function openMachineView() {
    if (currentData?.discoveries?.machineViews?.[0]?.url) {
      chrome.tabs.create({ url: currentData.discoveries.machineViews[0].url });
    }
  }

  /**
   * Show loading state
   */
  function showLoading() {
    document.getElementById('loading-state').style.display = 'flex';
    document.getElementById('error-state').style.display = 'none';
    document.querySelectorAll('.tab-content').forEach(el => el.style.display = 'none');
  }

  /**
   * Hide loading state
   */
  function hideLoading() {
    document.getElementById('loading-state').style.display = 'none';
  }

  /**
   * Show error state
   */
  function showError(message) {
    document.getElementById('loading-state').style.display = 'none';
    document.getElementById('error-state').style.display = 'flex';
    document.getElementById('error-message').textContent = message;
    document.querySelectorAll('.tab-content').forEach(el => el.style.display = 'none');
  }

  /**
   * Format bytes to human readable
   */
  function formatBytes(bytes) {
    if (!bytes || bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return Math.round(bytes / Math.pow(k, i) * 100) / 100 + ' ' + sizes[i];
  }

  /**
   * Format number with commas
   */
  function formatNumber(num) {
    return num.toLocaleString();
  }

  /**
   * Format currency
   */
  function formatCurrency(amount) {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD',
      minimumFractionDigits: 2,
      maximumFractionDigits: 4
    }).format(amount);
  }

  /**
   * Update GEO tab with analysis results
   */
  function updateGEOTab(data) {
    const scoreValue = document.getElementById('geo-score-value');
    const scoreRing = document.getElementById('geo-score-ring');
    const geoStatus = document.getElementById('geo-status');

    // Subscore elements
    const authorityEl = document.getElementById('geo-authority');
    const evidenceEl = document.getElementById('geo-evidence');
    const semanticEl = document.getElementById('geo-semantic');
    const arwReadinessEl = document.getElementById('geo-arw-readiness');

    // Detail elements
    const extLinksEl = document.getElementById('geo-ext-links');
    const extDomainsEl = document.getElementById('geo-ext-domains');
    const statisticsEl = document.getElementById('geo-statistics');
    const quotationsEl = document.getElementById('geo-quotations');
    const entitiesEl = document.getElementById('geo-entities');
    const wordCountEl = document.getElementById('geo-word-count');

    // Check if we have GEO data
    if (data.geo && data.geo.geoScore !== undefined) {
      // Display GEO score
      const score = data.geo.geoScore;
      scoreValue.textContent = score;
      animateScoreRing(scoreRing, score);

      // Display subscores
      if (data.geo.subscores) {
        authorityEl.textContent = data.geo.subscores.authority || 0;
        evidenceEl.textContent = data.geo.subscores.evidence || 0;
        semanticEl.textContent = data.geo.subscores.semanticClarity || 0;
        arwReadinessEl.textContent = data.geo.subscores.arwReadiness || 0;
      }

      // Display detailed metrics
      if (data.geo.metrics) {
        extLinksEl.textContent = data.geo.metrics.citations?.externalLinks || 0;
        extDomainsEl.textContent = data.geo.metrics.citations?.externalDomains?.length || 0;
        statisticsEl.textContent = data.geo.metrics.statistics?.total || 0;

        const quotesTotal = (data.geo.metrics.quotations?.blockquoteCount || 0) +
                           (data.geo.metrics.quotations?.inlineQuoteCount || 0);
        quotationsEl.textContent = quotesTotal;

        entitiesEl.textContent = data.geo.metrics.entities?.totalEntities || 0;
        wordCountEl.textContent = formatNumber(data.geo.metrics.structure?.wordCount || 0);
      }

      geoStatus.textContent = 'Analysis complete';
    } else {
      // No GEO data yet
      scoreValue.textContent = '--';
      scoreRing.style.strokeDashoffset = '565';

      authorityEl.textContent = '--';
      evidenceEl.textContent = '--';
      semanticEl.textContent = '--';
      arwReadinessEl.textContent = '--';

      extLinksEl.textContent = '--';
      extDomainsEl.textContent = '--';
      statisticsEl.textContent = '--';
      quotationsEl.textContent = '--';
      entitiesEl.textContent = '--';
      wordCountEl.textContent = '--';

      geoStatus.textContent = 'Analyzing...';
    }
  }

  /**
   * Animate the GEO score ring
   */
  function animateScoreRing(ringElement, score) {
    if (!ringElement) return;

    // Circle circumference = 2 * PI * radius
    // radius = 90 (from SVG), so circumference = 565.48...
    const circumference = 565;
    const progress = (score / 100) * circumference;
    const offset = circumference - progress;

    // Update stroke-dashoffset to show progress
    ringElement.style.strokeDashoffset = offset;

    // Update color based on score
    ringElement.classList.remove('score-excellent', 'score-good', 'score-average', 'score-poor');
    if (score >= 80) {
      ringElement.classList.add('score-excellent');
    } else if (score >= 60) {
      ringElement.classList.add('score-good');
    } else if (score >= 40) {
      ringElement.classList.add('score-average');
    } else {
      ringElement.classList.add('score-poor');
    }
  }

  /**
   * Open full GEO analysis in ARW Inspector
   */
  function openFullAnalysis() {
    const url = currentData?.url;
    if (url) {
      // Open ARW Inspector app with the current URL
      const inspectorUrl = `http://localhost:3003?url=${encodeURIComponent(url)}`;
      chrome.tabs.create({ url: inspectorUrl });
    } else {
      alert('No URL available for analysis');
    }
  }

})();
