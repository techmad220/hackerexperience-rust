// Hacker Experience - Main Game Logic

class Game {
    constructor() {
        this.currentPage = 'desktop';
        this.processes = [];
        this.software = [];
        this.player = {
            username: 'Player',
            level: 1,
            money: 1000,
            experience: 0
        };
        this.isInitialized = false;
        
        // Initialize time
        this.startTime = Date.now();
        this.updateTimeInterval = null;
        
        // Initialize bindings
        this.bindEvents();
    }

    static init() {
        window.gameInstance = new Game();
        return window.gameInstance.initialize();
    }

    async initialize() {
        console.log('Initializing Hacker Experience...');
        
        try {
            // Show loading screen
            this.showLoadingScreen();
            
            // Initialize API connection
            await API.initialize();
            
            // Check if user is authenticated
            if (!API.isAuthenticated()) {
                console.log('User not authenticated, redirecting to login');
                window.location.href = 'login.html';
                return;
            }
            
            // Load player data
            const playerLoaded = await this.loadPlayerData();
            if (!playerLoaded) {
                console.log('Failed to load player data, redirecting to login');
                window.location.href = 'login.html';
                return;
            }
            
            // Initialize WebSocket connection
            if (typeof WebSocketManager !== 'undefined') {
                // Set up WebSocket event handlers
                this.setupWebSocketHandlers();
                WebSocketManager.connect();
            }
            
            // Load initial data
            await this.loadInitialData();
            
            // Hide loading screen and show game
            this.hideLoadingScreen();
            
            // Start game loops
            this.startGameLoops();
            
            this.isInitialized = true;
            console.log('Game initialized successfully');
            
        } catch (error) {
            console.error('Failed to initialize game:', error);
            this.showError('Failed to initialize game: ' + error.message);
        }
    }

    showLoginForm() {
        window.location.href = 'login.html';
    }

    showLoadingScreen() {
        const loadingScreen = document.getElementById('loading-screen');
        const gameInterface = document.getElementById('game-interface');
        
        if (loadingScreen && gameInterface) {
            loadingScreen.style.display = 'flex';
            gameInterface.style.display = 'none';
        }
    }

    hideLoadingScreen() {
        const loadingScreen = document.getElementById('loading-screen');
        const gameInterface = document.getElementById('game-interface');
        
        if (loadingScreen && gameInterface) {
            setTimeout(() => {
                loadingScreen.style.display = 'none';
                gameInterface.style.display = 'block';
                gameInterface.classList.add('fade-in');
            }, 500);
        }
    }

    async loadPlayerData() {
        try {
            const response = await API.getPlayerInfo();
            if (response && response.success && response.data) {
                this.player = { ...this.player, ...response.data };
                this.updatePlayerUI();
                return true;
            } else {
                console.warn('Failed to load player data:', response);
                return false;
            }
        } catch (error) {
            console.warn('Could not load player data:', error);
            if (error.status === 401) {
                API.clearAuthToken();
            }
            return false;
        }
    }

    async loadInitialData() {
        try {
            // Load processes
            const processResult = await API.safeGet('/processes/active');
            if (processResult.success && processResult.data.success) {
                this.processes = processResult.data.data || [];
                this.updateProcessList();
            }

            // Load software
            const softwareResult = await API.safeGet('/software/installed');
            if (softwareResult.success && softwareResult.data.success) {
                this.software = softwareResult.data.data || [];
                this.updateSoftwareGrid();
            }

            // Load hardware info
            const hardwareResult = await API.safeGet('/hardware/owned');
            if (hardwareResult.success && hardwareResult.data.success) {
                this.updateHardwareInfo(hardwareResult.data.data);
            }

        } catch (error) {
            console.warn('Could not load initial data:', error);
        }
    }

    updatePlayerUI() {
        const usernameEl = document.getElementById('username');
        const levelEl = document.getElementById('player-level');
        const moneyEl = document.getElementById('player-money');

        if (usernameEl) usernameEl.textContent = this.player.username || 'Player';
        if (levelEl) levelEl.textContent = this.calculateLevel(this.player.experience || 0);
        if (moneyEl) moneyEl.textContent = (this.player.money || 0).toLocaleString();
    }

    calculateLevel(experience) {
        // Simple level calculation based on experience
        return Math.floor(experience / 1000) + 1;
    }

