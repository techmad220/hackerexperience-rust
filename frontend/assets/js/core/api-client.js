/**
 * HackerExperience API Client
 * Comprehensive 1:1 port of all PHP AJAX handlers with modern enhancements
 * 
 * This client provides a complete interface to all backend endpoints,
 * matching the original PHP functionality while adding modern features
 * like automatic retries, request queuing, and WebSocket fallback.
 */

class HackerExperienceAPI {
    constructor(config = {}) {
        this.config = {
            baseUrl: config.baseUrl || window.GAME_CONFIG?.apiUrl || '/api',
            wsUrl: config.wsUrl || window.GAME_CONFIG?.wsUrl || 'ws://localhost:8080/ws',
            timeout: config.timeout || 30000,
            maxRetries: config.maxRetries || 3,
            retryDelay: config.retryDelay || 1000,
            requestQueue: true,
            debug: config.debug || window.GAME_CONFIG?.debug || false,
            ...config
        };

        this.requestQueue = [];
        this.pendingRequests = new Map();
        this.csrfToken = window.GAME_CONFIG?.csrf_token || '';
        this.sessionId = window.GAME_CONFIG?.session_id || '';
        
        this.init();
    }

    init() {
        // Set up CSRF token refresh
        this.refreshCSRFToken();
        
        // Set up request interceptors
        this.setupInterceptors();
        
        // Initialize WebSocket connection for real-time features
        if (this.config.wsUrl) {
            this.initWebSocket();
        }
        
        if (this.config.debug) {
            console.log('[API] HackerExperience API Client initialized', this.config);
        }
    }

    setupInterceptors() {
        // Global error handling
        window.addEventListener('error', (event) => {
            if (this.config.debug) {
                console.error('[API] Global error:', event.error);
            }
        });
    }

    async refreshCSRFToken() {
        try {
            const response = await this.request('GET', '/csrf-token');
            if (response.success && response.data.token) {
                this.csrfToken = response.data.token;
                window.GAME_CONFIG.csrf_token = this.csrfToken;
            }
        } catch (error) {
            console.warn('[API] Failed to refresh CSRF token:', error);
        }
    }

    initWebSocket() {
        try {
            this.ws = new WebSocket(this.config.wsUrl);
            this.ws.onopen = () => {
                if (this.config.debug) {
                    console.log('[API] WebSocket connected');
                }
            };
            this.ws.onmessage = (event) => {
                try {
                    const data = JSON.parse(event.data);
                    this.handleWebSocketMessage(data);
                } catch (error) {
                    console.error('[API] WebSocket message parse error:', error);
                }
            };
            this.ws.onclose = () => {
                if (this.config.debug) {
                    console.log('[API] WebSocket disconnected, attempting to reconnect...');
                }
                setTimeout(() => this.initWebSocket(), 5000);
            };
        } catch (error) {
            console.error('[API] WebSocket initialization failed:', error);
        }
    }

    handleWebSocketMessage(data) {
        // Emit custom events for real-time updates
        const event = new CustomEvent('gameUpdate', { detail: data });
        window.dispatchEvent(event);
        
        // Handle specific message types
        switch (data.type) {
            case 'process_complete':
                this.handleProcessComplete(data);
                break;
            case 'new_notification':
                this.handleNotification(data);
                break;
            case 'player_stats_update':
                this.handleStatsUpdate(data);
                break;
        }
    }

    // Core request method with retry logic and error handling
    async request(method, endpoint, data = null, options = {}) {
        const requestId = this.generateRequestId();
        
        try {
            // Add to pending requests for tracking
            const requestPromise = this._executeRequest(method, endpoint, data, options);
            this.pendingRequests.set(requestId, requestPromise);
            
            const result = await requestPromise;
            this.pendingRequests.delete(requestId);
            
            return result;
        } catch (error) {
            this.pendingRequests.delete(requestId);
            throw error;
        }
    }

