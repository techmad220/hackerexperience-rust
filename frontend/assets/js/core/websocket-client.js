/**
 * HackerExperience WebSocket Client
 * Real-time communication system for live game updates
 * 
 * Handles all real-time features including:
 * - Process completion notifications
 * - Live player statistics updates
 * - Chat messages
 * - Attack notifications
 * - Clan war updates
 * - Market updates
 */

class HackerExperienceWebSocket {
    constructor(config = {}) {
        this.config = {
            url: config.url || window.GAME_CONFIG?.wsUrl || 'ws://localhost:8080/ws',
            reconnectInterval: config.reconnectInterval || 5000,
            maxReconnectAttempts: config.maxReconnectAttempts || 10,
            heartbeatInterval: config.heartbeatInterval || 30000,
            debug: config.debug || window.GAME_CONFIG?.debug || false,
            ...config
        };

        this.ws = null;
        this.reconnectAttempts = 0;
        this.heartbeatTimer = null;
        this.messageQueue = [];
        this.eventListeners = new Map();
        this.isConnected = false;
        this.sessionId = window.GAME_CONFIG?.session_id || '';
        
        this.init();
    }

    init() {
        this.connect();
        this.setupEventListeners();
        
        // Handle page visibility changes
        document.addEventListener('visibilitychange', () => {
            if (document.visibilityState === 'visible') {
                if (!this.isConnected) {
                    this.connect();
                }
            }
        });

        // Handle online/offline events
        window.addEventListener('online', () => {
            if (!this.isConnected) {
                this.connect();
            }
        });

        window.addEventListener('offline', () => {
            this.disconnect();
        });

        if (this.config.debug) {
            console.log('[WS] WebSocket client initialized', this.config);
        }
    }

    connect() {
        if (this.ws && this.ws.readyState === WebSocket.OPEN) {
            return;
        }

        try {
            const wsUrl = `${this.config.url}?session=${encodeURIComponent(this.sessionId)}`;
            this.ws = new WebSocket(wsUrl);

            this.ws.onopen = this.onOpen.bind(this);
            this.ws.onmessage = this.onMessage.bind(this);
            this.ws.onclose = this.onClose.bind(this);
            this.ws.onerror = this.onError.bind(this);

            if (this.config.debug) {
                console.log('[WS] Connecting to:', wsUrl);
            }
        } catch (error) {
            console.error('[WS] Connection failed:', error);
            this.scheduleReconnect();
        }
    }

    onOpen(event) {
        this.isConnected = true;
        this.reconnectAttempts = 0;
        
        if (this.config.debug) {
            console.log('[WS] Connected successfully');
        }

        // Send authentication message
        this.send({
            type: 'auth',
            session_id: this.sessionId,
            user_agent: navigator.userAgent,
            timestamp: Date.now()
        });

        // Start heartbeat
        this.startHeartbeat();

        // Process queued messages
        while (this.messageQueue.length > 0) {
            const message = this.messageQueue.shift();
            this.send(message);
        }

        // Emit connection event
        this.emit('connected', event);
    }

    onMessage(event) {
        try {
            const data = JSON.parse(event.data);
            
            if (this.config.debug) {
                console.log('[WS] Received:', data);
            }

            this.handleMessage(data);
        } catch (error) {
            console.error('[WS] Message parse error:', error, event.data);
        }
    }

    onClose(event) {
        this.isConnected = false;
        this.stopHeartbeat();
        
        if (this.config.debug) {
            console.log('[WS] Connection closed:', event.code, event.reason);
        }

        this.emit('disconnected', event);

        // Attempt reconnection if not a clean close
        if (event.code !== 1000) {
            this.scheduleReconnect();
        }
    }

    onError(event) {
        console.error('[WS] WebSocket error:', event);
        this.emit('error', event);
    }

    handleMessage(data) {
        // Handle heartbeat responses
        if (data.type === 'pong') {
            return;
        }

        // Handle authentication responses
        if (data.type === 'auth_response') {
            if (data.success) {
                this.emit('authenticated', data);
                this.subscribeToChannels();
            } else {
                console.error('[WS] Authentication failed:', data.message);
                this.emit('auth_failed', data);
            }
            return;
        }

        // Route messages to specific handlers
        switch (data.type) {
            case 'process_update':
                this.handleProcessUpdate(data);
                break;
            case 'process_complete':
                this.handleProcessComplete(data);
                break;
            case 'stats_update':
                this.handleStatsUpdate(data);
                break;
            case 'notification':
                this.handleNotification(data);
                break;
            case 'chat_message':
                this.handleChatMessage(data);
                break;
            case 'attack_started':
                this.handleAttackStarted(data);
                break;
            case 'attack_completed':
                this.handleAttackCompleted(data);
                break;
            case 'clan_update':
                this.handleClanUpdate(data);
                break;
            case 'war_update':
                this.handleWarUpdate(data);
                break;
            case 'market_update':
                this.handleMarketUpdate(data);
                break;
            case 'server_status':
                this.handleServerStatus(data);
                break;
            case 'player_online':
                this.handlePlayerOnline(data);
                break;
            case 'player_offline':
                this.handlePlayerOffline(data);
                break;
            default:
                // Generic message handling
                this.emit(data.type, data);
                break;
        }

        // Always emit a generic message event
        this.emit('message', data);
    }