    setupWebSocketHandlers() {
        // Override WebSocket handlers to update game state
        const originalHandleProcessStart = WebSocketManager.handleProcessStart;
        const originalHandleProcessComplete = WebSocketManager.handleProcessComplete;
        const originalHandleProcessKilled = WebSocketManager.handleProcessKilled;
        const originalHandleAllProcessesKilled = WebSocketManager.handleAllProcessesKilled;
        const originalHandlePlayerUpdate = WebSocketManager.handlePlayerUpdate;

        WebSocketManager.handleProcessStart = (data) => {
            originalHandleProcessStart.call(WebSocketManager, data);
            // Refresh processes if we're on the processes page
            if (this.currentPage === 'processes') {
                this.loadPageData('processes');
            }
        };

        WebSocketManager.handleProcessComplete = (data) => {
            originalHandleProcessComplete.call(WebSocketManager, data);
            // Refresh processes and potentially player data
            if (this.currentPage === 'processes') {
                this.loadPageData('processes');
            }
            // Reload player data to update money/experience
            this.loadPlayerData();
        };

        WebSocketManager.handleProcessKilled = (data) => {
            originalHandleProcessKilled.call(WebSocketManager, data);
            // Refresh processes
            if (this.currentPage === 'processes') {
                this.loadPageData('processes');
            }
        };

        WebSocketManager.handleAllProcessesKilled = (data) => {
            originalHandleAllProcessesKilled.call(WebSocketManager, data);
            // Clear processes and refresh
            this.processes = [];
            if (this.currentPage === 'processes') {
                this.updateProcessList();
            }
        };

        WebSocketManager.handlePlayerUpdate = (data) => {
            originalHandlePlayerUpdate.call(WebSocketManager, data);
            // Update local player data
            this.player = { ...this.player, ...data };
            this.updatePlayerUI();
        };
    }

    // Method to refresh current page data
    refreshCurrentPage() {
        if (this.currentPage && this.isInitialized) {
            this.loadPageData(this.currentPage);
        }
    }

    // Method to handle real-time data updates
    handleRealtimeUpdate(type, data) {
        switch (type) {
            case 'process_update':
                // Update specific process
                const processIndex = this.processes.findIndex(p => p.id === data.id);
                if (processIndex !== -1) {
                    this.processes[processIndex] = { ...this.processes[processIndex], ...data };
                    if (this.currentPage === 'processes') {
                        this.updateProcessList();
                    }
                }
                break;
            case 'player_update':
                this.player = { ...this.player, ...data };
                this.updatePlayerUI();
                break;
            case 'money_update':
                if (this.player) {
                    this.player.money = data.new_amount;
                    this.updatePlayerUI();
                }
                break;
            case 'software_update':
                // Refresh software if we're on the software page
                if (this.currentPage === 'software') {
                    this.loadPageData('software');
                }
                break;
            case 'hardware_update':
                // Refresh hardware if we're on the hardware page
                if (this.currentPage === 'hardware') {
                    this.loadPageData('hardware');
                }
                break;
        }
    }

    updateProcessList() {
        const processList = document.getElementById('process-list');
        if (!processList) return;

        processList.innerHTML = '';

        if (this.processes.length === 0) {
            processList.innerHTML = '<p class="no-processes">No running processes</p>';
            return;
        }

        this.processes.forEach(process => {
            const processItem = document.createElement('div');
            processItem.className = 'process-item';
            const progress = Math.round((process.progress || 0) * 100);
            const timeLeft = process.time_left || 0;
            const timeLeftStr = timeLeft > 0 ? `${timeLeft}s remaining` : 'Completed';
            
            processItem.innerHTML = `
                <div class="process-info">
                    <h4>${process.action || process.type} Process</h4>
                    <p>Target: ${process.target_ip || process.target || 'N/A'}</p>
                    <p>Status: ${process.status} | ${timeLeftStr}</p>
                </div>
                <div class="process-progress">
                    <div class="progress-bar">
                        <div class="progress-fill" style="width: ${progress}%"></div>
                    </div>
                    <span class="progress-text">${progress}%</span>
                </div>
                <div class="process-actions">
                    <button onclick="window.gameInstance.killProcess(${process.id})" class="btn btn-danger btn-small">Kill</button>
                </div>
            `;
            processList.appendChild(processItem);
        });

        // Update status bar
        const processCount = document.getElementById('active-process-count');
        if (processCount) {
            processCount.textContent = this.processes.length;
        }
    }

