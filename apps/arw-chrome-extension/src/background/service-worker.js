/**
 * ARW Background Service Worker
 * Coordinates extension behavior and manages inspection data
 */

// Store inspection results per tab
const tabInspections = new Map();

/**
 * Set up extension on install
 */
chrome.runtime.onInstalled.addListener(async (details) => {
  if (details.reason === 'install') {
    console.log('ARW Inspector installed successfully');
    chrome.action.setBadgeBackgroundColor({ color: '#94a3b8' });
  } else if (details.reason === 'update') {
    console.log('ARW Inspector updated to version', chrome.runtime.getManifest().version);
  }
});

/**
 * Open side panel when extension icon is clicked
 */
chrome.action.onClicked.addListener(async (tab) => {
  try {
    await chrome.sidePanel.open({ windowId: tab.windowId });
    console.log('Side panel opened for tab', tab.id);
  } catch (error) {
    console.error('Failed to open side panel:', error);
  }
});

/**
 * Handle messages from content scripts and side panel
 */
chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  switch (message.type) {
    case 'ARW_INSPECTION_COMPLETE':
      handleInspectionComplete(message.data, sender);
      sendResponse({ status: 'received' });
      break;

    case 'UPDATE_BADGE':
      handleBadgeUpdate(message.data, sender);
      sendResponse({ status: 'updated' });
      break;

    case 'GET_INSPECTION_DATA':
      handleGetInspectionData(sender, sendResponse);
      return true; // Keep channel open for async response

    case 'FETCH_MACHINE_VIEW':
      handleFetchMachineView(message.url, sendResponse);
      return true;

    case 'STORE_GEO_ANALYSIS':
      handleStoreGeoAnalysis(message.data, sender);
      sendResponse({ status: 'stored' });
      break;

    default:
      sendResponse({ status: 'unknown_message_type' });
  }
  return false;
});

/**
 * Handle completed inspection from content script
 */
function handleInspectionComplete(data, sender) {
  if (sender.tab?.id) {
    // Store inspection data for this tab
    tabInspections.set(sender.tab.id, {
      ...data,
      tabId: sender.tab.id,
      tabUrl: sender.tab.url
    });

    console.log('ARW Inspection Complete:', {
      tabId: sender.tab.id,
      url: data.url,
      arwCompliant: data.arwCompliant,
      pageSize: data.pageSize,
      machineViewSize: data.machineViewSize
    });

    // Notify side panel of new data
    notifySidePanel(sender.tab.id, data);
  }
}

/**
 * Store GEO analysis results
 */
function handleStoreGeoAnalysis(geoData, sender) {
  if (sender.tab?.id && tabInspections.has(sender.tab.id)) {
    const existingData = tabInspections.get(sender.tab.id);
    tabInspections.set(sender.tab.id, {
      ...existingData,
      geo: geoData
    });

    console.log('GEO Analysis Stored:', {
      tabId: sender.tab.id,
      score: geoData.overall?.score,
      usedLLM: geoData.usedLLM
    });

    // Notify side panel of updated data
    notifySidePanel(sender.tab.id, tabInspections.get(sender.tab.id));
  }
}

/**
 * Notify side panel of new inspection data
 */
async function notifySidePanel(tabId, data) {
  try {
    await chrome.runtime.sendMessage({
      type: 'INSPECTION_UPDATE',
      tabId: tabId,
      data: data
    });
  } catch (error) {
    // Side panel may not be open, ignore
  }
}

/**
 * Update badge for specific tab
 */
function handleBadgeUpdate(data, sender) {
  if (sender.tab?.id) {
    chrome.action.setBadgeText({
      tabId: sender.tab.id,
      text: data.text
    });

    chrome.action.setBadgeBackgroundColor({
      tabId: sender.tab.id,
      color: data.color
    });
  }
}

/**
 * Get inspection data for side panel
 */
async function handleGetInspectionData(sender, sendResponse) {
  try {
    // Get current active tab
    const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });

    if (tab?.id && tabInspections.has(tab.id)) {
      const inspectionData = tabInspections.get(tab.id);
      sendResponse({
        status: 'success',
        data: inspectionData
      });
    } else if (tab?.id) {
      // No data yet - content script should be running automatically
      // Just return no_data status and let the sidepanel wait for the INSPECTION_UPDATE message
      sendResponse({
        status: 'no_data',
        message: 'Inspection in progress...',
        tabId: tab.id
      });
    } else {
      sendResponse({
        status: 'no_data',
        message: 'No active tab found'
      });
    }
  } catch (error) {
    sendResponse({
      status: 'error',
      message: error.message
    });
  }
}

/**
 * Fetch machine view content (handles CORS)
 */
async function handleFetchMachineView(url, sendResponse) {
  try {
    const response = await fetch(url);
    if (response.ok) {
      const content = await response.text();
      sendResponse({
        status: 'success',
        content: content,
        size: content.length
      });
    } else {
      sendResponse({
        status: 'error',
        message: `HTTP ${response.status}`
      });
    }
  } catch (error) {
    sendResponse({
      status: 'error',
      message: error.message
    });
  }
}

/**
 * Clean up data when tab is closed
 */
chrome.tabs.onRemoved.addListener(async (tabId) => {
  if (tabInspections.has(tabId)) {
    tabInspections.delete(tabId);
    console.log(`Cleaned up inspection data for tab ${tabId}`);
  }
});

/**
 * Handle tab updates (navigation)
 */
chrome.tabs.onUpdated.addListener((tabId, changeInfo, tab) => {
  if (changeInfo.status === 'loading') {
    // Clear old inspection data when navigating to new page
    if (tabInspections.has(tabId)) {
      tabInspections.delete(tabId);
    }

    // Reset badge
    chrome.action.setBadgeText({ tabId, text: '' });
  }
});

/**
 * Handle tab activation - refresh side panel data
 */
chrome.tabs.onActivated.addListener(async (activeInfo) => {
  try {
    if (tabInspections.has(activeInfo.tabId)) {
      const data = tabInspections.get(activeInfo.tabId);
      notifySidePanel(activeInfo.tabId, data);
    }
  } catch (error) {
    // Ignore errors
  }
});

console.log('ARW Inspector Background Service Worker initialized (v2.0)');
