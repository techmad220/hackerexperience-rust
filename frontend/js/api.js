// API Client for Hacker Experience

class APIClient {
    constructor() {
        this.baseUrl = this.getBaseUrl();
        this.authToken = this.getAuthToken();
        this.defaultHeaders = {
            'Content-Type': 'application/json',
            'Accept': 'application/json'
        };
        
        if (this.authToken) {
            this.defaultHeaders['Authorization'] = `Bearer ${this.authToken}`;
        }
        
        // Retry configuration
        this.maxRetries = 3;
        this.retryDelay = 1000;
        this.retryableStatusCodes = [500, 502, 503, 504, 408, 429];
        
        // Request queue for offline scenarios
        this.requestQueue = [];
        this.isOnline = navigator.onLine;
        
        // Listen for online/offline events
        window.addEventListener('online', () => {
            this.isOnline = true;
            this.processRequestQueue();
        });
        
        window.addEventListener('offline', () => {
            this.isOnline = false;
        });
    }

    getBaseUrl() {
        const protocol = window.location.protocol;
        const host = window.location.host;
        return `${protocol}//${host}/api`;
    }

    getAuthToken() {
        return localStorage.getItem('auth_token') || sessionStorage.getItem('auth_token');
    }

    setAuthToken(token) {
        this.authToken = token;
        this.defaultHeaders['Authorization'] = `Bearer ${token}`;
        localStorage.setItem('auth_token', token);
    }

    clearAuthToken() {
        this.authToken = null;
        delete this.defaultHeaders['Authorization'];
        localStorage.removeItem('auth_token');
        sessionStorage.removeItem('auth_token');
    }

    async request(endpoint, options = {}, retryCount = 0) {
        const url = `${this.baseUrl}${endpoint}`;
        const config = {
            method: 'GET',
            headers: { ...this.defaultHeaders },
            timeout: 30000, // 30 second timeout
            ...options
        };

        // Add auth token to headers if available
        if (this.authToken && !config.headers['Authorization']) {
            config.headers['Authorization'] = `Bearer ${this.authToken}`;
        }

        // Check if offline and queue request
        if (!this.isOnline && config.method !== 'GET') {
            return this.queueRequest(endpoint, options);
        }

        try {
            console.log(`API Request: ${config.method} ${url} (attempt ${retryCount + 1})`);
            
            const controller = new AbortController();
            const timeoutId = setTimeout(() => controller.abort(), config.timeout);
            
            const response = await fetch(url, {
                ...config,
                signal: controller.signal
            });
            
            clearTimeout(timeoutId);
            
            // Handle different response types
            const contentType = response.headers.get('content-type');
            let data;
            
            if (contentType && contentType.includes('application/json')) {
                data = await response.json();
            } else {
                data = await response.text();
            }

            if (!response.ok) {
                const error = new Error(`HTTP ${response.status}: ${response.statusText}`);
                error.status = response.status;
                error.data = data;
                
                // Check if we should retry
                if (this.shouldRetry(error.status, retryCount)) {
                    return this.retryRequest(endpoint, options, retryCount);
                }
                
                throw error;
            }

            // Handle API response format
            if (data && typeof data === 'object' && data.hasOwnProperty('success')) {
                if (!data.success) {
                    const error = new Error(data.error || 'API request failed');
                    error.apiError = true;
                    error.data = data;
                    throw error;
                }
                return data; // Return the full response object
            }

            return data;
        } catch (error) {
            console.error(`API Error: ${config.method} ${url}`, error);
            
            // Handle specific error cases
            if (error.status === 401) {
                this.handleAuthError();
            } else if (error.name === 'AbortError') {
                error.message = 'Request timeout';
            } else if (!this.isOnline) {
                error.message = 'No internet connection';
            }
            
            // Check if we should retry
            if (this.shouldRetry(error.status, retryCount) || (error.name === 'TypeError' && retryCount < this.maxRetries)) {
                return this.retryRequest(endpoint, options, retryCount);
            }
            
            throw error;
        }
    }

    shouldRetry(status, retryCount) {
        return retryCount < this.maxRetries && 
               (this.retryableStatusCodes.includes(status) || !status);
    }

    async retryRequest(endpoint, options, retryCount) {
        const delay = this.retryDelay * Math.pow(2, retryCount); // Exponential backoff
        console.log(`Retrying request in ${delay}ms...`);
        
        await this.sleep(delay);
        return this.request(endpoint, options, retryCount + 1);
    }

    sleep(ms) {
        return new Promise(resolve => setTimeout(resolve, ms));
    }

    handleAuthError() {
        this.clearAuthToken();
        
        // Show notification
        if (window.WebSocketManager) {
            window.WebSocketManager.showNotification(
                'Session Expired', 
                'Please log in again', 
                'warning'
            );
        }
        
        // Redirect to login after a short delay
        setTimeout(() => {
            if (window.gameInstance) {
                window.gameInstance.showLoginForm();
            }
        }, 1000);
    }