    // Process management functions
    async killProcess(processId) {
        try {
            const result = await API.safePost(`/processes/${processId}/kill`);
            if (result.success && result.data.success) {
                this.showSuccess('Process killed');
                // Remove from local array
                this.processes = this.processes.filter(p => p.id !== processId);
                this.updateProcessList();
            } else {
                this.showError(result.message || 'Failed to kill process');
            }
        } catch (error) {
            this.showError('Failed to kill process: ' + error.message);
        }
    }

    async killAllProcesses() {
        if (confirm('Kill all running processes?')) {
            try {
                const result = await API.safePost('/processes/kill-all');
                if (result.success && result.data.success) {
                    this.showSuccess('All processes killed');
                    // Clear local array
                    this.processes = [];
                    this.updateProcessList();
                } else {
                    this.showError(result.message || 'Failed to kill processes');
                }
            } catch (error) {
                this.showError('Failed to kill processes: ' + error.message);
            }
        }
    }

    updateSoftwareGrid() {
        const softwareGrid = document.getElementById('software-grid');
        if (!softwareGrid) return;

        softwareGrid.innerHTML = '';

        if (this.software.length === 0) {
            softwareGrid.innerHTML = '<p class="no-software">No software installed</p>';
            return;
        }

        this.software.forEach(software => {
            const softwareItem = document.createElement('div');
            softwareItem.className = 'software-item';
            softwareItem.innerHTML = `
                <div class="icon">${this.getSoftwareIcon(software.type)}</div>
                <div class="name">${software.name || software.type}</div>
                <div class="version">v${software.version || '1.0'}</div>
            `;
            softwareItem.addEventListener('click', () => this.showSoftwareDetails(software));
            softwareGrid.appendChild(softwareItem);
        });
    }

    updateHardwareInfo(hardware) {
        const hardwareInfo = document.getElementById('hardware-info');
        if (!hardwareInfo || !hardware) return;

        hardwareInfo.innerHTML = `
            <div class="hardware-grid">
                <div class="hardware-item">
                    <h4>CPU</h4>
                    <p>${hardware.cpu?.type || 'Basic Processor'}</p>
                    <p>Speed: ${hardware.cpu?.speed || 1000} MHz</p>
                </div>
                <div class="hardware-item">
                    <h4>Memory</h4>
                    <p>${hardware.memory?.type || 'Basic RAM'}</p>
                    <p>Size: ${hardware.memory?.size || 1024} MB</p>
                </div>
                <div class="hardware-item">
                    <h4>Storage</h4>
                    <p>${hardware.storage?.type || 'Hard Drive'}</p>
                    <p>Size: ${(hardware.storage?.size || 10000) / 1000} GB</p>
                </div>
                <div class="hardware-item">
                    <h4>Network</h4>
                    <p>${hardware.network?.type || 'Ethernet'}</p>
                    <p>Speed: ${hardware.network?.speed || 100} Mbps</p>
                </div>
            </div>
        `;
    }

    getSoftwareIcon(type) {
        const icons = {
            'cracker': '🔓',
            'hasher': '🔐',
            'encryptor': '🛡️',
            'firewall': '🚧',
            'antivirus': '🔒',
            'scanner': '🔍',
            'backdoor': '🚪',
            'virus': '🦠'
        };
        return icons[type] || '💾';
    }

    bindEvents() {
        // Navigation
        document.addEventListener('click', (e) => {
            if (e.target.matches('.nav-link')) {
                e.preventDefault();
                const page = e.target.dataset.page;
                if (page) this.navigateTo(page);
            }

            // Desktop icons
            if (e.target.closest('.desktop-icon')) {
                const icon = e.target.closest('.desktop-icon');
                const action = icon.dataset.action;
                if (action) this.handleDesktopIconClick(action);
            }

            // Modal close
            if (e.target.matches('.modal-close') || e.target.matches('.modal-overlay')) {
                this.closeModal();
            }

            // Connect button
            if (e.target.matches('#connect-btn')) {
                this.handleConnect();
            }

            // Logout button
            if (e.target.matches('#logout-btn')) {
                this.handleLogout();
            }
        });

        // Terminal input
        const terminalInput = document.getElementById('terminal-input');
        if (terminalInput) {
            terminalInput.addEventListener('keypress', (e) => {
                if (e.key === 'Enter') {
                    this.executeCommand(terminalInput.value);
                    terminalInput.value = '';
                }
            });
        }

        // Chat input
        document.addEventListener('keypress', (e) => {
            if (e.target.id === 'chat-input' && e.key === 'Enter') {
                this.sendChatMessage();
            }
        });

        // Address bar
        const addressInput = document.getElementById('address-input');
        if (addressInput) {
            addressInput.addEventListener('keypress', (e) => {
                if (e.key === 'Enter') {
                    this.handleConnect();
                }
            });
        }
    }