    async _executeRequest(method, endpoint, data, options) {
        const url = `${this.config.baseUrl}${endpoint}`;
        const headers = {
            'Content-Type': 'application/json',
            'X-CSRF-Token': this.csrfToken,
            'X-Session-ID': this.sessionId,
            'X-Requested-With': 'XMLHttpRequest',
            ...options.headers
        };

        const requestOptions = {
            method,
            headers,
            signal: AbortSignal.timeout(this.config.timeout),
            ...options
        };

        if (data && (method === 'POST' || method === 'PUT' || method === 'PATCH')) {
            requestOptions.body = JSON.stringify(data);
        }

        let lastError;
        
        for (let attempt = 0; attempt <= this.config.maxRetries; attempt++) {
            try {
                if (this.config.debug) {
                    console.log(`[API] ${method} ${url}`, data);
                }

                const response = await fetch(url, requestOptions);
                const responseData = await response.json();

                if (response.ok) {
                    if (this.config.debug) {
                        console.log(`[API] Response:`, responseData);
                    }
                    return responseData;
                }

                // Handle specific error codes
                if (response.status === 401) {
                    this.handleUnauthorized();
                    throw new Error('Authentication required');
                }

                if (response.status === 403) {
                    await this.refreshCSRFToken();
                    throw new Error('Forbidden - CSRF token may be invalid');
                }

                if (response.status === 429) {
                    await this.delay(this.config.retryDelay * Math.pow(2, attempt));
                    continue;
                }

                throw new Error(responseData.message || `HTTP ${response.status}`);

            } catch (error) {
                lastError = error;
                
                if (attempt === this.config.maxRetries) {
                    break;
                }

                if (error.name === 'AbortError') {
                    break; // Don't retry timeouts
                }

                // Exponential backoff
                await this.delay(this.config.retryDelay * Math.pow(2, attempt));
            }
        }

        throw lastError;
    }

    // Authentication & Session Management (Original PHP: login.php, logout.php, etc.)
    async login(credentials) {
        const response = await this.request('POST', '/auth/login', {
            username: credentials.username,
            password: credentials.password,
            remember_me: credentials.rememberMe || false,
            captcha: credentials.captcha || null
        });

        if (response.success && response.data.session_id) {
            this.sessionId = response.data.session_id;
            window.GAME_CONFIG.session_id = this.sessionId;
        }

        return response;
    }

    async logout() {
        const response = await this.request('POST', '/auth/logout');
        
        if (response.success) {
            this.sessionId = '';
            window.GAME_CONFIG.session_id = '';
            window.location.href = '/';
        }

        return response;
    }

    async register(userData) {
        return await this.request('POST', '/auth/register', {
            username: userData.username,
            email: userData.email,
            password: userData.password,
            password_confirmation: userData.passwordConfirmation,
            terms_accepted: userData.termsAccepted,
            captcha: userData.captcha || null
        });
    }

    async forgotPassword(email) {
        return await this.request('POST', '/auth/forgot-password', { email });
    }

    async resetPassword(token, password) {
        return await this.request('POST', '/auth/reset-password', {
            token,
            password,
            password_confirmation: password
        });
    }

    // User Profile & Settings (Original PHP: profile.php, settings.php)
    async getProfile(userId = null) {
        const endpoint = userId ? `/users/${userId}/profile` : '/profile';
        return await this.request('GET', endpoint);
    }

    async updateProfile(profileData) {
        return await this.request('PUT', '/profile', profileData);
    }

    async getSettings() {
        return await this.request('GET', '/settings');
    }

    async updateSettings(settings) {
        return await this.request('PUT', '/settings', settings);
    }

    async changePassword(currentPassword, newPassword) {
        return await this.request('POST', '/auth/change-password', {
            current_password: currentPassword,
            new_password: newPassword,
            new_password_confirmation: newPassword
        });
    }

    // Process Management (Original PHP: processes.php, Process.class.php)
    async getProcesses() {
        return await this.request('GET', '/processes');
    }

    async createProcess(processData) {
        return await this.request('POST', '/processes', {
            action: processData.action,
            target_ip: processData.targetIp,
            software_id: processData.softwareId,
            parameters: processData.parameters || {}
        });
    }

    async getProcess(processId) {
        return await this.request('GET', `/processes/${processId}`);
    }

    async cancelProcess(processId) {
        return await this.request('DELETE', `/processes/${processId}`);
    }

    async pauseProcess(processId) {
        return await this.request('POST', `/processes/${processId}/pause`);
    }

    async resumeProcess(processId) {
        return await this.request('POST', `/processes/${processId}/resume`);
    }

    // Hardware Management (Original PHP: hardware.php)
    async getHardware() {
        return await this.request('GET', '/hardware');
    }

    async purchaseHardware(hardwareData) {
        return await this.request('POST', '/hardware', {
            type: hardwareData.type,
            quantity: hardwareData.quantity || 1,
            server_id: hardwareData.serverId || null
        });
    }