    queueRequest(endpoint, options) {
        return new Promise((resolve, reject) => {
            this.requestQueue.push({
                endpoint,
                options,
                resolve,
                reject,
                timestamp: Date.now()
            });
        });
    }

    async processRequestQueue() {
        console.log(`Processing ${this.requestQueue.length} queued requests...`);
        
        const queue = [...this.requestQueue];
        this.requestQueue = [];
        
        for (const item of queue) {
            try {
                const result = await this.request(item.endpoint, item.options);
                item.resolve(result);
            } catch (error) {
                item.reject(error);
            }
        }
    }

    async get(endpoint, params = {}) {
        const queryString = new URLSearchParams(params).toString();
        const url = queryString ? `${endpoint}?${queryString}` : endpoint;
        return this.request(url);
    }

    // Safe version of get method
    async safeGet(endpoint, params = {}) {
        return this.safeCall(this.get, endpoint, params);
    }

    async post(endpoint, data = {}, options = {}) {
        return this.request(endpoint, {
            method: 'POST',
            body: JSON.stringify(data),
            ...options
        });
    }

    // Safe version of post method
    async safePost(endpoint, data = {}, options = {}) {
        return this.safeCall(this.post, endpoint, data, options);
    }

    async put(endpoint, data = {}) {
        return this.request(endpoint, {
            method: 'PUT',
            body: JSON.stringify(data)
        });
    }

    async delete(endpoint) {
        return this.request(endpoint, {
            method: 'DELETE'
        });
    }

    async postForm(endpoint, formData) {
        // Remove Content-Type header to let browser set it with boundary for FormData
        const headers = { ...this.defaultHeaders };
        delete headers['Content-Type'];

        return this.request(endpoint, {
            method: 'POST',
            headers,
            body: formData
        });
    }

    // Authentication endpoints
    async login(username, password) {
        const response = await this.post('/auth/login', { username, password });
        if (response.success && response.data && response.data.token) {
            this.setAuthToken(response.data.token);
            // Store user data
            if (response.data.user) {
                localStorage.setItem('user_data', JSON.stringify(response.data.user));
                localStorage.setItem('player_id', response.data.user.id.toString());
            }
        }
        return response;
    }

    async logout() {
        try {
            await this.post('/auth/logout');
        } catch (error) {
            console.warn('Logout API call failed:', error);
        } finally {
            this.clearAuthToken();
            localStorage.removeItem('user_data');
            localStorage.removeItem('player_id');
        }
    }

    async register(username, email, password) {
        return this.post('/auth/register', { username, email, password });
    }

    async getCurrentUser() {
        return this.get('/auth/me');
    }

    // User/Player endpoints
    async getPlayerInfo() {
        return this.get('/user/profile');
    }

    async updatePlayer(data) {
        return this.put('/user/update', data);
    }

    async getPlayerStats() {
        return this.get('/user/stats');
    }

    // Process endpoints
    async getProcesses() {
        return this.get('/processes/active');
    }

    async startProcess(action, targetIp, softwareId = null) {
        return this.post('/processes/start', { action, target_ip: targetIp, software_id: softwareId });
    }

    async killProcess(processId) {
        return this.post(`/processes/${processId}/kill`);
    }

    async killAllProcesses() {
        return this.post('/processes/kill-all');
    }

    async getProcessDetails(processId) {
        return this.get(`/processes/${processId}`);
    }

    // Software endpoints
    async getSoftware() {
        return this.get('/software/installed');
    }

    async startSoftware(softwareId) {
        return this.post(`/software/${softwareId}/start`);
    }

    async stopSoftware(softwareId) {
        return this.post(`/software/${softwareId}/stop`);
    }

    async uninstallSoftware(softwareId) {
        return this.post(`/software/${softwareId}/uninstall`);
    }

    // Store endpoints
    async getSoftwareStore() {
        return this.get('/store/software');
    }

    async purchaseSoftware(softwareId) {
        return this.post('/store/purchase-software', { software_id: softwareId });
    }

    // Hardware endpoints
    async getHardware() {
        return this.get('/hardware/owned');
    }

    async upgradeHardware(hardwareId, newSpecValue) {
        return this.post('/hardware/upgrade', { hardware_id: hardwareId, new_spec_value: newSpecValue });
    }

    async getHardwareStore() {
        return this.get('/store/hardware');
    }

    async purchaseHardware(hardwareType, specValue, price) {
        return this.post('/store/purchase-hardware', { hardware_type: hardwareType, spec_value: specValue, price: price });
    }

    // Server/Network endpoints
    async scanNetwork() {
        return this.post('/network/scan');
    }