    subscribeToChannels() {
        // Subscribe to user-specific channel
        this.send({
            type: 'subscribe',
            channel: 'user'
        });

        // Subscribe to global channels
        this.send({
            type: 'subscribe',
            channel: 'global'
        });

        // Subscribe to clan channel if user is in a clan
        if (window.GAME_CONFIG?.user?.clan_id) {
            this.send({
                type: 'subscribe',
                channel: `clan_${window.GAME_CONFIG.user.clan_id}`
            });
        }
    }

    // Message Handlers
    handleProcessUpdate(data) {
        // Update process progress in UI
        const processElement = document.querySelector(`[data-process-id="${data.process_id}"]`);
        if (processElement) {
            const progressBar = processElement.querySelector('.progress-bar');
            if (progressBar) {
                progressBar.style.width = `${data.progress}%`;
                progressBar.textContent = `${data.progress}%`;
            }

            const timeLeft = processElement.querySelector('.time-left');
            if (timeLeft) {
                timeLeft.textContent = this.formatTimeLeft(data.time_remaining);
            }
        }

        this.emit('process_update', data);
    }

    handleProcessComplete(data) {
        // Show notification
        window.HackerExperience?.showNotification(
            'Process Complete',
            `${data.process_name} completed successfully`,
            'success'
        );

        // Play sound effect
        window.HackerExperience?.playSound('process-complete');

        // Remove process from UI
        const processElement = document.querySelector(`[data-process-id="${data.process_id}"]`);
        if (processElement) {
            processElement.remove();
        }

        // Update process count
        this.updateProcessCount();

        this.emit('process_complete', data);
    }

    handleStatsUpdate(data) {
        // Update money
        if (data.money !== undefined) {
            const moneyElements = document.querySelectorAll('#player-money, .player-money');
            moneyElements.forEach(el => {
                el.textContent = this.formatCurrency(data.money);
            });
        }

        // Update cryptocurrency
        if (data.crypto !== undefined) {
            const cryptoElements = document.querySelectorAll('#player-crypto, .player-crypto');
            cryptoElements.forEach(el => {
                el.textContent = `₿${data.crypto.toFixed(4)}`;
            });
        }

        // Update level
        if (data.level !== undefined) {
            const levelElements = document.querySelectorAll('#player-level, .player-level');
            levelElements.forEach(el => {
                el.textContent = data.level;
            });
        }

        // Update experience
        if (data.experience !== undefined) {
            const expElements = document.querySelectorAll('.player-exp');
            expElements.forEach(el => {
                el.textContent = data.experience;
            });
        }

        this.emit('stats_update', data);
    }

    handleNotification(data) {
        // Show toast notification
        window.HackerExperience?.showNotification(data.title, data.message, data.type);

        // Update notification count
        const notificationCount = document.getElementById('notification-count');
        if (notificationCount && data.unread_count !== undefined) {
            notificationCount.textContent = data.unread_count;
            notificationCount.style.display = data.unread_count > 0 ? 'flex' : 'none';
        }

        this.emit('notification', data);
    }

    handleChatMessage(data) {
        const chatContainer = document.getElementById('chat-messages');
        if (chatContainer) {
            const messageElement = this.createChatMessageElement(data);
            chatContainer.appendChild(messageElement);
            chatContainer.scrollTop = chatContainer.scrollHeight;
        }

        // Play sound for new messages (if not from current user)
        if (data.user_id !== window.GAME_CONFIG?.user?.id) {
            window.HackerExperience?.playSound('message-received');
        }

        this.emit('chat_message', data);
    }

    handleAttackStarted(data) {
        if (data.target_user_id === window.GAME_CONFIG?.user?.id) {
            // User is being attacked
            window.HackerExperience?.showNotification(
                'Under Attack!',
                `${data.attacker_name} is attempting to hack your server!`,
                'warning'
            );
            window.HackerExperience?.playSound('attack-warning');
        }

        this.emit('attack_started', data);
    }

