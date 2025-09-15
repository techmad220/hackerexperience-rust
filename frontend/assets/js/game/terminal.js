/**
 * HackerExperience Terminal Component
 * Realistic terminal interface with command execution
 * 
 * Features:
 * - Unix-like command system
 * - File system navigation
 * - Command history
 * - Tab completion
 * - Syntax highlighting
 * - Process execution
 */

class HackerExperienceTerminal {
    constructor(containerId, config = {}) {
        this.container = document.getElementById(containerId);
        if (!this.container) {
            throw new Error(`Terminal container not found: ${containerId}`);
        }

        this.config = {
            prompt: config.prompt || 'root@localhost:~#',
            maxHistory: config.maxHistory || 1000,
            maxOutput: config.maxOutput || 10000,
            enableColors: config.enableColors !== false,
            enableAutoComplete: config.enableAutoComplete !== false,
            currentPath: config.currentPath || '/home/user',
            currentServer: config.currentServer || 'localhost',
            ...config
        };

        this.commandHistory = [];
        this.historyIndex = -1;
        this.output = [];
        this.currentPath = this.config.currentPath;
        this.currentServer = this.config.currentServer;
        this.isConnected = true;
        this.currentProcess = null;

        // Command registry
        this.commands = new Map();
        this.aliases = new Map();

        this.init();
    }

    init() {
        this.createTerminalStructure();
        this.registerCommands();
        this.registerAliases();
        this.setupEventListeners();
        this.displayWelcomeMessage();
        this.focusInput();

        console.log('[Terminal] Initialized successfully');
    }

    createTerminalStructure() {
        this.container.innerHTML = `
            <div class="terminal-output" id="${this.container.id}-output">
                <!-- Terminal output will be added here -->
            </div>
            <div class="terminal-input-line">
                <span class="terminal-prompt" id="${this.container.id}-prompt">${this.getPrompt()}</span>
                <input type="text" 
                       class="terminal-input" 
                       id="${this.container.id}-input"
                       autocomplete="off" 
                       spellcheck="false"
                       autofocus>
            </div>
        `;

        this.outputElement = document.getElementById(`${this.container.id}-output`);
        this.promptElement = document.getElementById(`${this.container.id}-prompt`);
        this.inputElement = document.getElementById(`${this.container.id}-input`);
    }

    setupEventListeners() {
        this.inputElement.addEventListener('keydown', (e) => {
            switch (e.key) {
                case 'Enter':
                    e.preventDefault();
                    this.executeCommand();
                    break;
                case 'ArrowUp':
                    e.preventDefault();
                    this.navigateHistory(-1);
                    break;
                case 'ArrowDown':
                    e.preventDefault();
                    this.navigateHistory(1);
                    break;
                case 'Tab':
                    e.preventDefault();
                    if (this.config.enableAutoComplete) {
                        this.autoComplete();
                    }
                    break;
                case 'c':
                    if (e.ctrlKey) {
                        this.interrupt();
                    }
                    break;
                case 'l':
                    if (e.ctrlKey) {
                        e.preventDefault();
                        this.clear();
                    }
                    break;
            }
        });

        // Focus input when clicking on terminal
        this.container.addEventListener('click', () => {
            this.focusInput();
        });

        // Handle window resize
        window.addEventListener('resize', () => {
            this.scrollToBottom();
        });
    }

