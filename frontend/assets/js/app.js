/**
 * HackerExperience Main Application
 * Complete 1:1 frontend implementation with modern enhancements
 * 
 * This is the main application controller that orchestrates all frontend components,
 * manages the game state, handles user interactions, and provides the complete
 * gaming experience equivalent to the original PHP/JavaScript implementation.
 */

class HackerExperience {
    constructor(config = {}) {
        this.config = {
            debug: config.debug || false,
            autoSave: config.autoSave !== false,
            autoSaveInterval: config.autoSaveInterval || 30000,
            theme: config.theme || 'dark',
            soundEnabled: config.soundEnabled !== false,
            notificationsEnabled: config.notificationsEnabled !== false,
            ...config
        };

        // Core components
        this.api = null;
        this.websocket = null;
        this.terminal = null;
        this.currentPage = 'desktop';
        this.gameState = {};
        this.userStats = {};
        this.processes = new Map();
        this.notifications = [];
        
        // UI state
        this.isLoading = false;
        this.modalsOpen = new Set();
        this.currentModal = null;
        this.sidebarCollapsed = false;
        
        // Game systems
        this.processManager = null;
        this.softwareManager = null;
        this.hardwareManager = null;
        this.networkManager = null;
        
        // Timers and intervals
        this.autoSaveTimer = null;
        this.uiUpdateTimer = null;
        this.processUpdateTimer = null;
        
        this.initialized = false;
    }

    async init(gameConfig) {
        if (this.initialized) {
            console.warn('[App] Already initialized');
            return;
        }

        try {
            // Merge configuration
            this.config = { ...this.config, ...gameConfig };
            
            if (this.config.debug) {
                console.log('[App] Initializing HackerExperience', this.config);
            }

            // Show loading screen
            this.showLoadingScreen();

            // Initialize core systems
            await this.initializeCore();
            
            // Initialize UI components
            this.initializeUI();
            
            // Initialize game systems
            this.initializeGameSystems();
            
            // Load initial game state
            await this.loadGameState();
            
            // Start background timers
            this.startBackgroundTimers();
            
            // Setup event listeners
            this.setupEventListeners();
            
            // Hide loading screen
            this.hideLoadingScreen();
            
            this.initialized = true;
            
            if (this.config.debug) {
                console.log('[App] HackerExperience initialized successfully');
            }

            // Emit initialization complete event
            this.emit('initialized');
            
        } catch (error) {
            console.error('[App] Initialization failed:', error);
            this.showError('Initialization Failed', 'The game failed to initialize. Please refresh the page and try again.');
        }
    }

    async initializeCore() {
        // Initialize API client
        this.api = new HackerExperienceAPI(this.config);
        window.API = this.api;

        // Initialize WebSocket connection
        if (this.config.session_id) {
            this.websocket = new HackerExperienceWebSocket(this.config);
            window.GameWebSocket = this.websocket;
        }

        // Initialize terminal if desktop is loaded
        const terminalElement = document.getElementById('terminal-content');
        if (terminalElement) {
            this.terminal = new HackerExperienceTerminal('terminal-content', {
                currentServer: this.config.user?.current_server || 'localhost',
                currentPath: this.config.user?.current_path || '/home/user'
            });
        }
    }

    initializeUI() {
        // Initialize navigation
        this.initializeNavigation();
        
        // Initialize modals
        this.initializeModals();
        
        // Initialize tooltips
        this.initializeTooltips();
        
        // Initialize theme management
        this.initializeTheme();
        
        // Initialize responsive design
        this.initializeResponsive();
        
        // Initialize keyboard shortcuts
        this.initializeKeyboardShortcuts();
        
        // Initialize sound system
        this.initializeSounds();
        
        // Initialize notifications
        this.initializeNotifications();
    }

