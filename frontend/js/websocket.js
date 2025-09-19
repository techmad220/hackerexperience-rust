// WebSocket Manager for real-time game updates

class WebSocketManager {
    constructor() {
        this.socket = null;
        this.isConnected = false;
        this.reconnectAttempts = 0;
        this.maxReconnectAttempts = 5;
        this.reconnectDelay = 1000;
        this.messageQueue = [];
        this.subscriptions = new Set();
        
        this.eventHandlers = {
            'process_update': this.handleProcessUpdate.bind(this),
            'server_update': this.handleServerUpdate.bind(this),
            'player_update': this.handlePlayerUpdate.bind(this),
            'notification': this.handleNotification.bind(this),
            'mail_update': this.handleMailUpdate.bind(this),
            'clan_update': this.handleClanUpdate.bind(this),
            'system_message': this.handleSystemMessage.bind(this)
        };
    }

    connect(url = null) {
        const wsUrl = url || this.getWebSocketUrl();
        
        if (window.DEBUG_WEBSOCKET) console.log('Connecting to WebSocket:', wsUrl);
        
        try {
            this.socket = new WebSocket(wsUrl);
            this.setupEventHandlers();
        } catch (error) {
            console.error('Failed to create WebSocket connection:', error);
            this.scheduleReconnect();
        }
    }

    getWebSocketUrl() {
        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const host = window.location.host;
        return `${protocol}//${host}/ws`;
    }

    setupEventHandlers() {
        if (!this.socket) return;

        this.socket.onopen = (event) => {
            if (window.DEBUG_WEBSOCKET) console.log('WebSocket connected');
            this.isConnected = true;
            this.reconnectAttempts = 0;
            this.updateConnectionStatus(true);
            
            // Send queued messages
            this.processMessageQueue();
            
            // Send authentication if we have player data
            this.authenticate();
            
            // Subscribe to default channels
            this.subscribeToDefaultChannels();
        };

        this.socket.onmessage = (event) => {
            try {
                const message = JSON.parse(event.data);
                this.handleMessage(message);
            } catch (error) {
                if (window.DEBUG_WEBSOCKET) console.error('Failed to parse WebSocket message:', error, event.data);
            }
        };

        this.socket.onclose = (event) => {
            if (window.DEBUG_WEBSOCKET) console.log('WebSocket disconnected:', event.code, event.reason);
            this.isConnected = false;
            this.updateConnectionStatus(false);
            
            // Attempt to reconnect unless it was a clean close
            if (event.code !== 1000) {
                this.scheduleReconnect();
            }
        };

        this.socket.onerror = (error) => {
            if (window.DEBUG_WEBSOCKET) console.error('WebSocket error:', error);
            this.updateConnectionStatus(false);
        };
    }

    handleMessage(message) {
        if (window.DEBUG_WEBSOCKET) console.log('WebSocket message received:', message);
        
        const { type, data } = message;
        
        if (this.eventHandlers[type]) {
            this.eventHandlers[type](data);
        } else {
            console.warn('Unknown message type:', type);
        }
    }

    send(message) {
        if (this.isConnected && this.socket) {
            try {
                this.socket.send(JSON.stringify(message));
                return true;
            } catch (error) {
                console.error('Failed to send WebSocket message:', error);
                this.messageQueue.push(message);
                return false;
            }
        } else {
            console.warn('WebSocket not connected, queueing message');
            this.messageQueue.push(message);
            return false;
        }
    }

    authenticate() {
        // Prefer cookie-based authentication; send player_id only if available
        const playerId = localStorage.getItem('player_id') || sessionStorage.getItem('player_id');
        if (playerId) {
            this.send({ type: 'authenticate', data: { player_id: parseInt(playerId) } });
        }
    }

    subscribeToDefaultChannels() {
        const defaultChannels = ['processes', 'notifications', 'player', 'system'];
        this.subscribe(defaultChannels);
    }

    subscribe(channels) {
        if (!Array.isArray(channels)) {
            channels = [channels];
        }
        
        channels.forEach(channel => this.subscriptions.add(channel));
        
        this.send({
            type: 'subscribe',
            data: {
                channels: channels
            }
        });
    }

    unsubscribe(channels) {
        if (!Array.isArray(channels)) {
            channels = [channels];
        }
        
        channels.forEach(channel => this.subscriptions.delete(channel));
        
        this.send({
            type: 'unsubscribe',
            data: {
                channels: channels
            }
        });
    }