    handleAttackCompleted(data) {
        if (data.target_user_id === window.GAME_CONFIG?.user?.id) {
            // User was attacked
            const success = data.success ? 'successful' : 'failed';
            window.HackerExperience?.showNotification(
                'Attack Completed',
                `Attack by ${data.attacker_name} ${success}`,
                data.success ? 'error' : 'info'
            );
        } else if (data.attacker_user_id === window.GAME_CONFIG?.user?.id) {
            // User completed an attack
            window.HackerExperience?.showNotification(
                'Attack Completed',
                `Your attack on ${data.target_name} was ${data.success ? 'successful' : 'unsuccessful'}`,
                data.success ? 'success' : 'warning'
            );
        }

        this.emit('attack_completed', data);
    }

    handleClanUpdate(data) {
        // Update clan information in UI
        const clanElements = document.querySelectorAll('.clan-info');
        clanElements.forEach(el => {
            // Update clan-specific data
            const clanName = el.querySelector('.clan-name');
            if (clanName && data.name) {
                clanName.textContent = data.name;
            }

            const memberCount = el.querySelector('.member-count');
            if (memberCount && data.member_count !== undefined) {
                memberCount.textContent = data.member_count;
            }
        });

        this.emit('clan_update', data);
    }

    handleWarUpdate(data) {
        // Show war-related notifications
        if (data.event_type === 'war_declared') {
            window.HackerExperience?.showNotification(
                'War Declared!',
                `${data.attacking_clan} has declared war on ${data.defending_clan}`,
                'warning'
            );
        } else if (data.event_type === 'war_ended') {
            window.HackerExperience?.showNotification(
                'War Ended',
                `War between ${data.clan1} and ${data.clan2} has ended. Winner: ${data.winner}`,
                'info'
            );
        }

        this.emit('war_update', data);
    }

    handleMarketUpdate(data) {
        // Refresh market view if it's currently visible
        const marketPage = document.getElementById('market-page');
        if (marketPage && !marketPage.classList.contains('hidden')) {
            window.HackerExperience?.refreshMarket();
        }

        this.emit('market_update', data);
    }

    handleServerStatus(data) {
        // Update server status indicator
        const statusElements = document.querySelectorAll('.server-status');
        statusElements.forEach(el => {
            el.className = `server-status status-${data.status}`;
            const statusText = el.querySelector('.status-text');
            if (statusText) {
                statusText.textContent = data.status_text;
            }
        });

        this.emit('server_status', data);
    }

    handlePlayerOnline(data) {
        // Update online player count
        const onlineCountElements = document.querySelectorAll('#online-users-count, .online-users-count');
        onlineCountElements.forEach(el => {
            el.textContent = data.total_online;
        });

        // Update specific player status if visible
        const playerElement = document.querySelector(`[data-player-id="${data.user_id}"]`);
        if (playerElement) {
            const statusIndicator = playerElement.querySelector('.status-indicator');
            if (statusIndicator) {
                statusIndicator.className = 'status-indicator status-online';
            }
        }

        this.emit('player_online', data);
    }

    handlePlayerOffline(data) {
        // Update specific player status if visible
        const playerElement = document.querySelector(`[data-player-id="${data.user_id}"]`);
        if (playerElement) {
            const statusIndicator = playerElement.querySelector('.status-indicator');
            if (statusIndicator) {
                statusIndicator.className = 'status-indicator status-offline';
            }
        }

        this.emit('player_offline', data);
    }

    // Utility methods
    send(data) {
        if (this.isConnected && this.ws.readyState === WebSocket.OPEN) {
            const message = JSON.stringify(data);
            this.ws.send(message);
            
            if (this.config.debug) {
                console.log('[WS] Sent:', data);
            }
        } else {
            // Queue message for later sending
            this.messageQueue.push(data);
        }
    }

    startHeartbeat() {
        this.stopHeartbeat();
        this.heartbeatTimer = setInterval(() => {
            this.send({ type: 'ping', timestamp: Date.now() });
        }, this.config.heartbeatInterval);
    }

    stopHeartbeat() {
        if (this.heartbeatTimer) {
            clearInterval(this.heartbeatTimer);
            this.heartbeatTimer = null;
        }
    }

    scheduleReconnect() {
        if (this.reconnectAttempts >= this.config.maxReconnectAttempts) {
            console.error('[WS] Max reconnection attempts reached');
            this.emit('max_reconnect_attempts_reached');
            return;
        }

        this.reconnectAttempts++;
        const delay = this.config.reconnectInterval * Math.pow(2, this.reconnectAttempts - 1);
        
        if (this.config.debug) {
            console.log(`[WS] Reconnecting in ${delay}ms (attempt ${this.reconnectAttempts})`);
        }

        setTimeout(() => {
            this.connect();
        }, delay);
    }