    initializeGameSystems() {
        // Process management system
        this.processManager = new ProcessManager(this.api, this.websocket);
        
        // Software management system  
        this.softwareManager = new SoftwareManager(this.api);
        
        // Hardware management system
        this.hardwareManager = new HardwareManager(this.api);
        
        // Network management system
        this.networkManager = new NetworkManager(this.api);
        
        // Banking system
        this.bankingSystem = new BankingSystem(this.api);
        
        // Clan system
        this.clanSystem = new ClanSystem(this.api, this.websocket);
        
        // Mission system
        this.missionSystem = new MissionSystem(this.api);
        
        if (this.config.debug) {
            console.log('[App] Game systems initialized');
        }
    }

    initializeNavigation() {
        // Handle navigation clicks
        document.addEventListener('click', (event) => {
            const navLink = event.target.closest('.nav-link');
            if (navLink && navLink.dataset.page) {
                event.preventDefault();
                this.navigateToPage(navLink.dataset.page);
            }
        });

        // Handle desktop icon clicks
        document.addEventListener('click', (event) => {
            const desktopIcon = event.target.closest('.desktop-icon');
            if (desktopIcon && desktopIcon.dataset.app) {
                this.launchApplication(desktopIcon.dataset.app);
            }
        });

        // Handle breadcrumb navigation
        document.addEventListener('click', (event) => {
            if (event.target.classList.contains('breadcrumb-link')) {
                event.preventDefault();
                this.navigateToPage(event.target.dataset.page);
            }
        });
    }

    async navigateToPage(pageName) {
        if (this.currentPage === pageName) return;

        const oldPage = document.querySelector('.page:not(.hidden)');
        const newPage = document.getElementById(`${pageName}-page`);

        if (!newPage) {
            console.error(`[App] Page not found: ${pageName}`);
            return;
        }

        // Hide old page
        if (oldPage) {
            oldPage.classList.add('hidden');
        }

        // Show new page
        newPage.classList.remove('hidden');

        // Update navigation state
        document.querySelectorAll('.nav-link').forEach(link => {
            link.classList.remove('active');
        });

        const activeNavLink = document.querySelector(`[data-page="${pageName}"]`);
        if (activeNavLink) {
            activeNavLink.classList.add('active');
        }

        // Update page title
        document.title = `${this.getPageTitle(pageName)} - HackerExperience`;

        // Load page-specific data
        await this.loadPageData(pageName);

        this.currentPage = pageName;
        
        if (this.config.debug) {
            console.log(`[App] Navigated to page: ${pageName}`);
        }

        // Emit navigation event
        this.emit('page_changed', { from: oldPage?.id, to: pageName });
    }

    getPageTitle(pageName) {
        const titles = {
            'desktop': 'Desktop',
            'processes': 'Processes',
            'software': 'Software',
            'hardware': 'Hardware',
            'internet': 'Internet Browser',
            'my-servers': 'My Servers',
            'connections': 'Network Connections',
            'banking': 'Banking',
            'crypto': 'Cryptocurrency',
            'market': 'Market',
            'clan': 'Clan Management',
            'missions': 'Missions',
            'ranking': 'Rankings',
            'mail': 'Mail System',
            'chat': 'Chat',
            'logs': 'System Logs',
            'profile': 'Profile',
            'settings': 'Settings'
        };
        return titles[pageName] || 'HackerExperience';
    }

    async loadPageData(pageName) {
        this.setLoading(true);

        try {
            switch (pageName) {
                case 'processes':
                    await this.loadProcesses();
                    break;
                case 'software':
                    await this.loadSoftware();
                    break;
                case 'hardware':
                    await this.loadHardware();
                    break;
                case 'internet':
                    await this.loadInternetBrowser();
                    break;
                case 'banking':
                    await this.loadBanking();
                    break;
                case 'clan':
                    await this.loadClan();
                    break;
                case 'missions':
                    await this.loadMissions();
                    break;
                case 'ranking':
                    await this.loadRanking();
                    break;
                case 'mail':
                    await this.loadMail();
                    break;
                case 'logs':
                    await this.loadLogs();
                    break;
            }
        } catch (error) {
            console.error(`[App] Failed to load page data for ${pageName}:`, error);
            this.showError('Load Error', `Failed to load ${pageName} data. Please try again.`);
        } finally {
            this.setLoading(false);
        }
    }

