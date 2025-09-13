// Game Client Service - Centralized State Management for Hacker Experience

class GameClient {
    constructor() {
        this.state = {
            player: {
                id: null,
                username: '',
                level: 1,
                money: 0,
                experience: 0,
                reputation: 0.0,
                clan_id: null,
                is_online: false
            },
            processes: [],
            software: [],
            hardware: [],
            servers: [],
            missions: [],
            clan: null,
            chat: [],
            logs: [],
            gameStats: null,
            isOnline: navigator.onLine,
            lastUpdate: null
        };

        this.listeners = new Map();
        this.updateIntervals = new Map();
        this.autoSaveInterval = null;
        
        // Initialize event listeners
        this.setupEventListeners();
        
        // Load saved state
        this.loadState();
        
        console.log('GameClient initialized');
    }

    // Event system for state changes
    on(event, callback) {
        if (!this.listeners.has(event)) {
            this.listeners.set(event, []);
        }
        this.listeners.get(event).push(callback);
    }

    off(event, callback) {
        if (this.listeners.has(event)) {
            const callbacks = this.listeners.get(event);
            const index = callbacks.indexOf(callback);
            if (index !== -1) {
                callbacks.splice(index, 1);
            }
        }
    }

    emit(event, data) {
        if (this.listeners.has(event)) {
            this.listeners.get(event).forEach(callback => {
                try {
                    callback(data);
                } catch (error) {
                    console.error(`Error in event listener for ${event}:`, error);
                }
            });
        }
    }

    // Setup browser event listeners
    setupEventListeners() {
        // Online/offline detection
        window.addEventListener('online', () => {
            this.updateState('isOnline', true);
            this.emit('connection:online');
            this.syncWithServer();
        });

        window.addEventListener('offline', () => {
            this.updateState('isOnline', false);
            this.emit('connection:offline');
        });

        // Visibility change - pause/resume updates
        document.addEventListener('visibilitychange', () => {
            if (document.hidden) {
                this.pauseUpdates();
            } else {
                this.resumeUpdates();
            }
        });

        // Before unload - save state
        window.addEventListener('beforeunload', () => {
            this.saveState();
        });
    }

    // State management
    updateState(key, value) {
        const oldValue = this.getState(key);
        
        if (key.includes('.')) {
            // Handle nested keys like 'player.money'
            const keys = key.split('.');
            let current = this.state;
            
            for (let i = 0; i < keys.length - 1; i++) {
                current = current[keys[i]];
            }
            
            current[keys[keys.length - 1]] = value;
        } else {
            this.state[key] = value;
        }

        this.state.lastUpdate = Date.now();
        
        // Emit change event
        this.emit(`state:${key}`, { oldValue, newValue: value });
        this.emit('state:changed', { key, oldValue, newValue: value });
        
        // Auto-save important changes
        if (this.shouldAutoSave(key)) {
            this.saveState();
        }
    }

    getState(key) {
        if (!key) return this.state;
        
        if (key.includes('.')) {
            // Handle nested keys
            const keys = key.split('.');
            let current = this.state;
            
            for (const k of keys) {
                current = current[k];
                if (current === undefined) return undefined;
            }
            
            return current;
        }
        
        return this.state[key];
    }

    // Player management
    updatePlayer(playerData) {
        const oldPlayer = { ...this.state.player };
        this.state.player = { ...this.state.player, ...playerData };
        this.state.lastUpdate = Date.now();
        
        this.emit('player:updated', { old: oldPlayer, new: this.state.player });
        this.saveState();
    }

    getPlayer() {
        return this.state.player;
    }

    // Process management
    addProcess(process) {
        const existingIndex = this.state.processes.findIndex(p => p.id === process.id);
        
        if (existingIndex !== -1) {
            // Update existing process
            this.state.processes[existingIndex] = { ...this.state.processes[existingIndex], ...process };
            this.emit('process:updated', process);
        } else {
            // Add new process
            this.state.processes.push(process);
            this.emit('process:added', process);
        }
        
        this.state.lastUpdate = Date.now();
    }

    removeProcess(processId) {
        const index = this.state.processes.findIndex(p => p.id === processId);
        if (index !== -1) {
            const removedProcess = this.state.processes.splice(index, 1)[0];
            this.emit('process:removed', removedProcess);
            this.state.lastUpdate = Date.now();
        }
    }

    updateProcess(processId, updates) {
        const index = this.state.processes.findIndex(p => p.id === processId);
        if (index !== -1) {
            const oldProcess = { ...this.state.processes[index] };
            this.state.processes[index] = { ...oldProcess, ...updates };
            this.emit('process:updated', { old: oldProcess, new: this.state.processes[index] });
            this.state.lastUpdate = Date.now();
        }
    }