    processMessageQueue() {
        while (this.messageQueue.length > 0) {
            const message = this.messageQueue.shift();
            this.send(message);
        }
    }

    scheduleReconnect() {
        if (this.reconnectAttempts >= this.maxReconnectAttempts) {
            console.error('Max reconnection attempts reached');
            return;
        }
        
        this.reconnectAttempts++;
        const delay = this.reconnectDelay * Math.pow(2, this.reconnectAttempts - 1);
        
        if (window.DEBUG_WEBSOCKET) console.log(`Scheduling reconnection attempt ${this.reconnectAttempts} in ${delay}ms`);
        
        setTimeout(() => {
            this.connect();
        }, delay);
    }

    updateConnectionStatus(connected) {
        const statusEl = document.getElementById('connection-status');
        if (statusEl) {
            if (connected) {
                statusEl.textContent = 'Connected';
                statusEl.style.color = '#00ff00';
            } else {
                statusEl.textContent = 'Disconnected';
                statusEl.style.color = '#ff4444';
            }
        }
    }

    // Event Handlers

    handleProcessUpdate(data) {
        if (window.DEBUG_WEBSOCKET) console.log('Process update:', data);
        
        if (window.gameInstance) {
            // Update process in game instance
            const processIndex = window.gameInstance.processes.findIndex(p => p.id === data.process_id);
            
            if (processIndex !== -1) {
                if (data.status === 'completed' || data.status === 'failed') {
                    // Remove completed/failed process
                    window.gameInstance.processes.splice(processIndex, 1);
                } else {
                    // Update existing process
                    window.gameInstance.processes[processIndex] = { ...window.gameInstance.processes[processIndex], ...data };
                }
            } else if (data.status === 'started') {
                // Add new process
                window.gameInstance.processes.push(data);
            }
            
            // Update UI if on processes page
            if (window.gameInstance.currentPage === 'processes') {
                window.gameInstance.updateProcessList();
            }
        }
        
        // Show notification for completed processes
        if (data.status === 'completed') {
            this.showNotification('Process Completed', `${data.type} process completed successfully`, 'success');
        } else if (data.status === 'failed') {
            this.showNotification('Process Failed', `${data.type} process failed: ${data.failure_reason}`, 'error');
        }
    }

    handleServerUpdate(data) {
        if (window.DEBUG_WEBSOCKET) console.log('Server update:', data);
        
        // Handle server-related updates
        if (data.type === 'breach' && data.your_server) {
            this.showNotification('Security Alert', `Your server ${data.server_ip} has been breached!`, 'warning');
        }
    }

    handlePlayerUpdate(data) {
        if (window.DEBUG_WEBSOCKET) console.log('Player update:', data);
        
        if (window.gameInstance) {
            // Update player data
            window.gameInstance.player = { ...window.gameInstance.player, ...data };
            window.gameInstance.updatePlayerUI();
            
            // Show level up notification
            if (data.level_up) {
                this.showNotification('Level Up!', `Congratulations! You reached level ${data.level}`, 'success');
            }
            
            // Show money notification
            if (data.money_change) {
                const change = data.money_change;
                const type = change > 0 ? 'success' : 'info';
                const symbol = change > 0 ? '+' : '';
                this.showNotification('Money Update', `${symbol}$${change.toLocaleString()}`, type);
            }
        }
    }

    handleNotification(data) {
        if (window.DEBUG_WEBSOCKET) console.log('Notification:', data);
        
        this.showNotification(
            data.title || 'Notification',
            data.message || '',
            data.type || 'info'
        );
    }

    handleMailUpdate(data) {
        if (window.DEBUG_WEBSOCKET) console.log('Mail update:', data);
        
        if (data.new_mail) {
            const mailCount = document.getElementById('mail-count');
            if (mailCount) {
                mailCount.textContent = data.unread_count || 0;
            }
            
            this.showNotification('New Mail', `You have ${data.unread_count} unread messages`, 'info');
        }
    }

    handleClanUpdate(data) {
        if (window.DEBUG_WEBSOCKET) console.log('Clan update:', data);
        
        if (data.war_declared) {
            this.showNotification('Clan War', `War declared against ${data.target_clan}!`, 'warning');
        } else if (data.member_joined) {
            this.showNotification('Clan Update', `${data.username} joined the clan`, 'info');
        } else if (data.member_left) {
            this.showNotification('Clan Update', `${data.username} left the clan`, 'info');
        }
    }