    async loadGameState() {
        try {
            // Load user profile and statistics
            const [profileResponse, statsResponse] = await Promise.all([
                this.api.getProfile(),
                this.api.getStats()
            ]);

            if (profileResponse?.success) {
                this.gameState.profile = profileResponse.data;
                this.updateUserInterface(profileResponse.data);
            }

            if (statsResponse?.success) {
                this.userStats = statsResponse.data;
                this.updateStatsDisplay();
            }

            // Load active processes
            await this.loadProcesses();
            
        } catch (error) {
            console.error('[App] Failed to load game state:', error);
        }
    }

    async loadProcesses() {
        try {
            const response = await this.api.getProcesses();
            if (response?.success) {
                this.processes.clear();
                response.data.processes.forEach(process => {
                    this.processes.set(process.id, process);
                });
                this.updateProcessesDisplay();
                this.updateProcessCount();
            }
        } catch (error) {
            console.error('[App] Failed to load processes:', error);
        }
    }

    async loadSoftware() {
        try {
            const response = await this.api.getSoftware();
            if (response?.success) {
                this.updateSoftwareDisplay(response.data.software);
            }
        } catch (error) {
            console.error('[App] Failed to load software:', error);
        }
    }

    async loadHardware() {
        try {
            const response = await this.api.getHardware();
            if (response?.success) {
                this.updateHardwareDisplay(response.data.hardware);
            }
        } catch (error) {
            console.error('[App] Failed to load hardware:', error);
        }
    }

    // Application launchers
    launchApplication(appName) {
        switch (appName) {
            case 'file-manager':
                this.navigateToPage('filesystem');
                break;
            case 'process-manager':
                this.navigateToPage('processes');
                break;
            case 'software-center':
                this.navigateToPage('software');
                break;
            case 'hardware-monitor':
                this.navigateToPage('hardware');
                break;
            case 'network-scanner':
                this.launchNetworkScanner();
                break;
            case 'internet-browser':
                this.navigateToPage('internet');
                break;
            case 'banking-client':
                this.navigateToPage('banking');
                break;
            case 'crypto-wallet':
                this.navigateToPage('crypto');
                break;
            case 'run-software':
                const softwareId = event.target.closest('[data-software-id]')?.dataset.softwareId;
                if (softwareId) {
                    this.runSoftware(softwareId);
                }
                break;
        }
    }

    launchNetworkScanner() {
        this.showModal('network-scanner-modal', {
            title: 'Network Scanner',
            content: this.createNetworkScannerContent()
        });
    }

    async runSoftware(softwareId) {
        try {
            const response = await this.api.createProcess({
                action: 'run_software',
                softwareId: softwareId,
                parameters: {}
            });

            if (response?.success) {
                this.showNotification('Software Started', 'Software is now running', 'success');
                this.loadProcesses();
            } else {
                this.showError('Failed to Start', response?.message || 'Could not start software');
            }
        } catch (error) {
            this.showError('Error', `Failed to start software: ${error.message}`);
        }
    }

    // UI Update Methods
    updateUserInterface(userData) {
        // Update player information in header
        const usernameElements = document.querySelectorAll('#username, .username');
        usernameElements.forEach(el => {
            el.textContent = userData.username;
        });

        // Update avatar
        const avatarElements = document.querySelectorAll('.avatar-img');
        avatarElements.forEach(el => {
            el.src = userData.avatar_url || '/assets/images/default-avatar.png';
        });

        // Update level
        const levelElements = document.querySelectorAll('#player-level, .player-level');
        levelElements.forEach(el => {
            el.textContent = userData.level || 1;
        });
    }

