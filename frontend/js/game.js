// Hacker Experience - Main Game Logic

// Debug logger helper
const DEBUG_GAME = window.DEBUG_GAME === true;
function gameDebug(...args) { if (DEBUG_GAME) { try { console.log('[GAME]', ...args); } catch(_) {} } }

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
        gameDebug('Initializing Hacker Experience...');
        
        try {
            // Show loading screen
            this.showLoadingScreen();
            
            // Initialize API connection
            await API.initialize();
            
            // Check if user is authenticated
            if (!API.isAuthenticated()) {
                gameDebug('User not authenticated, redirecting to login');
                window.location.href = 'login.html';
                return;
            }
            
            // Load player data
            const playerLoaded = await this.loadPlayerData();
            if (!playerLoaded) {
                gameDebug('Failed to load player data, redirecting to login');
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
            gameDebug('Game initialized successfully');
            
        } catch (error) {
            console.error('Failed to initialize game:', error);
            this.showError('Failed to initialize game: ' + error.message);
        }

        // Ensure timers are cleared when navigating away
        window.addEventListener('beforeunload', () => {
            if (this.updateTimeInterval) {
                try { clearInterval(this.updateTimeInterval); } catch (_) {}
            }
        });
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
            if (DEBUG_GAME) console.warn('Could not load player data:', error);
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
                    <h4>${escapeHTML(process.action || process.type)} Process</h4>
                    <p>Target: ${escapeHTML(process.target_ip || process.target || 'N/A')}</p>
                    <p>Status: ${escapeHTML(process.status)} | ${escapeHTML(timeLeftStr)}</p>
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

        while (softwareGrid.firstChild) softwareGrid.removeChild(softwareGrid.firstChild);

        if (this.software.length === 0) {
            const p = document.createElement('p');
            p.className = 'no-software';
            p.textContent = 'No software installed';
            softwareGrid.appendChild(p);
            return;
        }

        this.software.forEach(software => {
            const softwareItem = document.createElement('div');
            softwareItem.className = 'software-item';
            softwareItem.innerHTML = `
                <div class="icon">${this.getSoftwareIcon(software.type)}</div>
                <div class="name">${escapeHTML(software.name || software.type)}</div>
                <div class="version">v${software.version || '1.0'}</div>
            `;
            softwareItem.addEventListener('click', () => this.showSoftwareDetails(software));
            softwareGrid.appendChild(softwareItem);
        });
    }

    updateHardwareInfo(hardware) {
        const hardwareInfo = document.getElementById('hardware-info');
        if (!hardwareInfo || !hardware) return;

        while (hardwareInfo.firstChild) hardwareInfo.removeChild(hardwareInfo.firstChild);
        const grid = document.createElement('div');
        grid.className = 'hardware-grid';

        const mkItem = (title, type, spec) => {
            const wrap = document.createElement('div');
            wrap.className = 'hardware-item';
            const h4 = document.createElement('h4');
            h4.textContent = title;
            const p1 = document.createElement('p');
            p1.textContent = type;
            const p2 = document.createElement('p');
            p2.textContent = spec;
            wrap.appendChild(h4); wrap.appendChild(p1); wrap.appendChild(p2);
            return wrap;
        };

        grid.appendChild(mkItem('CPU', hardware.cpu?.type || 'Basic Processor', `Speed: ${hardware.cpu?.speed || 1000} MHz`));
        grid.appendChild(mkItem('Memory', hardware.memory?.type || 'Basic RAM', `Size: ${hardware.memory?.size || 1024} MB`));
        grid.appendChild(mkItem('Storage', hardware.storage?.type || 'Hard Drive', `Size: ${(hardware.storage?.size || 10000) / 1000} GB`));
        grid.appendChild(mkItem('Network', hardware.network?.type || 'Ethernet', `Speed: ${hardware.network?.speed || 100} Mbps`));
        hardwareInfo.appendChild(grid);
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
            if (DEBUG_GAME) console.warn(`Failed to load data for page ${page}:`, error);
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
                gameDebug('Unknown desktop action:', action);
        }
    }

    executeCommand(command) {
        const terminal = document.getElementById('terminal');
        if (!terminal) return;

        // Add command to terminal
        const commandLine = document.createElement('div');
        commandLine.className = 'terminal-line';
        commandLine.innerHTML = `<span class="prompt">root@localhost:~#</span> <span class="command">${escapeHTML(command)}</span>`;
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
        outputLine.innerHTML = escapeHTML(output).replace(/\n/g, '<br>');
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
            while (browserContent.firstChild) browserContent.removeChild(browserContent.firstChild);
            const div = document.createElement('div'); div.className = 'error'; div.textContent = 'Please enter an IP address';
            browserContent.appendChild(div);
            return;
        }

        while (browserContent.firstChild) browserContent.removeChild(browserContent.firstChild);
        { const div = document.createElement('div'); div.className = 'loading'; div.textContent = 'Connecting...'; browserContent.appendChild(div); }

        try {
            const result = await API.connectToServer(ip);
            if (result && result.success) {
                while (browserContent.firstChild) browserContent.removeChild(browserContent.firstChild);
                const wrap = document.createElement('div'); wrap.className = 'server-info';
                const h3 = document.createElement('h3'); h3.textContent = `Connected to ${ip}`; wrap.appendChild(h3);
                const p1 = document.createElement('p'); p1.textContent = `Server Type: ${result.server_type || 'Unknown'}`; wrap.appendChild(p1);
                const p2 = document.createElement('p'); p2.textContent = `Status: ${result.protected ? 'Protected' : 'Open'}`; wrap.appendChild(p2);
                if (Array.isArray(result.files) && result.files.length) {
                    const h4 = document.createElement('h4'); h4.textContent = 'Files:'; wrap.appendChild(h4);
                    const ul = document.createElement('ul');
                    result.files.forEach(file => { const li = document.createElement('li'); li.textContent = `${file.name} (${Number(file.size)||0} bytes)`; ul.appendChild(li); });
                    wrap.appendChild(ul);
                }
                browserContent.appendChild(wrap);
            } else {
                const msg = (result && result.message) ? result.message : 'Unknown error';
                while (browserContent.firstChild) browserContent.removeChild(browserContent.firstChild);
                const div = document.createElement('div'); div.className = 'error'; div.textContent = `Connection failed: ${msg}`; browserContent.appendChild(div);
            }
        } catch (error) {
            while (browserContent.firstChild) browserContent.removeChild(browserContent.firstChild);
            const div = document.createElement('div'); div.className = 'error'; div.textContent = `Connection failed: ${(error && error.message) || 'Error'}`; browserContent.appendChild(div);
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

        while (content.firstChild) content.removeChild(content.firstChild);
        const h2 = document.createElement('h2'); h2.textContent = 'My Servers'; content.appendChild(h2);
        if (servers.length === 0) {
            const p = document.createElement('p'); p.textContent = 'No servers owned'; content.appendChild(p); return;
        }
        const grid = document.createElement('div'); grid.className = 'servers-grid';
        servers.forEach(server => {
            const item = document.createElement('div'); item.className = 'server-item';
            const h4 = document.createElement('h4'); h4.textContent = String(server.ip); item.appendChild(h4);
            const pType = document.createElement('p'); pType.textContent = `Type: ${server.server_type || 'Unknown'}`; item.appendChild(pType);
            const pStatus = document.createElement('p'); pStatus.textContent = `Status: ${server.is_online ? 'Online' : 'Offline'}`; item.appendChild(pStatus);
            const btn = document.createElement('button'); btn.className='btn btn-primary'; btn.textContent='Manage'; btn.addEventListener('click', ()=> this.manageServer(server.id)); item.appendChild(btn);
            grid.appendChild(item);
        });
        content.appendChild(grid);
    }

    displayAvailableServers(servers) {
        const browserContent = document.getElementById('browser-content');
        if (!browserContent) return;

        while (browserContent.firstChild) browserContent.removeChild(browserContent.firstChild);
        if (servers.length === 0) {
            const div = document.createElement('div'); div.className='welcome-message';
            const h3 = document.createElement('h3'); h3.textContent='No servers found';
            const p = document.createElement('p'); p.textContent='Try scanning the network first.'; div.appendChild(h3); div.appendChild(p);
            browserContent.appendChild(div); return;
        }
        const h3 = document.createElement('h3'); h3.textContent = 'Available Servers'; browserContent.appendChild(h3);
        const list = document.createElement('div'); list.className='servers-list';
        servers.forEach(server => {
            const item = document.createElement('div'); item.className='server-item';
            const info = document.createElement('div'); info.className='server-info';
            const strong = document.createElement('strong'); strong.textContent = String(server.ip); info.appendChild(strong);
            const spanType = document.createElement('span'); spanType.textContent = `Type: ${server.server_type || 'Unknown'}`; info.appendChild(spanType);
            const spanSec = document.createElement('span'); spanSec.textContent = `Security: ${server.password_protected ? 'High' : 'Low'}`; info.appendChild(spanSec);
            const actions = document.createElement('div'); actions.className='server-actions';
            const btnHack = document.createElement('button'); btnHack.className='btn btn-primary'; btnHack.textContent='Hack'; btnHack.addEventListener('click', ()=> this.hackServer(server.ip));
            const btnScan = document.createElement('button'); btnScan.className='btn btn-secondary'; btnScan.textContent='Scan'; btnScan.addEventListener('click', ()=> this.scanServer(server.ip));
            actions.appendChild(btnHack); actions.appendChild(btnScan);
            item.appendChild(info); item.appendChild(actions);
            list.appendChild(item);
        });
        browserContent.appendChild(list);
    }

    displayClanInfo(clanData) {
        const content = document.querySelector('#clan-page .page-content') || document.getElementById('clan-page');
        if (!content) return;
        while (content.firstChild) content.removeChild(content.firstChild);
        const { clan, members } = clanData;
        const h2 = document.createElement('h2'); h2.textContent = `${clan.name} [${clan.tag}]`; content.appendChild(h2);
        const p = document.createElement('p'); p.textContent = clan.description || ''; content.appendChild(p);
        const stats = document.createElement('div'); stats.className='clan-stats';
        const s1 = document.createElement('span'); s1.textContent = `Members: ${members.length}`; stats.appendChild(s1);
        const s2 = document.createElement('span'); s2.textContent = `Founded: ${new Date(clan.created_at).toLocaleDateString()}`; stats.appendChild(s2);
        content.appendChild(stats);
        const h3 = document.createElement('h3'); h3.textContent = 'Members'; content.appendChild(h3);
        const list = document.createElement('div'); list.className='members-list';
        members.forEach(member => {
            const item = document.createElement('div'); item.className='member-item';
            const u = document.createElement('span'); u.className='username'; u.textContent = member.username; item.appendChild(u);
            const lvl = document.createElement('span'); lvl.className='level'; lvl.textContent = `Level ${this.calculateLevel(member.experience)}`; item.appendChild(lvl);
            const st = document.createElement('span'); st.className = `status ${member.is_online ? 'online' : 'offline'}`; st.textContent = member.is_online ? 'Online' : 'Offline'; item.appendChild(st);
            list.appendChild(item);
        });
        content.appendChild(list);
        const actions = document.createElement('div'); actions.className='clan-actions';
        const btn = document.createElement('button'); btn.className='btn btn-danger'; btn.textContent='Leave Clan'; btn.addEventListener('click', ()=> this.leaveClan()); actions.appendChild(btn);
        content.appendChild(actions);
    }

    displayNoClan() {
        const content = document.querySelector('#clan-page .page-content') || document.getElementById('clan-page');
        if (!content) return;
        while (content.firstChild) content.removeChild(content.firstChild);
        const h2 = document.createElement('h2'); h2.textContent = 'Clan System'; content.appendChild(h2);
        const p = document.createElement('p'); p.textContent = 'You are not a member of any clan.'; content.appendChild(p);
        const actions = document.createElement('div'); actions.className='clan-actions';
        const btnCreate = document.createElement('button'); btnCreate.className='btn btn-primary'; btnCreate.textContent = 'Create Clan'; btnCreate.addEventListener('click', ()=> this.showCreateClanForm());
        const btnJoin = document.createElement('button'); btnJoin.className='btn btn-secondary'; btnJoin.textContent = 'Find Clans'; btnJoin.addEventListener('click', ()=> this.showJoinClanForm());
        actions.appendChild(btnCreate); actions.appendChild(btnJoin);
        content.appendChild(actions);
    }

    displayMissions(missions) {
        const content = document.querySelector('#missions-page .page-content') || document.getElementById('missions-page');
        if (!content) return;

        while (content.firstChild) content.removeChild(content.firstChild);
        const h2m = document.createElement('h2'); h2m.textContent = 'Active Missions'; content.appendChild(h2m);
        if (missions.length === 0) { const p = document.createElement('p'); p.textContent='No active missions'; content.appendChild(p); return; }
        let html = '<div class="missions-list">';
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
        const wrap = document.createElement('div'); wrap.className='missions-list';
        // For now, keep simple injection for mission list; consider DOM builders next pass
        wrap.innerHTML = html.replace('<div class="missions-list">','').replace('</div>','');
        content.appendChild(wrap);
    }

    displayRankings(rankings) {
        const content = document.querySelector('#ranking-page .page-content') || document.getElementById('ranking-page');
        if (!content) return;
        while (content.firstChild) content.removeChild(content.firstChild);
        const h2 = document.createElement('h2'); h2.textContent = 'Player Rankings'; content.appendChild(h2);
        const list = document.createElement('div'); list.className='rankings-list';
        rankings.forEach((player, index) => {
            const item = document.createElement('div'); item.className='ranking-item';
            const r = document.createElement('span'); r.className='rank'; r.textContent = `#${index+1}`;
            const u = document.createElement('span'); u.className='username'; u.textContent = player.username;
            const l = document.createElement('span'); l.className='level'; l.textContent = `Level ${this.calculateLevel(player.experience)}`;
            const m = document.createElement('span'); m.className='money'; m.textContent = `$${(player.money||0).toLocaleString()}`;
            const s = document.createElement('span'); s.className = `status ${player.is_online ? 'online' : 'offline'}`; s.textContent = player.is_online ? 'Online' : 'Offline';
            item.appendChild(r); item.appendChild(u); item.appendChild(l); item.appendChild(m); item.appendChild(s);
            list.appendChild(item);
        });
        content.appendChild(list);
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
            while (content.firstChild) content.removeChild(content.firstChild);
            const p = document.createElement('p'); p.textContent = 'Network connections feature will be implemented in a future update.'; content.appendChild(p);
        }
    }

    // Display functions
    displayChatMessages(messages) {
        const chatMessages = document.getElementById('chat-messages');
        if (!chatMessages) return;

        while (chatMessages.firstChild) chatMessages.removeChild(chatMessages.firstChild);
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
        const spanTime = document.createElement('span'); spanTime.className = 'chat-time'; spanTime.textContent = `[${time}]`;
        const spanSender = document.createElement('span'); spanSender.className = 'chat-sender'; spanSender.textContent = `${sender}:`;
        const spanText = document.createElement('span'); spanText.className = 'chat-text'; spanText.textContent = message;
        messageEl.appendChild(spanTime); messageEl.appendChild(spanSender); messageEl.appendChild(spanText);
        chatMessages.appendChild(messageEl);
        
        // Auto-scroll to bottom
        chatMessages.scrollTop = chatMessages.scrollHeight;
    }

    displayLogs(logs) {
        const logsList = document.getElementById('logs-list');
        if (!logsList) return;

        while (logsList.firstChild) logsList.removeChild(logsList.firstChild);
        if (logs.length === 0) {
            const p = document.createElement('p'); p.className = 'no-logs'; p.textContent = 'No logs found'; logsList.appendChild(p); return;
        }
        logs.forEach(log => {
            const logItem = document.createElement('div'); logItem.className = 'log-item';
            const timeText = new Date(log.created_at).toLocaleString();
            const header = document.createElement('div'); header.className = 'log-header';
            const spanTime = document.createElement('span'); spanTime.className = 'log-time'; spanTime.textContent = timeText;
            const spanType = document.createElement('span'); spanType.className = 'log-type'; spanType.textContent = log.log_type;
            header.appendChild(spanTime); header.appendChild(spanType);
            const msg = document.createElement('div'); msg.className = 'log-message'; msg.textContent = log.message;
            logItem.appendChild(header); logItem.appendChild(msg);
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

// Basic HTML escaping to mitigate XSS when injecting dynamic content
function escapeHTML(value) {
    if (value === null || value === undefined) return '';
    return String(value)
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;')
        .replace(/'/g, '&#39;');
}

// Auto-initialize if DOM is already loaded
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => Game.init());
} else {
    Game.init();
}
