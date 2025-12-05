/**
 * ARW Inspector Popup
 * Displays inspection results in the extension popup
 */

(function() {
  'use strict';

  // Get inspection data when popup opens
  document.addEventListener('DOMContentLoaded', async () => {
    try {
      const response = await chrome.runtime.sendMessage({
        type: 'GET_INSPECTION_DATA'
      });

      if (response.status === 'success' && response.data) {
        displayResults(response.data);
      } else {
        showError(response.message || 'No inspection data available for this page');
      }
    } catch (error) {
      showError(`Failed to load inspection data: ${error.message}`);
    }
  });

  /**
   * Display inspection results
   */
  function displayResults(data) {
    // Hide loading, show results
    document.getElementById('loading-state').style.display = 'none';
    document.getElementById('results-container').style.display = 'block';

    // Update compliance badge
    updateComplianceBadge(data.arwCompliant);

    // Update page info
    updatePageInfo(data);

    // Update features
    updateLlmsTxt(data.discoveries.llmsTxt);
    updateWellKnown(data.discoveries.wellKnown);
    updateMachineViews(data.discoveries.machineViews);
    updateRobotsTxt(data.discoveries.robotsTxt);
    updateMetaTags(data.discoveries.metaTags);

    // Show errors if any
    if (data.errors && data.errors.length > 0) {
      showErrors(data.errors);
    }
  }

  /**
   * Update compliance badge
   */
  function updateComplianceBadge(isCompliant) {
    const badge = document.getElementById('compliance-badge');
    const badgeText = document.getElementById('badge-text');

    badge.classList.remove('badge-unknown', 'badge-compliant', 'badge-non-compliant');

    if (isCompliant) {
      badge.classList.add('badge-compliant');
      badgeText.textContent = '✓ ARW Compliant';
    } else {
      badge.classList.add('badge-non-compliant');
      badgeText.textContent = '✗ Not Compliant';
    }
  }

  /**
   * Update page information
   */
  function updatePageInfo(data) {
    document.getElementById('page-url').textContent = data.url;
    document.getElementById('inspection-time').textContent =
      new Date(data.timestamp).toLocaleTimeString();
  }

  /**
   * Update llms.txt status
   */
  function updateLlmsTxt(llmsTxt) {
    const icon = document.getElementById('llms-icon');
    const status = document.getElementById('llms-status');
    const details = document.getElementById('llms-details');

    if (llmsTxt?.exists) {
      icon.textContent = '✓';
      status.textContent = 'Found';
      status.classList.add('found');

      details.innerHTML = `
        <div class="detail-item">
          <strong>URL:</strong> <a href="${llmsTxt.url}" target="_blank">${llmsTxt.url}</a><br>
          <strong>Size:</strong> ${formatBytes(llmsTxt.size)}<br>
          <strong>Has Content:</strong> ${llmsTxt.hasContent ? 'Yes' : 'No'}
        </div>
      `;
    } else {
      icon.textContent = '✗';
      status.textContent = 'Not Found';
      if (llmsTxt?.error) {
        details.innerHTML = `<div style="color: #dc2626; font-size: 11px;">Error: ${llmsTxt.error}</div>`;
      }
    }
  }

  /**
   * Update .well-known files status
   */
  function updateWellKnown(wellKnown) {
    const icon = document.getElementById('wellknown-icon');
    const status = document.getElementById('wellknown-status');
    const details = document.getElementById('wellknown-details');

    const foundFiles = Object.entries(wellKnown).filter(([_, data]) => data.exists);

    if (foundFiles.length > 0) {
      icon.textContent = '✓';
      status.textContent = `${foundFiles.length} Found`;
      status.classList.add('found');

      const filesList = foundFiles.map(([key, data]) =>
        `<li><strong>${key}:</strong> <a href="${data.url}" target="_blank">${data.url}</a></li>`
      ).join('');

      details.innerHTML = `<ul>${filesList}</ul>`;
    } else {
      icon.textContent = '✗';
      status.textContent = 'Not Found';
    }
  }

  /**
   * Update machine views status
   */
  function updateMachineViews(machineViews) {
    const icon = document.getElementById('machineview-icon');
    const status = document.getElementById('machineview-status');
    const details = document.getElementById('machineview-details');

    if (machineViews && machineViews.length > 0) {
      icon.textContent = '✓';
      status.textContent = `${machineViews.length} Found`;
      status.classList.add('found');

      const viewsList = machineViews.map(view =>
        `<li><a href="${view.url}" target="_blank">${view.url}</a></li>`
      ).join('');

      details.innerHTML = `<ul>${viewsList}</ul>`;
    } else {
      icon.textContent = '✗';
      status.textContent = 'Not Found';
      details.innerHTML = '<div style="font-size: 11px; color: #64748b;">No machine view found for this page</div>';
    }
  }

  /**
   * Update robots.txt status
   */
  function updateRobotsTxt(robotsTxt) {
    const icon = document.getElementById('robots-icon');
    const status = document.getElementById('robots-status');
    const details = document.getElementById('robots-details');

    if (robotsTxt?.exists) {
      icon.textContent = robotsTxt.hasArwHints ? '✓' : '⚠️';
      status.textContent = 'Found';
      status.classList.add('found');

      details.innerHTML = `
        <div class="detail-item">
          <strong>URL:</strong> <a href="${robotsTxt.url}" target="_blank">${robotsTxt.url}</a><br>
          <strong>ARW Hints:</strong> ${robotsTxt.hasArwHints ? 'Yes' : 'No'}
        </div>
      `;
    } else {
      icon.textContent = '✗';
      status.textContent = 'Not Found';
    }
  }

  /**
   * Update meta tags status
   */
  function updateMetaTags(metaTags) {
    const container = document.getElementById('metatags-container');
    const count = document.getElementById('metatags-count');
    const details = document.getElementById('metatags-details');

    if (metaTags && metaTags.length > 0) {
      container.style.display = 'block';
      count.textContent = `${metaTags.length} found`;

      const tagsList = metaTags.map(tag =>
        `<li><strong>&lt;${tag.tag}&gt;</strong> ${tag.name}: ${tag.content}</li>`
      ).join('');

      details.innerHTML = `<ul>${tagsList}</ul>`;
    }
  }

  /**
   * Show error state
   */
  function showError(message) {
    document.getElementById('loading-state').style.display = 'none';
    document.getElementById('error-state').style.display = 'flex';
    document.getElementById('error-message').textContent = message;
  }

  /**
   * Show errors section
   */
  function showErrors(errors) {
    const section = document.getElementById('errors-section');
    const list = document.getElementById('errors-list');

    section.style.display = 'block';
    list.innerHTML = errors.map(err =>
      `<div class="error-item"><strong>Error:</strong> ${err.message}</div>`
    ).join('');
  }

  /**
   * Format bytes to human readable
   */
  function formatBytes(bytes) {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return Math.round(bytes / Math.pow(k, i) * 100) / 100 + ' ' + sizes[i];
  }

})();