    updateStatsDisplay() {
        // Update money display
        const moneyElements = document.querySelectorAll('#player-money, .player-money');
        moneyElements.forEach(el => {
            el.textContent = this.formatCurrency(this.userStats.money || 0);
        });

        // Update crypto display
        const cryptoElements = document.querySelectorAll('#player-crypto, .player-crypto');
        cryptoElements.forEach(el => {
            el.textContent = `₿${(this.userStats.crypto || 0).toFixed(4)}`;
        });

        // Update other stats as needed
        this.updateSystemStats();
    }

    updateSystemStats() {
        // Update CPU usage
        const cpuElements = document.querySelectorAll('#cpu-usage');
        cpuElements.forEach(el => {
            el.textContent = `${Math.floor(Math.random() * 100)}%`;
        });

        // Update memory usage
        const memoryElements = document.querySelectorAll('#memory-usage');
        memoryElements.forEach(el => {
            el.textContent = `${Math.floor(Math.random() * 100)}%`;
        });

        // Update connection status
        const connectionElements = document.querySelectorAll('#connection-status');
        connectionElements.forEach(el => {
            el.textContent = this.websocket?.isConnected ? 'Connected' : 'Disconnected';
            el.className = `connection-status ${this.websocket?.isConnected ? 'online' : 'offline'}`;
        });
    }

    updateProcessCount() {
        const count = this.processes.size;
        const countElements = document.querySelectorAll('#active-processes-count, .active-processes-count');
        countElements.forEach(el => {
            el.textContent = count;
        });

        // Update badges
        const badgeElements = document.querySelectorAll('#processes-badge, .process-count');
        badgeElements.forEach(el => {
            el.textContent = count;
            el.style.display = count > 0 ? 'flex' : 'none';
        });
    }

    updateProcessesDisplay() {
        const processListElement = document.getElementById('process-list');
        if (!processListElement) return;

        if (this.processes.size === 0) {
            processListElement.innerHTML = `
                <div class="empty-state">
                    <i class="fas fa-tasks"></i>
                    <h3>No Active Processes</h3>
                    <p>Start a process by scanning networks or running software.</p>
                </div>
            `;
            return;
        }

        let html = '';
        this.processes.forEach(process => {
            const progress = this.calculateProcessProgress(process);
            const timeLeft = this.calculateTimeLeft(process);
            
            html += `
                <div class="process-item" data-process-id="${process.id}">
                    <div class="process-header">
                        <div class="process-info">
                            <h4 class="process-name">${process.action}</h4>
                            <p class="process-target">Target: ${process.target_ip || 'N/A'}</p>
                        </div>
                        <div class="process-actions">
                            <button class="btn btn-sm btn-warning" onclick="window.HackerExperience.pauseProcess(${process.id})">
                                <i class="fas fa-pause"></i>
                            </button>
                            <button class="btn btn-sm btn-danger" onclick="window.HackerExperience.cancelProcess(${process.id})">
                                <i class="fas fa-times"></i>
                            </button>
                        </div>
                    </div>
                    <div class="process-progress">
                        <div class="progress">
                            <div class="progress-bar" style="width: ${progress}%">${progress}%</div>
                        </div>
                        <div class="time-remaining">
                            <i class="fas fa-clock"></i>
                            <span class="time-left">${timeLeft}</span>
                        </div>
                    </div>
                    <div class="process-stats">
                        <div class="stat">
                            <span class="label">CPU:</span>
                            <span class="value">${process.cpu_usage || 0}%</span>
                        </div>
                        <div class="stat">
                            <span class="label">Network:</span>
                            <span class="value">${process.net_usage || 0}%</span>
                        </div>
                        <div class="stat">
                            <span class="label">Status:</span>
                            <span class="value status-${process.status}">${process.status}</span>
                        </div>
                    </div>
                </div>
            `;
        });

        processListElement.innerHTML = html;
    }