    async upgradeHardware(hardwareId, upgradeData) {
        return await this.request('PUT', `/hardware/${hardwareId}`, upgradeData);
    }

    async sellHardware(hardwareId) {
        return await this.request('DELETE', `/hardware/${hardwareId}`);
    }

    async getHardwareMarket() {
        return await this.request('GET', '/market/hardware');
    }

    // Software Management (Original PHP: software.php, createsoft.php)
    async getSoftware() {
        return await this.request('GET', '/software');
    }

    async createSoftware(softwareData) {
        return await this.request('POST', '/software', {
            name: softwareData.name,
            type: softwareData.type,
            version: softwareData.version || '1.0',
            description: softwareData.description || '',
            code: softwareData.code || ''
        });
    }

    async installSoftware(softwareId, serverId = null) {
        return await this.request('POST', `/software/${softwareId}/install`, {
            server_id: serverId
        });
    }

    async uninstallSoftware(softwareId) {
        return await this.request('DELETE', `/software/${softwareId}`);
    }

    async upgradeSoftware(softwareId) {
        return await this.request('POST', `/software/${softwareId}/upgrade`);
    }

    async getSoftwareMarket() {
        return await this.request('GET', '/market/software');
    }

    // Network Operations (Original PHP: internet.php)
    async scanNetwork(ip = null) {
        const params = ip ? `?target=${encodeURIComponent(ip)}` : '';
        return await this.request('GET', `/network/scan${params}`);
    }

    async connectToServer(ip) {
        return await this.request('POST', '/network/connect', { ip });
    }

    async disconnectFromServer() {
        return await this.request('POST', '/network/disconnect');
    }

    async getConnections() {
        return await this.request('GET', '/network/connections');
    }

    async traceroute(ip) {
        return await this.request('POST', '/network/traceroute', { ip });
    }

    async nslookup(domain) {
        return await this.request('POST', '/network/nslookup', { domain });
    }

    // Hacking Operations
    async hackServer(targetData) {
        return await this.request('POST', '/hack', {
            target_ip: targetData.ip,
            method: targetData.method || 'brute_force',
            software_id: targetData.softwareId || null,
            parameters: targetData.parameters || {}
        });
    }

    async uploadFile(targetIp, fileData) {
        return await this.request('POST', '/hack/upload', {
            target_ip: targetIp,
            file_name: fileData.name,
            file_content: fileData.content,
            file_type: fileData.type
        });
    }

    async downloadFile(targetIp, fileName) {
        return await this.request('POST', '/hack/download', {
            target_ip: targetIp,
            file_name: fileName
        });
    }

    async deleteFile(targetIp, fileName) {
        return await this.request('POST', '/hack/delete', {
            target_ip: targetIp,
            file_name: fileName
        });
    }

    // Banking & Finances (Original PHP: finances.php)
    async getBankAccounts() {
        return await this.request('GET', '/banking/accounts');
    }

    async transfer(transferData) {
        return await this.request('POST', '/banking/transfer', {
            from_account: transferData.fromAccount,
            to_account: transferData.toAccount,
            amount: transferData.amount,
            description: transferData.description || ''
        });
    }

    async getTransactionHistory(accountId = null) {
        const endpoint = accountId ? `/banking/accounts/${accountId}/transactions` : '/banking/transactions';
        return await this.request('GET', endpoint);
    }

    async createBankAccount(bankData) {
        return await this.request('POST', '/banking/accounts', bankData);
    }

    async closeBankAccount(accountId) {
        return await this.request('DELETE', `/banking/accounts/${accountId}`);
    }

    // Cryptocurrency
    async getCryptoWallet() {
        return await this.request('GET', '/crypto/wallet');
    }

    async buyCrypto(amount) {
        return await this.request('POST', '/crypto/buy', { amount });
    }

    async sellCrypto(amount) {
        return await this.request('POST', '/crypto/sell', { amount });
    }

    async transferCrypto(recipientAddress, amount) {
        return await this.request('POST', '/crypto/transfer', {
            recipient: recipientAddress,
            amount
        });
    }

    // Clan Management (Original PHP: clan.php)
    async getClan() {
        return await this.request('GET', '/clan');
    }

    async createClan(clanData) {
        return await this.request('POST', '/clan', {
            name: clanData.name,
            description: clanData.description || '',
            type: clanData.type || 'public'
        });
    }

