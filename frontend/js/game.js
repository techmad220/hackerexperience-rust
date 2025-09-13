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
            
            // Load player data
            await this.loadPlayerData();
            
            // Initialize WebSocket connection
            if (typeof WebSocketManager !== 'undefined') {
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
            const playerData = await API.getPlayerInfo();
            if (playerData) {
                this.player = { ...this.player, ...playerData };
                this.updatePlayerUI();
            }
        } catch (error) {
            console.warn('Could not load player data:', error);
        }
    }

    async loadInitialData() {
        try {
            // Load processes
            const processes = await API.getProcesses();
            if (processes) {
                this.processes = processes;
                this.updateProcessList();
            }

            // Load software
            const software = await API.getSoftware();
            if (software) {
                this.software = software;
                this.updateSoftwareGrid();
            }

            // Load hardware info
            const hardware = await API.getHardware();
            if (hardware) {
                this.updateHardwareInfo(hardware);
            }

        } catch (error) {
            console.warn('Could not load initial data:', error);
        }
    }

    updatePlayerUI() {
        const usernameEl = document.getElementById('username');
        const levelEl = document.getElementById('player-level');
        const moneyEl = document.getElementById('player-money');

        if (usernameEl) usernameEl.textContent = this.player.username;
        if (levelEl) levelEl.textContent = this.player.level;
        if (moneyEl) moneyEl.textContent = this.player.money.toLocaleString();
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
            processItem.innerHTML = `
                <div class="process-info">
                    <h4>${process.name || process.type}</h4>
                    <p>Target: ${process.target || 'N/A'} | Status: ${process.status}</p>
                </div>
                <div class="process-progress">
                    <div class="progress-bar">
                        <div class="progress-fill" style="width: ${process.progress || 0}%"></div>
                    </div>
                    <span class="progress-text">${process.progress || 0}%</span>
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
                    const processes = await API.getProcesses();
                    if (processes) {
                        this.processes = processes;
                        this.updateProcessList();
                    }
                    break;
                case 'software':
                    const software = await API.getSoftware();
                    if (software) {
                        this.software = software;
                        this.updateSoftwareGrid();
                    }
                    break;
                case 'hardware':
                    const hardware = await API.getHardware();
                    if (hardware) {
                        this.updateHardwareInfo(hardware);
                    }
                    break;
                case 'mail':
                    await this.loadMail();
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
            const processes = await API.getProcesses();
            if (processes) {
                this.processes = processes;
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
}

// Auto-initialize if DOM is already loaded
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => Game.init());
} else {
    Game.init();
}