    // Modal Management
    initializeModals() {
        // Close modal when clicking outside
        document.addEventListener('click', (event) => {
            if (event.target.classList.contains('modal-overlay')) {
                this.closeModal(event.target.id);
            }
        });

        // Close modal when clicking close button
        document.addEventListener('click', (event) => {
            if (event.target.classList.contains('modal-close')) {
                const modal = event.target.closest('.modal-overlay');
                if (modal) {
                    this.closeModal(modal.id);
                }
            }
        });

        // Handle escape key
        document.addEventListener('keydown', (event) => {
            if (event.key === 'Escape' && this.currentModal) {
                this.closeModal(this.currentModal);
            }
        });
    }

    showModal(modalId, options = {}) {
        const modal = document.getElementById(modalId);
        if (!modal) {
            console.error(`[App] Modal not found: ${modalId}`);
            return;
        }

        // Update modal content if provided
        if (options.title) {
            const titleElement = modal.querySelector('.modal-title');
            if (titleElement) {
                titleElement.textContent = options.title;
            }
        }

        if (options.content) {
            const bodyElement = modal.querySelector('.modal-body');
            if (bodyElement) {
                bodyElement.innerHTML = options.content;
            }
        }

        // Show modal
        modal.classList.add('show');
        this.modalsOpen.add(modalId);
        this.currentModal = modalId;
        
        // Focus first focusable element
        const firstInput = modal.querySelector('input, button, select, textarea');
        if (firstInput) {
            setTimeout(() => firstInput.focus(), 100);
        }
    }

    closeModal(modalId) {
        const modal = document.getElementById(modalId);
        if (!modal) return;

        modal.classList.remove('show');
        this.modalsOpen.delete(modalId);
        
        if (this.currentModal === modalId) {
            this.currentModal = this.modalsOpen.size > 0 ? Array.from(this.modalsOpen)[0] : null;
        }
    }

    // Notification System
    initializeNotifications() {
        // Request notification permission
        if ('Notification' in window && Notification.permission === 'default') {
            Notification.requestPermission();
        }
    }

    showNotification(title, message, type = 'info', duration = 5000) {
        if (!this.config.notificationsEnabled) return;

        // Create toast notification
        const toast = this.createToastNotification(title, message, type);
        const container = document.getElementById('toast-container');
        if (container) {
            container.appendChild(toast);
            
            // Show toast
            setTimeout(() => toast.classList.add('show'), 10);
            
            // Auto-hide toast
            setTimeout(() => {
                toast.classList.remove('show');
                setTimeout(() => toast.remove(), 300);
            }, duration);
        }

        // Show browser notification if permitted
        if ('Notification' in window && Notification.permission === 'granted') {
            new Notification(title, {
                body: message,
                icon: '/assets/images/favicon.png'
            });
        }

        // Play notification sound
        if (type === 'error') {
            this.playSound('error');
        } else if (type === 'success') {
            this.playSound('success');
        } else {
            this.playSound('notification');
        }
    }

    createToastNotification(title, message, type) {
        const toast = document.createElement('div');
        toast.className = `toast ${type}`;
        
        const icons = {
            success: 'fas fa-check-circle',
            error: 'fas fa-exclamation-circle',
            warning: 'fas fa-exclamation-triangle',
            info: 'fas fa-info-circle'
        };

        toast.innerHTML = `
            <div class="toast-icon">
                <i class="${icons[type] || icons.info}"></i>
            </div>
            <div class="toast-content">
                <div class="toast-title">${title}</div>
                <div class="toast-message">${message}</div>
            </div>
            <button class="toast-close">
                <i class="fas fa-times"></i>
            </button>
        `;

        // Handle close button
        toast.querySelector('.toast-close').addEventListener('click', () => {
            toast.classList.remove('show');
            setTimeout(() => toast.remove(), 300);
        });

        return toast;
    }