    async joinClan(clanId) {
        return await this.request('POST', `/clan/${clanId}/join`);
    }

    async leaveClan() {
        return await this.request('POST', '/clan/leave');
    }

    async inviteTosClan(username) {
        return await this.request('POST', '/clan/invite', { username });
    }

    async kickFromClan(userId) {
        return await this.request('POST', '/clan/kick', { user_id: userId });
    }

    async promoteClanMember(userId, rank) {
        return await this.request('POST', '/clan/promote', {
            user_id: userId,
            rank
        });
    }

    async getClanWars() {
        return await this.request('GET', '/clan/wars');
    }

    async declareWar(targetClanId) {
        return await this.request('POST', '/clan/wars', {
            target_clan_id: targetClanId
        });
    }

    // Mission System (Original PHP: missions.php)
    async getMissions() {
        return await this.request('GET', '/missions');
    }

    async acceptMission(missionId) {
        return await this.request('POST', `/missions/${missionId}/accept`);
    }

    async completeMission(missionId) {
        return await this.request('POST', `/missions/${missionId}/complete`);
    }

    async abandonMission(missionId) {
        return await this.request('POST', `/missions/${missionId}/abandon`);
    }

    async getMissionDetails(missionId) {
        return await this.request('GET', `/missions/${missionId}`);
    }

    // Mail System (Original PHP: mail.php)
    async getMessages() {
        return await this.request('GET', '/messages');
    }

    async sendMessage(messageData) {
        return await this.request('POST', '/messages', {
            recipient: messageData.recipient,
            subject: messageData.subject,
            content: messageData.content
        });
    }

    async getMessage(messageId) {
        return await this.request('GET', `/messages/${messageId}`);
    }

    async markMessageAsRead(messageId) {
        return await this.request('POST', `/messages/${messageId}/read`);
    }

    async deleteMessage(messageId) {
        return await this.request('DELETE', `/messages/${messageId}`);
    }

    // Ranking System (Original PHP: ranking.php)
    async getRanking(type = 'overall') {
        return await this.request('GET', `/ranking/${type}`);
    }

    async getPlayerRank(userId = null) {
        const endpoint = userId ? `/ranking/player/${userId}` : '/ranking/me';
        return await this.request('GET', endpoint);
    }

    // Statistics (Original PHP: stats.php)
    async getStats() {
        return await this.request('GET', '/stats');
    }

    async getPlayerStats(userId = null) {
        const endpoint = userId ? `/stats/player/${userId}` : '/stats/me';
        return await this.request('GET', endpoint);
    }

    async getServerStats() {
        return await this.request('GET', '/stats/server');
    }

    // Logs System (Original PHP: logs functionality)
    async getLogs(serverId = null) {
        const endpoint = serverId ? `/logs/${serverId}` : '/logs';
        return await this.request('GET', endpoint);
    }

    async clearLogs(serverId = null) {
        const endpoint = serverId ? `/logs/${serverId}` : '/logs';
        return await this.request('DELETE', endpoint);
    }

    async getLogEntry(logId) {
        return await this.request('GET', `/logs/entry/${logId}`);
    }

    // News System
    async getNews() {
        return await this.request('GET', '/news');
    }

    async getNewsItem(newsId) {
        return await this.request('GET', `/news/${newsId}`);
    }

    // Chat System
    async getChatMessages(channel = 'global') {
        return await this.request('GET', `/chat/${channel}`);
    }

    async sendChatMessage(channel, message) {
        return await this.request('POST', `/chat/${channel}`, { message });
    }

    // Notifications
    async getNotifications() {
        return await this.request('GET', '/notifications');
    }

    async markNotificationAsRead(notificationId) {
        return await this.request('POST', `/notifications/${notificationId}/read`);
    }

    async clearAllNotifications() {
        return await this.request('DELETE', '/notifications');
    }

    // Server Management
    async getServers() {
        return await this.request('GET', '/servers');
    }

    async createServer(serverData) {
        return await this.request('POST', '/servers', serverData);
    }

    async getServerDetails(serverId) {
        return await this.request('GET', `/servers/${serverId}`);
    }

    async updateServer(serverId, serverData) {
        return await this.request('PUT', `/servers/${serverId}`, serverData);
    }

    async deleteServer(serverId) {
        return await this.request('DELETE', `/servers/${serverId}`);
    }