    async traceRoute(targetIp) {
        return this.post('/network/trace', { target_ip: targetIp });
    }

    async connectToServer(ip) {
        return this.post('/servers/connect', { ip });
    }

    async getAvailableServers() {
        return this.get('/servers/available');
    }

    async getOwnedServers() {
        return this.get('/servers/owned');
    }

    async getServerDetails(serverId) {
        return this.get(`/servers/${serverId}`);
    }

    async getServerFiles(serverId) {
        return this.get(`/servers/${serverId}/files`);
    }

    async getServerLogs(serverId) {
        return this.get(`/servers/${serverId}/logs`);
    }

    // Log endpoints
    async getLogs() {
        return this.get('/logs/recent');
    }

    async clearLogs() {
        return this.post('/logs/clear');
    }

    // File endpoints
    async getFiles() {
        return this.get('/files/list');
    }

    async createFile(name, fileType, size, path, isHidden = false) {
        return this.post('/files/create', { name, file_type: fileType, size, path, is_hidden: isHidden });
    }

    async deleteFile(fileId) {
        return this.delete(`/files/${fileId}/delete`);
    }

    async downloadFile(fileId) {
        return this.get(`/files/${fileId}/download`);
    }

    // Clan endpoints
    async getClanInfo() {
        return this.get('/clan/info');
    }

    async createClan(name, tag, description) {
        return this.post('/clan/create', { name, tag, description });
    }

    async joinClan(clanId) {
        return this.post('/clan/join', { clan_id: clanId });
    }

    async leaveClan() {
        return this.post('/clan/leave');
    }

    async getClanMembers() {
        return this.get('/clan/members');
    }

    // Mission endpoints
    async getMissions() {
        return this.get('/missions/active');
    }

    async completeMission(missionId) {
        return this.post(`/missions/${missionId}/complete`);
    }

    async abandonMission(missionId) {
        return this.post(`/missions/${missionId}/abandon`);
    }

    // Ranking endpoints
    async getTopRankings() {
        return this.get('/rankings/top');
    }

    async getClanRankings() {
        return this.get('/rankings/clans');
    }

    // Chat endpoints
    async getChatHistory(limit = 50, offset = 0) {
        return this.get('/chat/history', { limit, offset });
    }

    async sendChatMessage(message) {
        return this.post('/chat/send', { message });
    }

    // File upload endpoint
    async uploadImage(file) {
        const formData = new FormData();
        formData.append('image', file);
        return this.postForm('/upload-image', formData);
    }

    // Game mechanics endpoints
    async getGameStats() {
        return this.get('/game/stats');
    }

    async gameTick() {
        return this.post('/game/tick');
    }

    // Utility methods
    getUserData() {
        const userData = localStorage.getItem('user_data');
        return userData ? JSON.parse(userData) : null;
    }

    isAuthenticated() {
        return !!this.authToken && !!this.getUserData();
    }

    // Enhanced error handling helpers
    handleError(error, context = '') {
        console.error(`API Error ${context}:`, error);
        
        let userMessage = 'An error occurred';
        
        if (error.apiError && error.data && error.data.error) {
            userMessage = error.data.error;
        } else if (error.message === 'Request timeout') {
            userMessage = 'Request timed out. Please try again.';
        } else if (error.message === 'No internet connection') {
            userMessage = 'No internet connection. Request will be retried when online.';
        } else if (error.status >= 500) {
            userMessage = 'Server error. Please try again later.';
        } else if (error.status === 404) {
            userMessage = 'Resource not found.';
        } else if (error.status === 403) {
            userMessage = 'Access denied.';
        }
        
        // Show notification if available
        if (window.WebSocketManager) {
            window.WebSocketManager.showNotification(
                'Error', 
                userMessage, 
                'error'
            );
        }
        
        return userMessage;
    }

    // Safe API call wrapper
    async safeCall(apiMethod, ...args) {
        try {
            const result = await apiMethod.apply(this, args);
            return { success: true, data: result };
        } catch (error) {
            const message = this.handleError(error, apiMethod.name);
            return { success: false, error, message };
        }
    }

    // Initialize method
    async initialize() {
        try {
            // If we have a token, verify it's still valid
            if (this.authToken) {
                const result = await this.safeCall(this.getCurrentUser);
                if (result.success && result.data && result.data.success) {
                    console.log('API connection established with authenticated user');
                    return true;
                } else {
                    // Token expired or invalid
                    this.clearAuthToken();
                }
            }
            
            console.log('API connection established (not authenticated)');
            return true;
        } catch (error) {
            console.warn('API initialization failed:', error);
            return false;
        }
    }
}

// Create global API instance
window.API = new APIClient();

// Export for module systems
if (typeof module !== 'undefined' && module.exports) {
    module.exports = APIClient;
}