    // Sound System
    initializeSounds() {
        this.sounds = {
            'notification': document.getElementById('notification-sound'),
            'process-complete': document.getElementById('process-complete-sound'),
            'error': document.getElementById('error-sound'),
            'success': document.getElementById('success-sound')
        };
    }

    playSound(soundName) {
        if (!this.config.soundEnabled) return;
        
        const sound = this.sounds[soundName];
        if (sound) {
            sound.currentTime = 0;
            sound.play().catch(() => {
                // Sound play failed, probably due to browser autoplay policy
            });
        }
    }

    // Theme Management
    initializeTheme() {
        const savedTheme = localStorage.getItem('he-theme') || this.config.theme;
        this.setTheme(savedTheme);

        // Handle theme toggle
        const themeToggle = document.getElementById('theme-toggle');
        if (themeToggle) {
            themeToggle.addEventListener('click', () => {
                const currentTheme = document.body.classList.contains('theme-light') ? 'light' : 'dark';
                const newTheme = currentTheme === 'dark' ? 'light' : 'dark';
                this.setTheme(newTheme);
            });
        }
    }

    setTheme(theme) {
        document.body.className = document.body.className.replace(/theme-\w+/g, '');
        document.body.classList.add(`theme-${theme}`);
        
        const themeCSS = document.getElementById('theme-css');
        if (themeCSS) {
            themeCSS.href = `/assets/css/themes/${theme}.css`;
        }
        
        localStorage.setItem('he-theme', theme);
        this.config.theme = theme;
    }

    // Error Handling
    showError(title, message) {
        this.showNotification(title, message, 'error', 8000);
    }

    // Loading States
    setLoading(loading) {
        this.isLoading = loading;
        const loadingElements = document.querySelectorAll('.loading-indicator');
        loadingElements.forEach(el => {
            el.style.display = loading ? 'block' : 'none';
        });
    }

    showLoadingScreen() {
        const loadingScreen = document.getElementById('loading-screen');
        if (loadingScreen) {
            loadingScreen.style.display = 'flex';
        }
    }

    hideLoadingScreen() {
        const loadingScreen = document.getElementById('loading-screen');
        if (loadingScreen) {
            loadingScreen.classList.add('fade-out');
            setTimeout(() => {
                loadingScreen.style.display = 'none';
            }, 500);
        }
    }

    // Background Timers
    startBackgroundTimers() {
        // Auto-save timer
        if (this.config.autoSave) {
            this.autoSaveTimer = setInterval(() => {
                this.autoSave();
            }, this.config.autoSaveInterval);
        }

        // UI update timer
        this.uiUpdateTimer = setInterval(() => {
            this.updateSystemStats();
            this.updateTime();
        }, 1000);

        // Process update timer
        this.processUpdateTimer = setInterval(() => {
            this.updateProcessProgress();
        }, 5000);
    }

    stopBackgroundTimers() {
        if (this.autoSaveTimer) {
            clearInterval(this.autoSaveTimer);
        }
        if (this.uiUpdateTimer) {
            clearInterval(this.uiUpdateTimer);
        }
        if (this.processUpdateTimer) {
            clearInterval(this.processUpdateTimer);
        }
    }

    async autoSave() {
        // Auto-save game state if needed
        if (this.config.debug) {
            console.log('[App] Auto-saving game state');
        }
    }

    updateTime() {
        const timeElements = document.querySelectorAll('#current-time, .current-time');
        const now = new Date();
        const timeString = now.toLocaleTimeString();
        
        timeElements.forEach(el => {
            el.textContent = timeString;
        });
    }

