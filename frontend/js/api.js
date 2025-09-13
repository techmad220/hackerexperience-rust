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

    async request(endpoint, options = {}) {
        const url = `${this.baseUrl}${endpoint}`;
        const config = {
            method: 'GET',
            headers: { ...this.defaultHeaders },
            ...options
        };

        // Add auth token to headers if available
        if (this.authToken && !config.headers['Authorization']) {
            config.headers['Authorization'] = `Bearer ${this.authToken}`;
        }

        try {
            console.log(`API Request: ${config.method} ${url}`);
            const response = await fetch(url, config);
            
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
                throw error;
            }

            return data;
        } catch (error) {
            console.error(`API Error: ${config.method} ${url}`, error);
            
            // Handle specific error cases
            if (error.status === 401) {
                this.clearAuthToken();
                // Optionally redirect to login
                // window.location.href = '/login.php';
            }
            
            throw error;
        }
    }

    async get(endpoint, params = {}) {
        const queryString = new URLSearchParams(params).toString();
        const url = queryString ? `${endpoint}?${queryString}` : endpoint;
        return this.request(url);
    }

    async post(endpoint, data = {}, options = {}) {
        return this.request(endpoint, {
            method: 'POST',
            body: JSON.stringify(data),
            ...options
        });
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
        if (response.token) {
            this.setAuthToken(response.token);
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
        }
    }

    async register(userData) {
        return this.post('/auth/register', userData);
    }

    // Player endpoints
    async getPlayerInfo() {
        return this.get('/player/info');
    }

    async updatePlayer(data) {
        return this.put('/player/info', data);
    }

    async getPlayerStats() {
        return this.get('/player/stats');
    }

    // Process endpoints
    async getProcesses() {
        return this.get('/processes');
    }

    async startProcess(processData) {
        return this.post('/processes', processData);
    }

    async pauseProcess(processId) {
        return this.put(`/processes/${processId}/pause`);
    }

    async cancelProcess(processId) {
        return this.delete(`/processes/${processId}`);
    }

    async getProcessStatus(processId) {
        return this.get(`/processes/${processId}`);
    }

    // Software endpoints
    async getSoftware() {
        return this.get('/software');
    }

    async installSoftware(softwareData) {
        return this.post('/software', softwareData);
    }

    async upgradeSoftware(softwareId) {
        return this.put(`/software/${softwareId}/upgrade`);
    }

    async removeSoftware(softwareId) {
        return this.delete(`/software/${softwareId}`);
    }

    async getSoftwareInfo(softwareType) {
        return this.get(`/software/info/${softwareType}`);
    }

    // Hardware endpoints
    async getHardware() {
        return this.get('/hardware');
    }

    async buyHardware(hardwareData) {
        return this.post('/hardware', hardwareData);
    }

    async upgradeHardware(hardwareId) {
        return this.put(`/hardware/${hardwareId}/upgrade`);
    }

    // Server/Network endpoints
    async scanNetwork(network = '192.168.1') {
        return this.post('/network/scan', { network });
    }

    async connectToServer(ip) {
        return this.post('/network/connect', { ip });
    }

    async disconnectFromServer() {
        return this.post('/network/disconnect');
    }

    async hackServer(ip, hackType = 'password') {
        return this.post('/network/hack', { ip, type: hackType });
    }

    async uploadFile(ip, filename, content) {
        return this.post('/network/upload', { ip, filename, content });
    }

    async downloadFile(ip, filename) {
        return this.post('/network/download', { ip, filename });
    }

    // Log endpoints
    async getLogs(serverId = null) {
        return this.get('/logs', serverId ? { server_id: serverId } : {});
    }

    async deleteLog(logId) {
        return this.delete(`/logs/${logId}`);
    }

    async hideLog(logId) {
        return this.put(`/logs/${logId}/hide`);
    }

    async editLog(logId, newMessage) {
        return this.put(`/logs/${logId}`, { message: newMessage });
    }

    async clearLogs(serverId = null) {
        return this.delete('/logs', serverId ? { server_id: serverId } : {});
    }

    // Mail endpoints
    async getMail(folder = 'inbox') {
        return this.get('/mail', { folder });
    }

    async sendMail(to, subject, message) {
        return this.post('/mail', { to, subject, message });
    }

    async deleteMail(mailId) {
        return this.delete(`/mail/${mailId}`);
    }

    async markMailRead(mailId) {
        return this.put(`/mail/${mailId}/read`);
    }

    async replyToMail(originalId, message) {
        return this.post('/mail/reply', { original_id: originalId, message });
    }

    // Clan endpoints
    async getClanInfo(clanId = null) {
        return this.get(clanId ? `/clan/${clanId}` : '/clan');
    }

    async createClan(name, description) {
        return this.post('/clan', { name, description });
    }

    async joinClan(clanId) {
        return this.post(`/clan/${clanId}/join`);
    }

    async leaveClan() {
        return this.post('/clan/leave');
    }

    async inviteToClan(username) {
        return this.post('/clan/invite', { username });
    }

    async getClanMembers(clanId = null) {
        return this.get(clanId ? `/clan/${clanId}/members` : '/clan/members');
    }

    async getClanWars(clanId = null) {
        return this.get(clanId ? `/clan/${clanId}/wars` : '/clan/wars');
    }

    // Mission endpoints
    async getMissions(difficulty = 'all', type = 'all') {
        return this.get('/missions', { difficulty, type });
    }

    async acceptMission(missionId) {
        return this.post(`/missions/${missionId}/accept`);
    }

    async completeMission(missionId) {
        return this.post(`/missions/${missionId}/complete`);
    }

    async abandonMission(missionId) {
        return this.post(`/missions/${missionId}/abandon`);
    }

    async getMissionProgress(missionId) {
        return this.get(`/missions/${missionId}/progress`);
    }

    // Financial endpoints
    async getBankAccounts() {
        return this.get('/bank/accounts');
    }

    async createBankAccount(bank, accountType = 'checking') {
        return this.post('/bank/accounts', { bank, type: accountType });
    }

    async getAccountBalance(accountNumber) {
        return this.get(`/bank/accounts/${accountNumber}/balance`);
    }

    async bankTransfer(amount, fromAccount, toAccount) {
        return this.post('/bank/transfer', { amount, from: fromAccount, to: toAccount });
    }

    async getTransactionHistory(accountNumber, limit = 50) {
        return this.get(`/bank/accounts/${accountNumber}/transactions`, { limit });
    }

    // Premium endpoints
    async getPremiumStatus() {
        return this.get('/premium/status');
    }

    async purchasePremium(packageId) {
        return this.post('/premium/purchase', { package_id: packageId });
    }

    // File upload endpoint
    async uploadImage(file) {
        const formData = new FormData();
        formData.append('image', file);
        return this.postForm('/upload-image', formData);
    }

    // AJAX endpoint (for legacy compatibility)
    async ajax(func, params = {}) {
        const formData = new FormData();
        formData.append('func', func);
        
        Object.entries(params).forEach(([key, value]) => {
            formData.append(key, value);
        });

        return this.postForm('/ajax', formData);
    }

    // Initialize method
    async initialize() {
        try {
            // Test connection
            await this.get('/health');
            console.log('API connection established');
            return true;
        } catch (error) {
            console.warn('API connection failed:', error);
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