    navigateTo(page) {
        // Update active nav link
        document.querySelectorAll('.nav-link').forEach(link => {
            link.classList.remove('active');
        });
        document.querySelector(`[data-page="${page}"]`)?.classList.add('active');

        // Show/hide pages
        document.querySelectorAll('.page').forEach(pageEl => {
            pageEl.style.display = 'none';
        });
        const pageEl = document.getElementById(`${page}-page`);
        if (pageEl) {
            pageEl.style.display = 'block';
            pageEl.classList.add('fade-in');
        }

        this.currentPage = page;

        // Load page-specific data
        this.loadPageData(page);
    }

    async loadPageData(page) {
        try {
            switch (page) {
                case 'processes':
                    const processResult = await API.safeGet('/processes/active');
                    if (processResult.success && processResult.data.success) {
                        this.processes = processResult.data.data || [];
                        this.updateProcessList();
                    }
                    break;
                case 'software':
                    const softwareResult = await API.safeGet('/software/installed');
                    if (softwareResult.success && softwareResult.data.success) {
                        this.software = softwareResult.data.data || [];
                        this.updateSoftwareGrid();
                    }
                    break;
                case 'hardware':
                    const hardwareResult = await API.safeGet('/hardware/owned');
                    if (hardwareResult.success && hardwareResult.data.success) {
                        this.updateHardwareInfo(hardwareResult.data.data);
                    }
                    break;
                case 'servers':
                    await this.loadServers();
                    break;
                case 'internet':
                    await this.loadAvailableServers();
                    break;
                case 'clan':
                    await this.loadClan();
                    break;
                case 'missions':
                    await this.loadMissions();
                    break;
                case 'ranking':
                    await this.loadRankings();
                    break;
                case 'mail':
                    await this.loadChat();
                    break;
                case 'logs':
                    await this.loadLogs();
                    break;
                case 'connections':
                    await this.loadConnections();
                    break;
            }
        } catch (error) {
            console.warn(`Failed to load data for page ${page}:`, error);
        }
    }

    handleDesktopIconClick(action) {
        switch (action) {
            case 'software':
                this.navigateTo('software');
                break;
            case 'processes':
                this.navigateTo('processes');
                break;
            case 'internet':
                this.navigateTo('internet');
                break;
            default:
                console.log('Unknown desktop action:', action);
        }
    }

    executeCommand(command) {
        const terminal = document.getElementById('terminal');
        if (!terminal) return;

        // Add command to terminal
        const commandLine = document.createElement('div');
        commandLine.className = 'terminal-line';
        commandLine.innerHTML = `<span class="prompt">root@localhost:~#</span> <span class="command">${command}</span>`;
        terminal.appendChild(commandLine);

        // Process command
        let output = '';
        const cmd = command.toLowerCase().trim();

        switch (cmd) {
            case 'help':
                output = `Available commands:
  help          - Show this help message
  ps            - List running processes
  software      - List installed software
  scan <ip>     - Scan a network address
  connect <ip>  - Connect to a server
  clear         - Clear terminal`;
                break;
            case 'ps':
                output = this.processes.length > 0 
                    ? this.processes.map(p => `${p.id}: ${p.type} - ${p.status}`).join('\n')
                    : 'No running processes';
                break;
            case 'software':
                output = this.software.length > 0 
                    ? this.software.map(s => `${s.name || s.type} v${s.version || '1.0'}`).join('\n')
                    : 'No software installed';
                break;
            case 'clear':
                terminal.innerHTML = '';
                return;
            default:
                if (cmd.startsWith('scan ')) {
                    const ip = cmd.substring(5);
                    output = `Scanning ${ip}...\nScan complete. Use 'connect ${ip}' to connect.`;
                } else if (cmd.startsWith('connect ')) {
                    const ip = cmd.substring(8);
                    this.navigateTo('internet');
                    const addressInput = document.getElementById('address-input');
                    if (addressInput) addressInput.value = ip;
                    output = `Connecting to ${ip}...`;
                } else {
                    output = `Command not found: ${command}. Type 'help' for available commands.`;
                }
        }

        // Add output to terminal
        const outputLine = document.createElement('div');
        outputLine.className = 'terminal-line';
        outputLine.innerHTML = output.replace(/\n/g, '<br>');
        terminal.appendChild(outputLine);

        // Scroll to bottom
        terminal.scrollTop = terminal.scrollHeight;
    }