    handleSystemMessage(data) {
        if (window.DEBUG_WEBSOCKET) console.log('System message:', data);
        
        this.showNotification(
            'System Message',
            data.message || '',
            data.level || 'info'
        );
        
        // Handle maintenance notifications
        if (data.maintenance) {
            this.showMaintenanceNotification(data);
        }
    }

    // Notification System

    showNotification(title, message, type = 'info') {
        const notification = this.createNotification(title, message, type);
        document.body.appendChild(notification);
        
        // Animate in
        setTimeout(() => {
            notification.classList.add('show');
        }, 100);
        
        // Auto-remove after 5 seconds
        setTimeout(() => {
            this.removeNotification(notification);
        }, 5000);
    }

    createNotification(title, message, type) {
        const notification = document.createElement('div');
        notification.className = `notification notification-${type}`;
        notification.innerHTML = `
            <div class="notification-content">
                <div class="notification-title">${title}</div>
                <div class="notification-message">${message}</div>
            </div>
            <button class="notification-close" onclick="this.parentElement.remove()">×</button>
        `;
        
        // Add CSS if not already present
        if (!document.querySelector('#notification-styles')) {
            const style = document.createElement('style');
            style.id = 'notification-styles';
            style.textContent = `
                .notification {
                    position: fixed;
                    top: 20px;
                    right: 20px;
                    background: #222;
                    border: 1px solid #333;
                    border-radius: 8px;
                    padding: 15px;
                    max-width: 350px;
                    z-index: 10001;
                    transform: translateX(100%);
                    transition: transform 0.3s ease;
                    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
                }
                
                .notification.show {
                    transform: translateX(0);
                }
                
                .notification-success {
                    border-color: #00ff00;
                    background: linear-gradient(135deg, #001100, #002200);
                }
                
                .notification-error {
                    border-color: #ff4444;
                    background: linear-gradient(135deg, #220000, #330000);
                }
                
                .notification-warning {
                    border-color: #ffaa00;
                    background: linear-gradient(135deg, #221100, #332200);
                }
                
                .notification-info {
                    border-color: #00aaff;
                    background: linear-gradient(135deg, #001122, #002233);
                }
                
                .notification-content {
                    margin-right: 30px;
                }
                
                .notification-title {
                    font-weight: bold;
                    margin-bottom: 5px;
                    color: #fff;
                }
                
                .notification-message {
                    color: #ccc;
                    font-size: 0.9em;
                }
                
                .notification-close {
                    position: absolute;
                    top: 10px;
                    right: 10px;
                    background: none;
                    border: none;
                    color: #888;
                    cursor: pointer;
                    font-size: 1.2em;
                    width: 20px;
                    height: 20px;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                }
                
                .notification-close:hover {
                    color: #fff;
                }
            `;
            document.head.appendChild(style);
        }
        
        return notification;
    }

    removeNotification(notification) {
        notification.classList.remove('show');
        setTimeout(() => {
            if (notification.parentElement) {
                notification.parentElement.removeChild(notification);
            }
        }, 300);
    }

    showMaintenanceNotification(data) {
        if (window.gameInstance) {
            window.gameInstance.showModal(
                'System Maintenance',
                `
                    <p>${data.message}</p>
                    ${data.scheduled_time ? `<p><strong>Scheduled:</strong> ${data.scheduled_time}</p>` : ''}
                    ${data.duration ? `<p><strong>Duration:</strong> ${data.duration}</p>` : ''}
                `,
                [
                    { text: 'OK', class: 'btn-primary', onclick: 'window.gameInstance.closeModal()' }
                ]
            );
        }
    }

    disconnect() {
        if (this.socket) {
            this.socket.close(1000, 'Client disconnecting');
            this.socket = null;
        }
        this.isConnected = false;
        this.updateConnectionStatus(false);
    }
}

// Create global instance
window.WebSocketManager = new WebSocketManager();

// Auto-connect when page loads if authenticated
document.addEventListener('DOMContentLoaded', () => {
    // Delay connection to allow game to initialize first
    setTimeout(() => {
        if (window.API && window.API.isAuthenticated()) {
            window.WebSocketManager.connect();
        } else {
            if (window.DEBUG_WEBSOCKET) console.log('Not connecting WebSocket - user not authenticated');
        }
    }, 1000);
});