    updateProcessProgress() {
        this.processes.forEach((process, id) => {
            const processElement = document.querySelector(`[data-process-id="${id}"]`);
            if (processElement) {
                const progress = this.calculateProcessProgress(process);
                const timeLeft = this.calculateTimeLeft(process);
                
                const progressBar = processElement.querySelector('.progress-bar');
                const timeLeftElement = processElement.querySelector('.time-left');
                
                if (progressBar) {
                    progressBar.style.width = `${progress}%`;
                    progressBar.textContent = `${progress}%`;
                }
                
                if (timeLeftElement) {
                    timeLeftElement.textContent = timeLeft;
                }
                
                // Remove completed processes
                if (progress >= 100) {
                    this.processes.delete(id);
                    processElement.remove();
                    this.updateProcessCount();
                }
            }
        });
    }

    // Utility Methods
    calculateProcessProgress(process) {
        const now = Date.now();
        const startTime = new Date(process.created_at).getTime();
        const duration = process.duration * 1000; // Convert to milliseconds
        const elapsed = now - startTime;
        
        return Math.min(Math.floor((elapsed / duration) * 100), 100);
    }

    calculateTimeLeft(process) {
        const now = Date.now();
        const startTime = new Date(process.created_at).getTime();
        const duration = process.duration * 1000;
        const elapsed = now - startTime;
        const remaining = Math.max(0, duration - elapsed);
        
        return this.formatDuration(Math.floor(remaining / 1000));
    }

    formatDuration(seconds) {
        if (seconds < 60) return `${seconds}s`;
        if (seconds < 3600) return `${Math.floor(seconds / 60)}m ${seconds % 60}s`;
        return `${Math.floor(seconds / 3600)}h ${Math.floor((seconds % 3600) / 60)}m`;
    }

    formatCurrency(amount) {
        return new Intl.NumberFormat('en-US', {
            style: 'currency',
            currency: 'USD'
        }).format(amount);
    }

    // Event System
    setupEventListeners() {
        // WebSocket events
        if (this.websocket) {
            this.websocket.on('process_complete', (data) => {
                this.processes.delete(data.process_id);
                this.updateProcessCount();
                this.updateProcessesDisplay();
            });

            this.websocket.on('stats_update', (data) => {
                Object.assign(this.userStats, data);
                this.updateStatsDisplay();
            });
        }

        // Window events
        window.addEventListener('beforeunload', () => {
            this.destroy();
        });

        window.addEventListener('online', () => {
            this.showNotification('Connection Restored', 'Internet connection is back online', 'success');
        });

        window.addEventListener('offline', () => {
            this.showNotification('Connection Lost', 'Internet connection lost', 'warning');
        });
    }

    emit(eventName, data = null) {
        const event = new CustomEvent(eventName, { detail: data });
        window.dispatchEvent(event);
    }

    // Process Management
    async cancelProcess(processId) {
        try {
            const response = await this.api.cancelProcess(processId);
            if (response?.success) {
                this.processes.delete(processId);
                this.updateProcessCount();
                this.updateProcessesDisplay();
                this.showNotification('Process Cancelled', 'Process has been cancelled', 'info');
            }
        } catch (error) {
            this.showError('Error', `Failed to cancel process: ${error.message}`);
        }
    }

    async pauseProcess(processId) {
        try {
            const response = await this.api.pauseProcess(processId);
            if (response?.success) {
                this.showNotification('Process Paused', 'Process has been paused', 'info');
                this.loadProcesses();
            }
        } catch (error) {
            this.showError('Error', `Failed to pause process: ${error.message}`);
        }
    }

    // Network Operations
    async scanNetwork() {
        try {
            const response = await this.api.scanNetwork();
            if (response?.success) {
                this.showNotification('Scan Complete', 'Network scan completed successfully', 'success');
                // Handle scan results
            }
        } catch (error) {
            this.showError('Scan Failed', error.message);
        }
    }