    async handleConnect() {
        const addressInput = document.getElementById('address-input');
        const browserContent = document.getElementById('browser-content');
        
        if (!addressInput || !browserContent) return;

        const ip = addressInput.value.trim();
        if (!ip) {
            browserContent.innerHTML = '<div class="error">Please enter an IP address</div>';
            return;
        }

        browserContent.innerHTML = '<div class="loading">Connecting...</div>';

        try {
            const result = await API.connectToServer(ip);
            if (result && result.success) {
                browserContent.innerHTML = `
                    <div class="server-info">
                        <h3>Connected to ${ip}</h3>
                        <p>Server Type: ${result.server_type || 'Unknown'}</p>
                        <p>Status: ${result.protected ? 'Protected' : 'Open'}</p>
                        ${result.files ? `
                            <h4>Files:</h4>
                            <ul>
                                ${result.files.map(file => `<li>${file.name} (${file.size} bytes)</li>`).join('')}
                            </ul>
                        ` : ''}
                    </div>
                `;
            } else {
                browserContent.innerHTML = `<div class="error">Connection failed: ${result?.message || 'Unknown error'}</div>`;
            }
        } catch (error) {
            browserContent.innerHTML = `<div class="error">Connection failed: ${error.message}</div>`;
        }
    }

    showModal(title, content, buttons = []) {
        const modalOverlay = document.getElementById('modal-overlay');
        const modalTitle = document.getElementById('modal-title');
        const modalBody = document.getElementById('modal-body');
        const modalFooter = document.getElementById('modal-footer');

        if (!modalOverlay || !modalTitle || !modalBody || !modalFooter) return;

        modalTitle.textContent = title;
        modalBody.innerHTML = content;
        modalFooter.innerHTML = buttons.map(btn => 
            `<button class="btn ${btn.class || ''}" onclick="${btn.onclick || ''}">${btn.text}</button>`
        ).join('');

        modalOverlay.style.display = 'flex';
        modalOverlay.classList.add('fade-in');
    }

    closeModal() {
        const modalOverlay = document.getElementById('modal-overlay');
        if (modalOverlay) {
            modalOverlay.style.display = 'none';
            modalOverlay.classList.remove('fade-in');
        }
    }

    showSoftwareDetails(software) {
        this.showModal(
            `${software.name || software.type} v${software.version || '1.0'}`,
            `
                <p><strong>Type:</strong> ${software.type}</p>
                <p><strong>Size:</strong> ${software.size || 'Unknown'} MB</p>
                <p><strong>Description:</strong> ${software.description || 'No description available'}</p>
            `,
            [
                { text: 'Close', class: 'btn-secondary', onclick: 'window.gameInstance.closeModal()' }
            ]
        );
    }

    startGameLoops() {
        // Update time every second
        this.updateTimeInterval = setInterval(() => {
            this.updateTime();
        }, 1000);

        // Update processes every 5 seconds
        setInterval(() => {
            if (this.isInitialized) {
                this.updateProcesses();
            }
        }, 5000);

        // Update player data every 30 seconds
        setInterval(() => {
            if (this.isInitialized) {
                this.loadPlayerData();
            }
        }, 30000);
    }

    updateTime() {
        const timeEl = document.getElementById('current-time');
        if (timeEl) {
            const now = new Date();
            timeEl.textContent = now.toTimeString().substring(0, 8);
        }
    }

    async updateProcesses() {
        try {
            const result = await API.safeGet('/processes/active');
            if (result.success && result.data.success) {
                this.processes = result.data.data || [];
                if (this.currentPage === 'processes') {
                    this.updateProcessList();
                }
            }
        } catch (error) {
            console.warn('Failed to update processes:', error);
        }
    }

    async loadMail() {
        try {
            const mail = await API.getMail();
            if (mail) {
                const mailCount = document.getElementById('mail-count');
                if (mailCount) {
                    mailCount.textContent = mail.unread_count || 0;
                }
                // TODO: Update mail page content
            }
        } catch (error) {
            console.warn('Failed to load mail:', error);
        }
    }