    registerCommands() {
        // File system commands
        this.registerCommand('ls', this.commandLs.bind(this), 'List directory contents');
        this.registerCommand('cd', this.commandCd.bind(this), 'Change directory');
        this.registerCommand('pwd', this.commandPwd.bind(this), 'Print working directory');
        this.registerCommand('mkdir', this.commandMkdir.bind(this), 'Create directory');
        this.registerCommand('rmdir', this.commandRmdir.bind(this), 'Remove directory');
        this.registerCommand('touch', this.commandTouch.bind(this), 'Create file');
        this.registerCommand('rm', this.commandRm.bind(this), 'Remove file');
        this.registerCommand('cat', this.commandCat.bind(this), 'Display file contents');
        this.registerCommand('nano', this.commandNano.bind(this), 'Edit file');
        this.registerCommand('chmod', this.commandChmod.bind(this), 'Change file permissions');

        // System commands
        this.registerCommand('ps', this.commandPs.bind(this), 'List running processes');
        this.registerCommand('kill', this.commandKill.bind(this), 'Terminate process');
        this.registerCommand('top', this.commandTop.bind(this), 'Display system processes');
        this.registerCommand('uptime', this.commandUptime.bind(this), 'Show system uptime');
        this.registerCommand('whoami', this.commandWhoami.bind(this), 'Display current user');
        this.registerCommand('date', this.commandDate.bind(this), 'Display current date/time');
        this.registerCommand('uname', this.commandUname.bind(this), 'System information');

        // Network commands
        this.registerCommand('ping', this.commandPing.bind(this), 'Ping a host');
        this.registerCommand('nmap', this.commandNmap.bind(this), 'Network scanner');
        this.registerCommand('ssh', this.commandSsh.bind(this), 'Secure shell connection');
        this.registerCommand('disconnect', this.commandDisconnect.bind(this), 'Disconnect from server');
        this.registerCommand('netstat', this.commandNetstat.bind(this), 'Network connections');

        // Hacking commands
        this.registerCommand('scan', this.commandScan.bind(this), 'Scan for vulnerabilities');
        this.registerCommand('exploit', this.commandExploit.bind(this), 'Execute exploit');
        this.registerCommand('upload', this.commandUpload.bind(this), 'Upload file to target');
        this.registerCommand('download', this.commandDownload.bind(this), 'Download file from target');
        this.registerCommand('crack', this.commandCrack.bind(this), 'Crack password/encryption');

        // Terminal commands
        this.registerCommand('help', this.commandHelp.bind(this), 'Display available commands');
        this.registerCommand('clear', this.commandClear.bind(this), 'Clear terminal screen');
        this.registerCommand('history', this.commandHistory.bind(this), 'Command history');
        this.registerCommand('exit', this.commandExit.bind(this), 'Exit terminal');
        this.registerCommand('man', this.commandMan.bind(this), 'Manual pages');

        // Game-specific commands
        this.registerCommand('tutorial', this.commandTutorial.bind(this), 'Start tutorial');
        this.registerCommand('stats', this.commandStats.bind(this), 'Display player statistics');
        this.registerCommand('mission', this.commandMission.bind(this), 'Mission management');
        this.registerCommand('bank', this.commandBank.bind(this), 'Banking operations');
    }

    registerAliases() {
        this.registerAlias('l', 'ls');
        this.registerAlias('ll', 'ls -la');
        this.registerAlias('la', 'ls -a');
        this.registerAlias('..', 'cd ..');
        this.registerAlias('?', 'help');
        this.registerAlias('cls', 'clear');
        this.registerAlias('dir', 'ls');
    }

    registerCommand(name, handler, description) {
        this.commands.set(name, {
            handler,
            description
        });
    }

    registerAlias(alias, command) {
        this.aliases.set(alias, command);
    }

    async executeCommand() {
        const input = this.inputElement.value.trim();
        if (!input) return;

        // Add to history
        this.addToHistory(input);

        // Display command in output
        this.addOutput(`${this.getPrompt()} ${input}`, 'command');

        // Clear input
        this.inputElement.value = '';
        this.historyIndex = -1;

        // Parse command
        const { command, args } = this.parseCommand(input);

        try {
            await this.runCommand(command, args);
        } catch (error) {
            this.addOutput(`Error: ${error.message}`, 'error');
        }

        this.scrollToBottom();
    }