    disconnect() {
        this.isConnected = false;
        this.stopHeartbeat();
        
        if (this.ws) {
            this.ws.close(1000, 'Client disconnecting');
            this.ws = null;
        }
    }

    // Event System
    on(event, callback) {
        if (!this.eventListeners.has(event)) {
            this.eventListeners.set(event, []);
        }
        this.eventListeners.get(event).push(callback);
    }

    off(event, callback) {
        const listeners = this.eventListeners.get(event);
        if (listeners) {
            const index = listeners.indexOf(callback);
            if (index > -1) {
                listeners.splice(index, 1);
            }
        }
    }

    emit(event, data) {
        const listeners = this.eventListeners.get(event);
        if (listeners) {
            listeners.forEach(callback => {
                try {
                    callback(data);
                } catch (error) {
                    console.error('[WS] Event listener error:', error);
                }
            });
        }
    }

    // Helper methods
    formatCurrency(amount) {
        return new Intl.NumberFormat('en-US', {
            style: 'currency',
            currency: 'USD',
            minimumFractionDigits: 2
        }).format(amount);
    }

    formatTimeLeft(seconds) {
        if (seconds < 60) {
            return `${seconds}s`;
        } else if (seconds < 3600) {
            const minutes = Math.floor(seconds / 60);
            const remainingSeconds = seconds % 60;
            return `${minutes}m ${remainingSeconds}s`;
        } else {
            const hours = Math.floor(seconds / 3600);
            const minutes = Math.floor((seconds % 3600) / 60);
            return `${hours}h ${minutes}m`;
        }
    }

    createChatMessageElement(data) {
        const messageElement = document.createElement('div');
        messageElement.className = 'chat-message';
        messageElement.innerHTML = `
            <div class="message-header">
                <span class="message-author">${this.escapeHtml(data.username)}</span>
                <span class="message-timestamp">${this.formatTimestamp(data.timestamp)}</span>
            </div>
            <div class="message-content">${this.escapeHtml(data.message)}</div>
        `;
        return messageElement;
    }

    formatTimestamp(timestamp) {
        const date = new Date(timestamp);
        return date.toLocaleTimeString();
    }

    escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }

    updateProcessCount() {
        const processCountElements = document.querySelectorAll('#active-processes-count, .active-processes-count');
        const processElements = document.querySelectorAll('.process-item');
        
        processCountElements.forEach(el => {
            el.textContent = processElements.length;
        });
    }

    setupEventListeners() {
        // Handle session changes
        window.addEventListener('session_updated', (event) => {
            this.sessionId = event.detail.sessionId;
            if (this.isConnected) {
                this.send({
                    type: 'session_update',
                    session_id: this.sessionId
                });
            }
        });

        // Handle user logout
        window.addEventListener('user_logout', () => {
            this.disconnect();
        });
    }

    // Public API methods
    sendChatMessage(channel, message) {
        this.send({
            type: 'chat_message',
            channel: channel,
            message: message
        });
    }

    joinChannel(channel) {
        this.send({
            type: 'subscribe',
            channel: channel
        });
    }

    leaveChannel(channel) {
        this.send({
            type: 'unsubscribe',
            channel: channel
        });
    }

    getConnectionStatus() {
        return {
            connected: this.isConnected,
            readyState: this.ws ? this.ws.readyState : WebSocket.CLOSED,
            reconnectAttempts: this.reconnectAttempts
        };
    }

    // Cleanup
    destroy() {
        this.stopHeartbeat();
        this.disconnect();
        this.eventListeners.clear();
        this.messageQueue = [];
        
        // Remove event listeners
        document.removeEventListener('visibilitychange', this.onVisibilityChange);
        window.removeEventListener('online', this.onOnline);
        window.removeEventListener('offline', this.onOffline);
    }
}

// Global WebSocket instance
if (typeof window !== 'undefined') {
    window.HackerExperienceWebSocket = HackerExperienceWebSocket;
    
    // Initialize global instance
    window.addEventListener('DOMContentLoaded', () => {
        if (window.GAME_CONFIG && window.GAME_CONFIG.session_id) {
            window.GameWebSocket = new HackerExperienceWebSocket(window.GAME_CONFIG);
        }
    });
}

// Export for module systems
if (typeof module !== 'undefined' && module.exports) {
    module.exports = HackerExperienceWebSocket;
}