    async handleLogout() {
        try {
            await API.logout();
            window.location.href = 'login.html';
        } catch (error) {
            console.error('Logout error:', error);
            // Force logout even if API call fails
            API.clearAuthToken();
            window.location.href = 'login.html';
        }
    }

    showError(message) {
        console.error(message);
        this.showModal('Error', `<p class="error">${message}</p>`, [
            { text: 'OK', class: 'btn-primary', onclick: 'window.gameInstance.closeModal()' }
        ]);
    }

    showSuccess(message) {
        this.showModal('Success', `<p class="success">${message}</p>`, [
            { text: 'OK', class: 'btn-primary', onclick: 'window.gameInstance.closeModal()' }
        ]);
    }

    // Display functions for different data types
    displayServers(servers) {
        const content = document.querySelector('#servers-page .page-content') || document.getElementById('servers-page');
        if (!content) return;

        if (servers.length === 0) {
            content.innerHTML = '<h2>My Servers</h2><p>No servers owned</p>';
            return;
        }

        let html = '<h2>My Servers</h2><div class="servers-grid">';
        servers.forEach(server => {
            html += `
                <div class="server-item">
                    <h4>${server.ip}</h4>
                    <p>Type: ${server.server_type || 'Unknown'}</p>
                    <p>Status: ${server.is_online ? 'Online' : 'Offline'}</p>
                    <button onclick="window.gameInstance.manageServer(${server.id})" class="btn btn-primary">Manage</button>
                </div>
            `;
        });
        html += '</div>';
        content.innerHTML = html;
    }

    displayAvailableServers(servers) {
        const browserContent = document.getElementById('browser-content');
        if (!browserContent) return;

        if (servers.length === 0) {
            browserContent.innerHTML = '<div class="welcome-message"><h3>No servers found</h3><p>Try scanning the network first.</p></div>';
            return;
        }

        let html = '<h3>Available Servers</h3><div class="servers-list">';
        servers.forEach(server => {
            html += `
                <div class="server-item">
                    <div class="server-info">
                        <strong>${server.ip}</strong>
                        <span>Type: ${server.server_type || 'Unknown'}</span>
                        <span>Security: ${server.password_protected ? 'High' : 'Low'}</span>
                    </div>
                    <div class="server-actions">
                        <button onclick="window.gameInstance.hackServer('${server.ip}')" class="btn btn-primary">Hack</button>
                        <button onclick="window.gameInstance.scanServer('${server.ip}')" class="btn btn-secondary">Scan</button>
                    </div>
                </div>
            `;
        });
        html += '</div>';
        browserContent.innerHTML = html;
    }

    displayClanInfo(clanData) {
        const content = document.querySelector('#clan-page .page-content') || document.getElementById('clan-page');
        if (!content) return;

        const { clan, members } = clanData;
        let html = `
            <h2>${clan.name} [${clan.tag}]</h2>
            <p>${clan.description}</p>
            <div class="clan-stats">
                <span>Members: ${members.length}</span>
                <span>Founded: ${new Date(clan.created_at).toLocaleDateString()}</span>
            </div>
            <h3>Members</h3>
            <div class="members-list">
        `;
        
        members.forEach(member => {
            html += `
                <div class="member-item">
                    <span class="username">${member.username}</span>
                    <span class="level">Level ${this.calculateLevel(member.experience)}</span>
                    <span class="status ${member.is_online ? 'online' : 'offline'}">${member.is_online ? 'Online' : 'Offline'}</span>
                </div>
            `;
        });
        
        html += `
            </div>
            <div class="clan-actions">
                <button onclick="window.gameInstance.leaveClan()" class="btn btn-danger">Leave Clan</button>
            </div>
        `;
        
        content.innerHTML = html;
    }

    displayNoClan() {
        const content = document.querySelector('#clan-page .page-content') || document.getElementById('clan-page');
        if (!content) return;

        content.innerHTML = `
            <h2>Clan System</h2>
            <p>You are not a member of any clan.</p>
            <div class="clan-actions">
                <button onclick="window.gameInstance.showCreateClanForm()" class="btn btn-primary">Create Clan</button>
                <button onclick="window.gameInstance.showJoinClanForm()" class="btn btn-secondary">Find Clans</button>
            </div>
        `;
    }

