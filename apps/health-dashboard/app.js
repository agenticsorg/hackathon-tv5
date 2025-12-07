(function() {
    'use strict';

    const REFRESH_INTERVAL = 10000; // 10 seconds
    const API_ENDPOINT = '/health/aggregate';

    let refreshTimer = null;
    let countdownTimer = null;
    let countdownSeconds = 10;

    const elements = {
        overallStatus: document.getElementById('overall-status'),
        overallStatusText: document.getElementById('overall-status-text'),
        lastUpdateTime: document.getElementById('last-update-time'),
        servicesHealthy: document.getElementById('services-healthy'),
        servicesTotal: document.getElementById('services-total'),
        depsHealthy: document.getElementById('deps-healthy'),
        depsTotal: document.getElementById('deps-total'),
        overallLatency: document.getElementById('overall-latency'),
        servicesGrid: document.getElementById('services-grid'),
        dependenciesGrid: document.getElementById('dependencies-grid'),
        errorContainer: document.getElementById('error-container'),
        errorMessageText: document.getElementById('error-message-text'),
        refreshCountdown: document.getElementById('refresh-countdown')
    };

    function formatTimestamp(timestamp) {
        const date = new Date(timestamp);
        const hours = String(date.getHours()).padStart(2, '0');
        const minutes = String(date.getMinutes()).padStart(2, '0');
        const seconds = String(date.getSeconds()).padStart(2, '0');
        return `${hours}:${minutes}:${seconds}`;
    }

    function getStatusClass(status) {
        return status.toLowerCase();
    }

    function showError(message) {
        elements.errorMessageText.textContent = message;
        elements.errorContainer.style.display = 'block';
        setTimeout(() => {
            elements.errorContainer.style.display = 'none';
        }, 5000);
    }

    function createServiceCard(service) {
        const card = document.createElement('div');
        card.className = `card ${getStatusClass(service.status)}`;

        const latencyText = service.latency_ms !== null && service.latency_ms !== undefined
            ? `${service.latency_ms}ms`
            : 'N/A';

        const lastChecked = formatTimestamp(service.last_checked);

        let errorHtml = '';
        if (service.error) {
            errorHtml = `<div class="card-error">${escapeHtml(service.error)}</div>`;
        }

        card.innerHTML = `
            <div class="card-header">
                <div class="card-title">${escapeHtml(service.name)}</div>
                <div class="card-status ${getStatusClass(service.status)}">
                    <span class="status-dot ${getStatusClass(service.status)}"></span>
                    ${service.status}
                </div>
            </div>
            <div class="card-body">
                <div class="card-metric">
                    <span class="card-metric-label">Response Time</span>
                    <span class="card-metric-value">${latencyText}</span>
                </div>
                <div class="card-metric">
                    <span class="card-metric-label">Last Checked</span>
                    <span class="card-metric-value">${lastChecked}</span>
                </div>
            </div>
            ${errorHtml}
        `;

        return card;
    }

    function createDependencyCard(dependency) {
        const card = document.createElement('div');
        card.className = `card ${getStatusClass(dependency.status)}`;

        const latencyText = dependency.latency_ms !== null && dependency.latency_ms !== undefined
            ? `${dependency.latency_ms}ms`
            : 'N/A';

        const lastChecked = formatTimestamp(dependency.last_checked);

        let errorHtml = '';
        if (dependency.error) {
            errorHtml = `<div class="card-error">${escapeHtml(dependency.error)}</div>`;
        }

        card.innerHTML = `
            <div class="card-header">
                <div class="card-title">${escapeHtml(dependency.name)}</div>
                <div class="card-status ${getStatusClass(dependency.status)}">
                    <span class="status-dot ${getStatusClass(dependency.status)}"></span>
                    ${dependency.status}
                </div>
            </div>
            <div class="card-body">
                <div class="card-metric">
                    <span class="card-metric-label">Response Time</span>
                    <span class="card-metric-value">${latencyText}</span>
                </div>
                <div class="card-metric">
                    <span class="card-metric-label">Last Checked</span>
                    <span class="card-metric-value">${lastChecked}</span>
                </div>
            </div>
            ${errorHtml}
        `;

        return card;
    }

    function escapeHtml(text) {
        const map = {
            '&': '&amp;',
            '<': '&lt;',
            '>': '&gt;',
            '"': '&quot;',
            "'": '&#039;'
        };
        return text.replace(/[&<>"']/g, m => map[m]);
    }

    function updateDashboard(data) {
        // Update overall status
        const statusClass = getStatusClass(data.status);
        elements.overallStatus.className = `status-badge ${statusClass}`;
        elements.overallStatusText.textContent = data.status;

        // Update timestamp
        elements.lastUpdateTime.textContent = formatTimestamp(data.timestamp);

        // Update metrics
        const healthyServices = data.services.filter(s => s.status === 'healthy').length;
        elements.servicesHealthy.textContent = healthyServices;
        elements.servicesTotal.textContent = data.services.length;

        const healthyDeps = data.dependencies.filter(d => d.status === 'healthy').length;
        elements.depsHealthy.textContent = healthyDeps;
        elements.depsTotal.textContent = data.dependencies.length;

        elements.overallLatency.textContent = data.overall_latency_ms;

        // Update services grid
        elements.servicesGrid.innerHTML = '';
        data.services.forEach(service => {
            elements.servicesGrid.appendChild(createServiceCard(service));
        });

        // Update dependencies grid
        elements.dependenciesGrid.innerHTML = '';
        data.dependencies.forEach(dependency => {
            elements.dependenciesGrid.appendChild(createDependencyCard(dependency));
        });
    }

    function startCountdown() {
        countdownSeconds = 10;
        elements.refreshCountdown.textContent = countdownSeconds;

        if (countdownTimer) {
            clearInterval(countdownTimer);
        }

        countdownTimer = setInterval(() => {
            countdownSeconds--;
            elements.refreshCountdown.textContent = countdownSeconds;
            if (countdownSeconds <= 0) {
                countdownSeconds = 10;
            }
        }, 1000);
    }

    async function fetchHealth() {
        try {
            const response = await fetch(API_ENDPOINT);

            if (!response.ok) {
                throw new Error(`HTTP ${response.status}: ${response.statusText}`);
            }

            const data = await response.json();
            updateDashboard(data);
            startCountdown();
        } catch (error) {
            console.error('Failed to fetch health data:', error);
            showError(`Failed to fetch health data: ${error.message}`);
        }
    }

    function startAutoRefresh() {
        if (refreshTimer) {
            clearInterval(refreshTimer);
        }

        refreshTimer = setInterval(fetchHealth, REFRESH_INTERVAL);
    }

    function init() {
        // Initial fetch
        fetchHealth();

        // Start auto-refresh
        startAutoRefresh();
        startCountdown();

        // Handle visibility change to pause/resume when tab is hidden
        document.addEventListener('visibilitychange', () => {
            if (document.hidden) {
                if (refreshTimer) {
                    clearInterval(refreshTimer);
                }
                if (countdownTimer) {
                    clearInterval(countdownTimer);
                }
            } else {
                fetchHealth();
                startAutoRefresh();
                startCountdown();
            }
        });
    }

    // Start when DOM is ready
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }
})();