    // File System Operations
    async getFileSystem(serverId = null) {
        const endpoint = serverId ? `/filesystem/${serverId}` : '/filesystem';
        return await this.request('GET', endpoint);
    }

    async createFile(path, content) {
        return await this.request('POST', '/filesystem/files', {
            path,
            content
        });
    }

    async createDirectory(path) {
        return await this.request('POST', '/filesystem/directories', { path });
    }

    async deleteFileOrDirectory(path) {
        return await this.request('DELETE', '/filesystem', { path });
    }

    // Market Operations
    async getMarketListings(category = 'all') {
        return await this.request('GET', `/market/${category}`);
    }

    async createMarketListing(itemData) {
        return await this.request('POST', '/market', itemData);
    }

    async purchaseFromMarket(listingId) {
        return await this.request('POST', `/market/${listingId}/purchase`);
    }

    // Utility Methods
    generateRequestId() {
        return Math.random().toString(36).substr(2, 9);
    }

    delay(ms) {
        return new Promise(resolve => setTimeout(resolve, ms));
    }

    handleUnauthorized() {
        // Clear session and redirect to login
        this.sessionId = '';
        window.GAME_CONFIG.session_id = '';
        
        if (window.location.pathname !== '/login') {
            window.location.href = '/login';
        }
    }

    handleProcessComplete(data) {
        // Show notification
        window.HackerExperience?.showNotification('Process Complete', data.message, 'success');
        
        // Play sound
        window.HackerExperience?.playSound('process-complete');
        
        // Refresh processes view if visible
        if (document.querySelector('#processes-page:not(.hidden)')) {
            this.getProcesses().then(response => {
                if (response.success) {
                    window.HackerExperience?.updateProcesses(response.data);
                }
            });
        }
    }

    handleNotification(data) {
        window.HackerExperience?.showNotification(data.title, data.message, data.type);
    }

    handleStatsUpdate(data) {
        // Update player stats in UI
        if (data.money !== undefined) {
            const moneyElement = document.getElementById('player-money');
            if (moneyElement) {
                moneyElement.textContent = data.money.toLocaleString();
            }
        }
        
        if (data.crypto !== undefined) {
            const cryptoElement = document.getElementById('player-crypto');
            if (cryptoElement) {
                cryptoElement.textContent = data.crypto.toFixed(4);
            }
        }
        
        if (data.level !== undefined) {
            const levelElement = document.getElementById('player-level');
            if (levelElement) {
                levelElement.textContent = data.level;
            }
        }
    }

    // Batch Operations
    async batch(requests) {
        const promises = requests.map(req => 
            this.request(req.method, req.endpoint, req.data, req.options)
        );
        
        return await Promise.allSettled(promises);
    }

    // Cache Management
    clearCache() {
        // Clear any cached data
        if (window.caches) {
            window.caches.keys().then(names => {
                names.forEach(name => {
                    if (name.startsWith('he-api-')) {
                        window.caches.delete(name);
                    }
                });
            });
        }
    }

    // Connection Status
    isOnline() {
        return navigator.onLine && this.ws?.readyState === WebSocket.OPEN;
    }

    // Error Reporting
    reportError(error, context = '') {
        if (this.config.debug) {
            console.error(`[API] Error in ${context}:`, error);
        }
        
        // Send error to backend for logging (if not in debug mode)
        if (!this.config.debug && context !== 'error-reporting') {
            this.request('POST', '/errors', {
                message: error.message,
                stack: error.stack,
                context,
                timestamp: new Date().toISOString(),
                user_agent: navigator.userAgent,
                url: window.location.href
            }).catch(() => {
                // Silently fail error reporting to avoid infinite loops
            });
        }
    }

    // Cleanup
    destroy() {
        if (this.ws) {
            this.ws.close();
        }
        
        this.pendingRequests.forEach((request, id) => {
            if (request.abort) {
                request.abort();
            }
        });
        
        this.pendingRequests.clear();
        this.requestQueue = [];
    }
}

// Global API instance
if (typeof window !== 'undefined') {
    window.HackerExperienceAPI = HackerExperienceAPI;
    
    // Initialize global instance
    window.addEventListener('DOMContentLoaded', () => {
        if (window.GAME_CONFIG) {
            window.API = new HackerExperienceAPI(window.GAME_CONFIG);
        }
    });
}

// Export for module systems
if (typeof module !== 'undefined' && module.exports) {
    module.exports = HackerExperienceAPI;
}