    // Keyboard Shortcuts
    initializeKeyboardShortcuts() {
        document.addEventListener('keydown', (event) => {
            // Ctrl+/ - Show help
            if (event.ctrlKey && event.key === '/') {
                event.preventDefault();
                this.showHelp();
            }

            // Ctrl+Shift+D - Toggle debug mode
            if (event.ctrlKey && event.shiftKey && event.key === 'D') {
                this.config.debug = !this.config.debug;
                console.log(`[App] Debug mode ${this.config.debug ? 'enabled' : 'disabled'}`);
            }

            // F11 - Toggle fullscreen
            if (event.key === 'F11') {
                event.preventDefault();
                this.toggleFullscreen();
            }
        });
    }

    toggleFullscreen() {
        if (!document.fullscreenElement) {
            document.documentElement.requestFullscreen();
        } else {
            if (document.exitFullscreen) {
                document.exitFullscreen();
            }
        }
    }

    initializeResponsive() {
        // Handle sidebar toggle on mobile
        const sidebarToggle = document.getElementById('sidebar-toggle');
        if (sidebarToggle) {
            sidebarToggle.addEventListener('click', () => {
                this.toggleSidebar();
            });
        }

        // Handle window resize
        window.addEventListener('resize', () => {
            if (window.innerWidth > 768 && this.sidebarCollapsed) {
                this.showSidebar();
            }
        });
    }

    toggleSidebar() {
        const sidebar = document.getElementById('sidebar');
        if (sidebar) {
            sidebar.classList.toggle('collapsed');
            this.sidebarCollapsed = !this.sidebarCollapsed;
        }
    }

    showSidebar() {
        const sidebar = document.getElementById('sidebar');
        if (sidebar) {
            sidebar.classList.remove('collapsed');
            this.sidebarCollapsed = false;
        }
    }

    initializeTooltips() {
        // Initialize tooltips for elements with title attributes
        document.addEventListener('mouseenter', (event) => {
            if (event.target.hasAttribute('title') || event.target.hasAttribute('data-tooltip')) {
                this.showTooltip(event.target, event);
            }
        });

        document.addEventListener('mouseleave', (event) => {
            if (event.target.hasAttribute('title') || event.target.hasAttribute('data-tooltip')) {
                this.hideTooltip();
            }
        });
    }

    showTooltip(element, event) {
        const text = element.getAttribute('data-tooltip') || element.getAttribute('title');
        if (!text) return;

        const tooltip = document.getElementById('tooltip');
        if (tooltip) {
            tooltip.querySelector('#tooltip-content').textContent = text;
            tooltip.classList.add('show');
            
            const rect = element.getBoundingClientRect();
            tooltip.style.left = rect.left + (rect.width / 2) + 'px';
            tooltip.style.top = rect.top - tooltip.offsetHeight - 10 + 'px';
        }
    }

    hideTooltip() {
        const tooltip = document.getElementById('tooltip');
        if (tooltip) {
            tooltip.classList.remove('show');
        }
    }

    // Cleanup
    destroy() {
        this.stopBackgroundTimers();
        
        if (this.websocket) {
            this.websocket.destroy();
        }
        
        if (this.terminal) {
            this.terminal.destroy();
        }
        
        // Clear any remaining timers
        this.processes.clear();
        this.notifications = [];
        
        if (this.config.debug) {
            console.log('[App] HackerExperience destroyed');
        }
    }

    // Public API for global access
    getAPI() {
        return this.api;
    }

    getWebSocket() {
        return this.websocket;
    }

    getTerminal() {
        return this.terminal;
    }

    getCurrentPage() {
        return this.currentPage;
    }

    getGameState() {
        return this.gameState;
    }

    getUserStats() {
        return this.userStats;
    }
}

// Global application instance
if (typeof window !== 'undefined') {
    window.HackerExperience = HackerExperience;
    
    // Auto-initialize if configuration is available
    window.addEventListener('DOMContentLoaded', () => {
        if (window.GAME_CONFIG) {
            const app = new HackerExperience();
            app.init(window.GAME_CONFIG);
            window.GameApp = app;
        }
    });
}

// Export for module systems
if (typeof module !== 'undefined' && module.exports) {
    module.exports = HackerExperience;
}