    displayMissions(missions) {
        const content = document.querySelector('#missions-page .page-content') || document.getElementById('missions-page');
        if (!content) return;

        if (missions.length === 0) {
            content.innerHTML = '<h2>Active Missions</h2><p>No active missions</p>';
            return;
        }

        let html = '<h2>Active Missions</h2><div class="missions-list">';
        missions.forEach(mission => {
            html += `
                <div class="mission-item">
                    <h4>${mission.title}</h4>
                    <p>${mission.description}</p>
                    <div class="mission-info">
                        <span>Reward: $${mission.reward_money}</span>
                        <span>XP: ${mission.reward_experience}</span>
                        <span>Difficulty: ${mission.difficulty}</span>
                    </div>
                    <div class="mission-actions">
                        <button onclick="window.gameInstance.completeMission(${mission.id})" class="btn btn-primary">Complete</button>
                        <button onclick="window.gameInstance.abandonMission(${mission.id})" class="btn btn-danger">Abandon</button>
                    </div>
                </div>
            `;
        });
        html += '</div>';
        content.innerHTML = html;
    }

    displayRankings(rankings) {
        const content = document.querySelector('#ranking-page .page-content') || document.getElementById('ranking-page');
        if (!content) return;

        let html = '<h2>Player Rankings</h2><div class="rankings-list">';
        rankings.forEach((player, index) => {
            html += `
                <div class="ranking-item">
                    <span class="rank">#${index + 1}</span>
                    <span class="username">${player.username}</span>
                    <span class="level">Level ${this.calculateLevel(player.experience)}</span>
                    <span class="money">$${player.money.toLocaleString()}</span>
                    <span class="status ${player.is_online ? 'online' : 'offline'}">${player.is_online ? 'Online' : 'Offline'}</span>
                </div>
            `;
        });
        html += '</div>';
        content.innerHTML = html;
    }

    // Clan management functions
    async leaveClan() {
        if (confirm('Are you sure you want to leave your clan?')) {
            try {
                const result = await API.safePost('/clan/leave');
                if (result.success) {
                    this.showSuccess('Left clan successfully');
                    this.loadPageData('clan');
                } else {
                    this.showError(result.message || 'Failed to leave clan');
                }
            } catch (error) {
                this.showError('Failed to leave clan: ' + error.message);
            }
        }
    }

    // Mission management functions
    async completeMission(missionId) {
        try {
            const result = await API.safePost(`/missions/${missionId}/complete`);
            if (result.success && result.data.success) {
                const missionResult = result.data.data;
                this.showModal('Mission Complete!', `
                    <p>${missionResult.message}</p>
                    <p>Reward: $${missionResult.reward_money.toLocaleString()}</p>
                    <p>Experience: ${missionResult.reward_experience}</p>
                `, [
                    { text: 'OK', class: 'btn-primary', onclick: 'window.gameInstance.closeModal(); window.gameInstance.loadPageData("missions");' }
                ]);
            } else {
                this.showError(result.message || 'Failed to complete mission');
            }
        } catch (error) {
            this.showError('Failed to complete mission: ' + error.message);
        }
    }

    async abandonMission(missionId) {
        if (confirm('Are you sure you want to abandon this mission?')) {
            try {
                const result = await API.safePost(`/missions/${missionId}/abandon`);
                if (result.success) {
                    this.showSuccess('Mission abandoned');
                    this.loadPageData('missions');
                } else {
                    this.showError(result.message || 'Failed to abandon mission');
                }
            } catch (error) {
                this.showError('Failed to abandon mission: ' + error.message);
            }
        }
    }

    // Additional page loading functions
    async loadChat() {
        try {
            const result = await API.safeGet('/chat/history', { limit: 50 });
            if (result.success && result.data.success) {
                this.displayChatMessages(result.data.data || []);
            }
        } catch (error) {
            console.warn('Failed to load chat history:', error);
        }
    }

    async loadLogs() {
        try {
            const result = await API.safeGet('/logs/recent');
            if (result.success && result.data.success) {
                this.displayLogs(result.data.data || []);
            }
        } catch (error) {
            console.warn('Failed to load logs:', error);
        }
    }

    async loadConnections() {
        // This would show active network connections
        const content = document.querySelector('#connections-page .connections-content');
        if (content) {
            content.innerHTML = '<p>Network connections feature will be implemented in a future update.</p>';
        }
    }