    parseCommand(input) {
        // Handle aliases
        if (this.aliases.has(input.split(' ')[0])) {
            input = this.aliases.get(input.split(' ')[0]) + input.substring(input.indexOf(' '));
        }

        const parts = input.match(/(?:[^\s"']+|"[^"]*"|'[^']*')+/g) || [];
        const command = parts[0] || '';
        const args = parts.slice(1).map(arg => {
            // Remove quotes
            if ((arg.startsWith('"') && arg.endsWith('"')) || 
                (arg.startsWith("'") && arg.endsWith("'"))) {
                return arg.slice(1, -1);
            }
            return arg;
        });

        return { command, args };
    }

    async runCommand(command, args) {
        if (!command) return;

        const cmd = this.commands.get(command);
        if (cmd) {
            await cmd.handler(args);
        } else {
            this.addOutput(`Command not found: ${command}`, 'error');
            this.addOutput('Type "help" for available commands.', 'info');
        }
    }

    // Command implementations
    async commandLs(args) {
        const response = await window.API?.getFileSystem();
        if (response?.success) {
            const files = response.data.files || [];
            if (files.length === 0) {
                this.addOutput('Directory is empty.', 'info');
                return;
            }

            const hasLongFormat = args.includes('-l') || args.includes('-la');
            const showHidden = args.includes('-a') || args.includes('-la');

            let output = '';
            files.forEach(file => {
                if (!showHidden && file.name.startsWith('.')) return;

                if (hasLongFormat) {
                    const permissions = file.permissions || 'rw-r--r--';
                    const size = file.size || 0;
                    const modified = file.modified || new Date().toISOString();
                    const type = file.type === 'directory' ? 'd' : '-';
                    
                    output += `${type}${permissions} 1 user user ${size.toString().padStart(8)} ${new Date(modified).toLocaleDateString()} ${file.name}\n`;
                } else {
                    const color = file.type === 'directory' ? 'directory' : 'file';
                    output += `<span class="file-${color}">${file.name}</span>  `;
                }
            });

            this.addOutput(output.trim(), 'output');
        } else {
            this.addOutput('Failed to list directory contents.', 'error');
        }
    }

    async commandCd(args) {
        const path = args[0] || '/home/user';
        
        // Simulate directory change
        if (path === '..') {
            const parts = this.currentPath.split('/').filter(p => p);
            parts.pop();
            this.currentPath = '/' + parts.join('/') || '/';
        } else if (path.startsWith('/')) {
            this.currentPath = path;
        } else {
            this.currentPath = this.currentPath.endsWith('/') ? 
                this.currentPath + path : 
                this.currentPath + '/' + path;
        }

        this.updatePrompt();
    }

    commandPwd() {
        this.addOutput(this.currentPath, 'output');
    }

    async commandPs() {
        const response = await window.API?.getProcesses();
        if (response?.success) {
            const processes = response.data.processes || [];
            
            let output = 'PID    CMD                     STATUS      TIME\n';
            output += '----   ----------------------  ----------  --------\n';
            
            processes.forEach(proc => {
                const pid = proc.id.toString().padEnd(6);
                const cmd = (proc.action || 'unknown').padEnd(22);
                const status = (proc.status || 'running').padEnd(10);
                const time = this.formatDuration(proc.duration || 0).padEnd(8);
                output += `${pid} ${cmd} ${status} ${time}\n`;
            });

            this.addOutput(output, 'output');
        } else {
            this.addOutput('Failed to retrieve process list.', 'error');
        }
    }

    async commandNmap(args) {
        const target = args[0];
        if (!target) {
            this.addOutput('Usage: nmap <target>', 'error');
            return;
        }

        this.addOutput(`Starting nmap scan on ${target}...`, 'info');
        
        try {
            const response = await window.API?.scanNetwork(target);
            if (response?.success) {
                const results = response.data.results || [];
                
                this.addOutput(`\nNmap scan report for ${target}`, 'output');
                this.addOutput('PORT     STATE    SERVICE', 'output');
                
                results.forEach(result => {
                    const port = result.port.toString().padEnd(8);
                    const state = result.state.padEnd(8);
                    const service = result.service || 'unknown';
                    this.addOutput(`${port} ${state} ${service}`, 'output');
                });
            } else {
                this.addOutput('Scan failed: ' + (response?.message || 'Unknown error'), 'error');
            }
        } catch (error) {
            this.addOutput(`Scan failed: ${error.message}`, 'error');
        }
    }

    async commandSsh(args) {
        const target = args[0];
        if (!target) {
            this.addOutput('Usage: ssh <hostname/ip>', 'error');
            return;
        }

        this.addOutput(`Connecting to ${target}...`, 'info');
        
        try {
            const response = await window.API?.connectToServer(target);
            if (response?.success) {
                this.currentServer = target;
                this.currentPath = '/home/user';
                this.isConnected = true;
                this.updatePrompt();
                this.addOutput(`Connected to ${target}`, 'success');
            } else {
                this.addOutput('Connection failed: ' + (response?.message || 'Unknown error'), 'error');
            }
        } catch (error) {
            this.addOutput(`Connection failed: ${error.message}`, 'error');
        }
    }

    async commandScan(args) {
        const target = args[0] || this.currentServer;
        
        this.addOutput(`Scanning ${target} for vulnerabilities...`, 'info');
        this.showProgress('Scanning', 5000);
        
        try {
            const response = await window.API?.scanNetwork(target);
            if (response?.success) {
                const vulnerabilities = response.data.vulnerabilities || [];
                
                if (vulnerabilities.length === 0) {
                    this.addOutput('No vulnerabilities found.', 'info');
                } else {
                    this.addOutput(`Found ${vulnerabilities.length} vulnerabilities:`, 'warning');
                    vulnerabilities.forEach((vuln, index) => {
                        this.addOutput(`${index + 1}. ${vuln.name} (${vuln.severity})`, 'output');
                        this.addOutput(`   ${vuln.description}`, 'output');
                    });
                }
            } else {
                this.addOutput('Scan failed: ' + (response?.message || 'Unknown error'), 'error');
            }
        } catch (error) {
            this.addOutput(`Scan failed: ${error.message}`, 'error');
        }
    }

    commandHelp() {
        this.addOutput('Available commands:', 'info');
        this.addOutput('', 'output');
        
        const categories = {
            'File System': ['ls', 'cd', 'pwd', 'mkdir', 'rm', 'cat', 'touch'],
            'System': ['ps', 'kill', 'top', 'whoami', 'date', 'uptime'],
            'Network': ['ping', 'nmap', 'ssh', 'disconnect', 'netstat'],
            'Hacking': ['scan', 'exploit', 'upload', 'download', 'crack'],
            'Terminal': ['help', 'clear', 'history', 'exit', 'man']
        };

        Object.entries(categories).forEach(([category, commands]) => {
            this.addOutput(`${category}:`, 'category');
            commands.forEach(cmd => {
                const command = this.commands.get(cmd);
                if (command) {
                    this.addOutput(`  ${cmd.padEnd(12)} - ${command.description}`, 'output');
                }
            });
            this.addOutput('', 'output');
        });

        this.addOutput('Use "man <command>" for detailed help on a specific command.', 'info');
    }

    commandClear() {
        this.clear();
    }

    commandHistory() {
        this.addOutput('Command history:', 'info');
        this.commandHistory.forEach((cmd, index) => {
            this.addOutput(`${(index + 1).toString().padStart(4)}: ${cmd}`, 'output');
        });
    }

    commandWhoami() {
        const username = window.GAME_CONFIG?.user?.username || 'root';
        this.addOutput(username, 'output');
    }

    commandDate() {
        this.addOutput(new Date().toString(), 'output');
    }

    commandUptime() {
        const uptime = Math.floor(Math.random() * 100000) + 50000; // Simulated uptime
        const days = Math.floor(uptime / 86400);
        const hours = Math.floor((uptime % 86400) / 3600);
        const minutes = Math.floor((uptime % 3600) / 60);
        
        this.addOutput(`up ${days} days, ${hours}:${minutes.toString().padStart(2, '0')}`, 'output');
    }

    async commandStats() {
        const response = await window.API?.getPlayerStats();
        if (response?.success) {
            const stats = response.data;
            
            this.addOutput('Player Statistics:', 'info');
            this.addOutput(`Level: ${stats.level || 1}`, 'output');
            this.addOutput(`Experience: ${stats.experience || 0}`, 'output');
            this.addOutput(`Money: $${(stats.money || 0).toLocaleString()}`, 'output');
            this.addOutput(`Reputation: ${stats.reputation || 0}`, 'output');
            this.addOutput(`Successful hacks: ${stats.successful_hacks || 0}`, 'output');
            this.addOutput(`Failed hacks: ${stats.failed_hacks || 0}`, 'output');
        } else {
            this.addOutput('Failed to retrieve statistics.', 'error');
        }
    }

    commandTutorial() {
        this.addOutput('Starting interactive tutorial...', 'info');
        this.addOutput('', 'output');
        this.addOutput('Welcome to HackerExperience!', 'success');
        this.addOutput('This terminal is your main interface for interacting with the game.', 'output');
        this.addOutput('', 'output');
        this.addOutput('Basic commands to get started:', 'info');
        this.addOutput('  ls     - List files and directories', 'output');
        this.addOutput('  help   - Show all available commands', 'output');
        this.addOutput('  scan   - Scan for vulnerabilities', 'output');
        this.addOutput('  nmap   - Network scanner', 'output');
        this.addOutput('', 'output');
        this.addOutput('Try typing "ls" to see what\'s in your current directory!', 'info');
    }

    // Utility methods
    addOutput(text, type = 'output') {
        const outputLine = document.createElement('div');
        outputLine.className = `terminal-line terminal-${type}`;
        
        if (this.config.enableColors) {
            outputLine.innerHTML = this.colorizeText(text);
        } else {
            outputLine.textContent = text;
        }

        this.outputElement.appendChild(outputLine);
        this.output.push({ text, type, timestamp: Date.now() });

        // Limit output history
        if (this.output.length > this.config.maxOutput) {
            this.output.shift();
            this.outputElement.removeChild(this.outputElement.firstChild);
        }
    }

    colorizeText(text) {
        // Simple syntax highlighting
        return text
            .replace(/(\d+\.\d+\.\d+\.\d+)/g, '<span class="ip-address">$1</span>')
            .replace(/(\d+)/g, '<span class="number">$1</span>')
            .replace(/(\/[^\s]*)/g, '<span class="path">$1</span>')
            .replace(/(\w+@\w+)/g, '<span class="user-host">$1</span>');
    }

    addToHistory(command) {
        if (command && command !== this.commandHistory[this.commandHistory.length - 1]) {
            this.commandHistory.push(command);
            
            if (this.commandHistory.length > this.config.maxHistory) {
                this.commandHistory.shift();
            }
        }
    }

    navigateHistory(direction) {
        if (this.commandHistory.length === 0) return;

        this.historyIndex += direction;
        
        if (this.historyIndex < 0) {
            this.historyIndex = 0;
        } else if (this.historyIndex >= this.commandHistory.length) {
            this.historyIndex = this.commandHistory.length;
            this.inputElement.value = '';
            return;
        }

        this.inputElement.value = this.commandHistory[this.historyIndex] || '';
    }

    autoComplete() {
        const input = this.inputElement.value;
        const parts = input.split(' ');
        const lastPart = parts[parts.length - 1];

        if (parts.length === 1) {
            // Command completion
            const matches = Array.from(this.commands.keys())
                .filter(cmd => cmd.startsWith(lastPart))
                .sort();

            if (matches.length === 1) {
                this.inputElement.value = matches[0] + ' ';
            } else if (matches.length > 1) {
                this.addOutput('Available commands:', 'info');
                this.addOutput(matches.join('  '), 'output');
            }
        } else {
            // File/path completion (simplified)
            const commonPaths = ['/home/user', '/etc', '/var/log', '/tmp'];
            const matches = commonPaths.filter(path => path.startsWith(lastPart));
            
            if (matches.length === 1) {
                parts[parts.length - 1] = matches[0];
                this.inputElement.value = parts.join(' ') + ' ';
            }
        }
    }

    showProgress(action, duration) {
        const progressLine = document.createElement('div');
        progressLine.className = 'terminal-line terminal-progress';
        progressLine.innerHTML = `
            <span>${action}: </span>
            <div class="progress-bar">
                <div class="progress-fill"></div>
            </div>
        `;
        
        this.outputElement.appendChild(progressLine);
        
        const progressFill = progressLine.querySelector('.progress-fill');
        let progress = 0;
        const interval = setInterval(() => {
            progress += 100 / (duration / 100);
            progressFill.style.width = `${Math.min(progress, 100)}%`;
            
            if (progress >= 100) {
                clearInterval(interval);
                setTimeout(() => {
                    progressLine.remove();
                }, 500);
            }
        }, 100);
    }

    getPrompt() {
        const user = window.GAME_CONFIG?.user?.username || 'root';
        const path = this.currentPath.replace(/^\/home\/\w+/, '~');
        return `${user}@${this.currentServer}:${path}#`;
    }

    updatePrompt() {
        this.promptElement.textContent = this.getPrompt();
    }

    clear() {
        this.outputElement.innerHTML = '';
        this.output = [];
    }

    focusInput() {
        this.inputElement.focus();
    }

    scrollToBottom() {
        this.container.scrollTop = this.container.scrollHeight;
    }

    interrupt() {
        if (this.currentProcess) {
            this.addOutput('^C', 'command');
            this.addOutput('Process interrupted.', 'warning');
            this.currentProcess = null;
        }
        this.inputElement.value = '';
    }

    displayWelcomeMessage() {
        const welcome = [
            'HackerExperience Terminal v2.0.0',
            `Connected to ${this.currentServer}`,
            `Welcome back, ${window.GAME_CONFIG?.user?.username || 'Player'}!`,
            '',
            'Type "help" for available commands or "tutorial" for a quick start guide.',
            'Type "stats" to view your current statistics.',
            ''
        ];

        welcome.forEach(line => {
            this.addOutput(line, line.includes('Welcome') ? 'success' : 'info');
        });
    }

    formatDuration(seconds) {
        if (seconds < 60) return `${seconds}s`;
        if (seconds < 3600) return `${Math.floor(seconds / 60)}m`;
        return `${Math.floor(seconds / 3600)}h`;
    }

    // Public API
    executeCustomCommand(command) {
        this.inputElement.value = command;
        this.executeCommand();
    }

    addCustomCommand(name, handler, description) {
        this.registerCommand(name, handler, description);
    }

    getCommandHistory() {
        return [...this.commandHistory];
    }

    getCurrentPath() {
        return this.currentPath;
    }

    getCurrentServer() {
        return this.currentServer;
    }

    isServerConnected() {
        return this.isConnected;
    }

    destroy() {
        this.container.innerHTML = '';
        this.commands.clear();
        this.aliases.clear();
        this.commandHistory = [];
        this.output = [];
    }
}

// Global terminal instance
if (typeof window !== 'undefined') {
    window.HackerExperienceTerminal = HackerExperienceTerminal;
}