    clearProcesses() {
        const oldProcesses = [...this.state.processes];
        this.state.processes = [];
        this.emit('processes:cleared', oldProcesses);
        this.state.lastUpdate = Date.now();
    }

    getProcesses() {
        return this.state.processes;
    }

    getActiveProcesses() {
        return this.state.processes.filter(p => p.status === 'running' || p.status === 'pending');
    }

    // Chat management
    addChatMessage(sender, message, timestamp) {
        const chatMessage = {
            id: Date.now() + Math.random(),
            sender,
            message,
            timestamp: timestamp || new Date().toISOString()
        };
        
        this.state.chat.push(chatMessage);
        
        // Keep only last 100 messages
        if (this.state.chat.length > 100) {
            this.state.chat = this.state.chat.slice(-100);
        }
        
        this.emit('chat:message', chatMessage);
        this.state.lastUpdate = Date.now();
    }

    getChatMessages() {
        return this.state.chat;
    }

    // Auto-update system
    startUpdates() {
        // Process updates every 2 seconds
        this.updateIntervals.set('processes', setInterval(() => {
            if (this.state.isOnline && this.getActiveProcesses().length > 0) {
                this.updateProcessProgress();
            }
        }, 2000));

        // Player data updates every 30 seconds
        this.updateIntervals.set('player', setInterval(() => {
            if (this.state.isOnline) {
                this.syncPlayerData();
            }
        }, 30000));

        // Auto-save every 60 seconds
        this.autoSaveInterval = setInterval(() => {
            this.saveState();
        }, 60000);
    }

    pauseUpdates() {
        this.updateIntervals.forEach((interval, key) => {
            clearInterval(interval);
        });
        this.updateIntervals.clear();
        
        if (this.autoSaveInterval) {
            clearInterval(this.autoSaveInterval);
            this.autoSaveInterval = null;
        }
    }

    resumeUpdates() {
        this.startUpdates();
    }

    // Sync with server
    async syncWithServer() {
        if (!this.state.isOnline || !window.API || !window.API.isAuthenticated()) {
            return;
        }

        try {
            await Promise.allSettled([
                this.syncPlayerData(),
                this.syncProcesses()
            ]);
            
            this.emit('sync:completed');
        } catch (error) {
            console.error('Sync failed:', error);
            this.emit('sync:failed', error);
        }
    }

    async syncPlayerData() {
        try {
            const result = await window.API.safeGet('/user/profile');
            if (result.success && result.data.success) {
                this.updatePlayer(result.data.data);
            }
        } catch (error) {
            console.warn('Failed to sync player data:', error);
        }
    }

    async syncProcesses() {
        try {
            const result = await window.API.safeGet('/processes/active');
            if (result.success && result.data.success) {
                const serverProcesses = result.data.data || [];
                
                serverProcesses.forEach(serverProcess => {
                    this.addProcess(serverProcess);
                });
                
                this.state.processes = this.state.processes.filter(localProcess => 
                    serverProcesses.some(serverProcess => serverProcess.id === localProcess.id)
                );
                
                this.emit('processes:synced', this.state.processes);
            }
        } catch (error) {
            console.warn('Failed to sync processes:', error);
        }
    }

    // Local process progress simulation
    updateProcessProgress() {
        this.state.processes.forEach(process => {
            if (process.status === 'running' && process.time_left > 0) {
                process.time_left = Math.max(0, process.time_left - 2);
                process.progress = 1.0 - (process.time_left / process.duration);
                
                if (process.time_left <= 0) {
                    process.status = 'completed';
                    process.progress = 1.0;
                }
                
                this.emit('process:progress', process);
            }
        });
    }

    // State persistence
    saveState() {
        try {
            const stateToSave = {
                player: this.state.player,
                chat: this.state.chat.slice(-20),
                lastUpdate: this.state.lastUpdate
            };
            
            localStorage.setItem('gameState', JSON.stringify(stateToSave));
        } catch (error) {
            console.error('Failed to save state:', error);
        }
    }

    loadState() {
        try {
            const savedState = localStorage.getItem('gameState');
            if (savedState) {
                const parsed = JSON.parse(savedState);
                
                this.state.player = { ...this.state.player, ...parsed.player };
                this.state.chat = parsed.chat || [];
                this.state.lastUpdate = parsed.lastUpdate;
                
                console.log('Game state loaded from localStorage');
            }
        } catch (error) {
            console.error('Failed to load state:', error);
        }
    }

    shouldAutoSave(key) {
        const autoSaveKeys = ['player', 'processes', 'chat'];
        return autoSaveKeys.some(saveKey => key.startsWith(saveKey));
    }

    // Cleanup
    destroy() {
        this.pauseUpdates();
        this.listeners.clear();
        this.saveState();
        console.log('GameClient destroyed');
    }
}

// Create global instance
window.GameClient = new GameClient();

console.log('GameClient service loaded');