    // Display functions
    displayChatMessages(messages) {
        const chatMessages = document.getElementById('chat-messages');
        if (!chatMessages) return;

        chatMessages.innerHTML = '';
        messages.forEach(msg => {
            this.addChatMessage(msg.sender, msg.message, msg.timestamp);
        });

        // Scroll to bottom
        chatMessages.scrollTop = chatMessages.scrollHeight;
    }

    addChatMessage(sender, message, timestamp) {
        const chatMessages = document.getElementById('chat-messages');
        if (!chatMessages) return;

        const messageEl = document.createElement('div');
        messageEl.className = 'chat-message';
        const time = new Date(timestamp).toLocaleTimeString();
        messageEl.innerHTML = `
            <span class="chat-time">[${time}]</span>
            <span class="chat-sender">${sender}:</span>
            <span class="chat-text">${message}</span>
        `;
        chatMessages.appendChild(messageEl);
        
        // Auto-scroll to bottom
        chatMessages.scrollTop = chatMessages.scrollHeight;
    }

    displayLogs(logs) {
        const logsList = document.getElementById('logs-list');
        if (!logsList) return;

        if (logs.length === 0) {
            logsList.innerHTML = '<p class="no-logs">No logs found</p>';
            return;
        }

        logsList.innerHTML = '';
        logs.forEach(log => {
            const logItem = document.createElement('div');
            logItem.className = 'log-item';
            const time = new Date(log.created_at).toLocaleString();
            logItem.innerHTML = `
                <div class="log-header">
                    <span class="log-time">${time}</span>
                    <span class="log-type">${log.log_type}</span>
                </div>
                <div class="log-message">${log.message}</div>
            `;
            logsList.appendChild(logItem);
        });
    }

    // Chat functions
    sendChatMessage() {
        const chatInput = document.getElementById('chat-input');
        if (!chatInput) return;

        const message = chatInput.value.trim();
        if (!message) return;

        // Send via WebSocket for real-time delivery
        if (window.WebSocketManager && window.WebSocketManager.isConnected) {
            window.WebSocketManager.sendChatMessage(message);
        } else {
            // Fallback to API
            this.sendChatMessageAPI(message);
        }

        chatInput.value = '';
    }

    async sendChatMessageAPI(message) {
        try {
            const result = await API.safePost('/chat/send', { message });
            if (!result.success) {
                this.showError('Failed to send message');
            }
        } catch (error) {
            this.showError('Failed to send message: ' + error.message);
        }
    }

    // Log functions
    refreshLogs() {
        this.loadPageData('logs');
    }

    async clearLogs() {
        if (confirm('Clear all system logs?')) {
            try {
                const result = await API.safePost('/logs/clear');
                if (result.success && result.data.success) {
                    this.showSuccess('Logs cleared');
                    this.loadPageData('logs');
                } else {
                    this.showError(result.message || 'Failed to clear logs');
                }
            } catch (error) {
                this.showError('Failed to clear logs: ' + error.message);
            }
        }
    }

    // Network functions
    async scanNetwork() {
        const browserContent = document.getElementById('browser-content');
        if (!browserContent) return;

        browserContent.innerHTML = '<div class="loading">Scanning network...</div>';

        try {
            const result = await API.safePost('/network/scan');
            if (result.success && result.data.success) {
                const scanResults = result.data.data;
                let html = '<h3>Network Scan Results</h3><div class="scan-results">';
                scanResults.forEach(server => {
                    html += `
                        <div class="scan-result">
                            <div class="server-ip">${server.ip}</div>
                            <div class="server-info">
                                ${server.hostname ? `<p>Hostname: ${server.hostname}</p>` : ''}
                                ${server.os ? `<p>OS: ${server.os}</p>` : ''}
                                <p>Ports: ${server.open_ports.join(', ')}</p>
                                <p>Response time: ${server.response_time}ms</p>
                            </div>
                            <div class="server-actions">
                                <button onclick="document.getElementById('address-input').value='${server.ip}'; window.gameInstance.handleConnect()" class="btn btn-primary">Connect</button>
                            </div>
                        </div>
                    `;
                });
                html += '</div>';
                browserContent.innerHTML = html;
            } else {
                browserContent.innerHTML = '<div class="error">Network scan failed</div>';
            }
        } catch (error) {
            browserContent.innerHTML = `<div class="error">Network scan failed: ${error.message}</div>`;
        }
    }
}

// Auto-initialize if DOM is already loaded
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => Game.init());
} else {
    Game.init();
}