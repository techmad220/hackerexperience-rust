/**
 * HackerExperience - Interactive Onboarding Tutorial
 * Guides new users through game mechanics step by step
 */

class OnboardingTutorial {
    constructor() {
        this.currentStep = 0;
        this.completed = localStorage.getItem('tutorial_completed') === 'true';
        this.steps = [
            {
                title: "Welcome to HackerExperience!",
                content: "You're about to enter the world of hacking. This tutorial will guide you through the basics.",
                element: null,
                position: 'center',
                action: null
            },
            {
                title: "Your Dashboard",
                content: "This is your main dashboard. Here you can see your stats, hardware, and recent activity.",
                element: '.dashboard',
                position: 'bottom',
                highlight: true
            },
            {
                title: "Hardware Management",
                content: "Your hardware determines your hacking capabilities. Better hardware means faster processes.",
                element: '.hardware-panel',
                position: 'right',
                highlight: true
            },
            {
                title: "Software Library",
                content: "Install and manage your hacking tools here. Different software serves different purposes.",
                element: '.software-panel',
                position: 'left',
                highlight: true
            },
            {
                title: "Internet Browser",
                content: "Explore the internet to find servers to hack. Each server has different security levels.",
                element: '.internet-link',
                position: 'bottom',
                action: () => this.showInternetDemo()
            },
            {
                title: "Starting a Hack",
                content: "Select a target server and choose your attack method. Watch your processes in real-time.",
                element: '.process-panel',
                position: 'top',
                action: () => this.showHackingDemo()
            },
            {
                title: "Missions",
                content: "Complete missions to earn money and experience. They guide you through game progression.",
                element: '.missions-link',
                position: 'bottom',
                highlight: true
            },
            {
                title: "Clans",
                content: "Join or create a clan to collaborate with other hackers and dominate the rankings.",
                element: '.clan-link',
                position: 'bottom',
                highlight: true
            },
            {
                title: "Ready to Begin!",
                content: "You now know the basics. Start with the tutorial mission to practice your skills!",
                element: null,
                position: 'center',
                action: () => this.completeTutorial()
            }
        ];
    }

    start() {
        if (this.completed && !confirm("You've already completed the tutorial. Do you want to see it again?")) {
            return;
        }

        this.currentStep = 0;
        this.createOverlay();
        this.showStep(0);
    }

    createOverlay() {
        // Create tutorial overlay
        const overlay = document.createElement('div');
        overlay.id = 'tutorial-overlay';
        overlay.className = 'tutorial-overlay';
        overlay.innerHTML = `
            <div class="tutorial-backdrop"></div>
            <div class="tutorial-tooltip" role="dialog" aria-label="Tutorial step">
                <div class="tutorial-header">
                    <h3 class="tutorial-title"></h3>
                    <button class="tutorial-close" aria-label="Close tutorial">&times;</button>
                </div>
                <div class="tutorial-content"></div>
                <div class="tutorial-footer">
                    <div class="tutorial-progress">
                        <span class="step-counter"></span>
                        <div class="progress-bar">
                            <div class="progress-fill"></div>
                        </div>
                    </div>
                    <div class="tutorial-actions">
                        <button class="tutorial-prev" aria-label="Previous step">Previous</button>
                        <button class="tutorial-next" aria-label="Next step">Next</button>
                        <button class="tutorial-skip" aria-label="Skip tutorial">Skip Tutorial</button>
                    </div>
                </div>
            </div>
        `;

        document.body.appendChild(overlay);

        // Add event listeners
        overlay.querySelector('.tutorial-close').addEventListener('click', () => this.close());
        overlay.querySelector('.tutorial-prev').addEventListener('click', () => this.previousStep());
        overlay.querySelector('.tutorial-next').addEventListener('click', () => this.nextStep());
        overlay.querySelector('.tutorial-skip').addEventListener('click', () => this.skip());

        // Add styles
        this.addStyles();
    }

    showStep(stepIndex) {
        const step = this.steps[stepIndex];
        const overlay = document.getElementById('tutorial-overlay');

        if (!overlay) return;

        // Update content
        overlay.querySelector('.tutorial-title').textContent = step.title;
        overlay.querySelector('.tutorial-content').innerHTML = step.content;
        overlay.querySelector('.step-counter').textContent = `Step ${stepIndex + 1} of ${this.steps.length}`;

        // Update progress bar
        const progress = ((stepIndex + 1) / this.steps.length) * 100;
        overlay.querySelector('.progress-fill').style.width = `${progress}%`;

        // Update button states
        overlay.querySelector('.tutorial-prev').disabled = stepIndex === 0;
        overlay.querySelector('.tutorial-next').textContent =
            stepIndex === this.steps.length - 1 ? 'Finish' : 'Next';

        // Position tooltip
        this.positionTooltip(step);

        // Highlight element if specified
        if (step.highlight && step.element) {
            this.highlightElement(step.element);
        }

        // Execute step action if specified
        if (step.action) {
            step.action();
        }

        // Focus management for accessibility
        overlay.querySelector('.tutorial-title').focus();
    }

    positionTooltip(step) {
        const tooltip = document.querySelector('.tutorial-tooltip');

        if (!step.element || step.position === 'center') {
            // Center the tooltip
            tooltip.style.top = '50%';
            tooltip.style.left = '50%';
            tooltip.style.transform = 'translate(-50%, -50%)';
        } else {
            const element = document.querySelector(step.element);
            if (element) {
                const rect = element.getBoundingClientRect();

                switch(step.position) {
                    case 'top':
                        tooltip.style.top = `${rect.top - tooltip.offsetHeight - 10}px`;
                        tooltip.style.left = `${rect.left + rect.width / 2}px`;
                        tooltip.style.transform = 'translateX(-50%)';
                        break;
                    case 'bottom':
                        tooltip.style.top = `${rect.bottom + 10}px`;
                        tooltip.style.left = `${rect.left + rect.width / 2}px`;
                        tooltip.style.transform = 'translateX(-50%)';
                        break;
                    case 'left':
                        tooltip.style.top = `${rect.top + rect.height / 2}px`;
                        tooltip.style.left = `${rect.left - tooltip.offsetWidth - 10}px`;
                        tooltip.style.transform = 'translateY(-50%)';
                        break;
                    case 'right':
                        tooltip.style.top = `${rect.top + rect.height / 2}px`;
                        tooltip.style.left = `${rect.right + 10}px`;
                        tooltip.style.transform = 'translateY(-50%)';
                        break;
                }
            }
        }
    }

    highlightElement(selector) {
        // Remove previous highlights
        document.querySelectorAll('.tutorial-highlight').forEach(el => {
            el.classList.remove('tutorial-highlight');
        });

        // Add new highlight
        const element = document.querySelector(selector);
        if (element) {
            element.classList.add('tutorial-highlight');
            element.scrollIntoView({ behavior: 'smooth', block: 'center' });
        }
    }

    nextStep() {
        if (this.currentStep < this.steps.length - 1) {
            this.currentStep++;
            this.showStep(this.currentStep);
        } else {
            this.completeTutorial();
        }
    }

    previousStep() {
        if (this.currentStep > 0) {
            this.currentStep--;
            this.showStep(this.currentStep);
        }
    }

    skip() {
        if (confirm("Are you sure you want to skip the tutorial? You can restart it anytime from the help menu.")) {
            this.close();
        }
    }

    close() {
        const overlay = document.getElementById('tutorial-overlay');
        if (overlay) {
            overlay.remove();
        }

        // Remove highlights
        document.querySelectorAll('.tutorial-highlight').forEach(el => {
            el.classList.remove('tutorial-highlight');
        });
    }

    completeTutorial() {
        localStorage.setItem('tutorial_completed', 'true');
        this.completed = true;

        // Show completion message
        alert("🎉 Tutorial completed! You're ready to start hacking!");

        this.close();

        // Start first mission
        if (window.startFirstMission) {
            window.startFirstMission();
        }
    }

    showInternetDemo() {
        // Demo of internet browsing
        console.log("Showing internet demo...");
    }

    showHackingDemo() {
        // Demo of hacking process
        console.log("Showing hacking demo...");
    }

    addStyles() {
        if (document.getElementById('tutorial-styles')) return;

        const styles = document.createElement('style');
        styles.id = 'tutorial-styles';
        styles.textContent = `
            .tutorial-overlay {
                position: fixed;
                top: 0;
                left: 0;
                right: 0;
                bottom: 0;
                z-index: 10000;
            }

            .tutorial-backdrop {
                position: absolute;
                top: 0;
                left: 0;
                right: 0;
                bottom: 0;
                background: rgba(0, 0, 0, 0.8);
            }

            .tutorial-tooltip {
                position: absolute;
                background: #0a0a0a;
                border: 2px solid #00ff00;
                color: #00ff00;
                padding: 20px;
                border-radius: 5px;
                max-width: 400px;
                box-shadow: 0 0 20px rgba(0, 255, 0, 0.5);
            }

            .tutorial-header {
                display: flex;
                justify-content: space-between;
                align-items: center;
                margin-bottom: 15px;
            }

            .tutorial-title {
                margin: 0;
                font-size: 1.2em;
                color: #00ff00;
            }

            .tutorial-close {
                background: transparent;
                border: 1px solid #00ff00;
                color: #00ff00;
                cursor: pointer;
                padding: 5px 10px;
                font-size: 20px;
            }

            .tutorial-content {
                margin-bottom: 20px;
                line-height: 1.6;
            }

            .tutorial-footer {
                border-top: 1px solid #00ff00;
                padding-top: 15px;
            }

            .tutorial-progress {
                margin-bottom: 15px;
            }

            .step-counter {
                display: block;
                margin-bottom: 5px;
                font-size: 0.9em;
            }

            .progress-bar {
                width: 100%;
                height: 5px;
                background: #003300;
                border: 1px solid #00ff00;
                border-radius: 3px;
                overflow: hidden;
            }

            .progress-fill {
                height: 100%;
                background: #00ff00;
                transition: width 0.3s ease;
            }

            .tutorial-actions {
                display: flex;
                justify-content: space-between;
                gap: 10px;
            }

            .tutorial-actions button {
                background: #0a0a0a;
                border: 1px solid #00ff00;
                color: #00ff00;
                padding: 8px 15px;
                cursor: pointer;
                flex: 1;
            }

            .tutorial-actions button:hover:not(:disabled) {
                background: #00ff00;
                color: #0a0a0a;
            }

            .tutorial-actions button:disabled {
                opacity: 0.5;
                cursor: not-allowed;
            }

            .tutorial-highlight {
                position: relative;
                z-index: 9999;
                box-shadow: 0 0 30px rgba(0, 255, 0, 0.8);
                animation: pulse 2s infinite;
            }

            @keyframes pulse {
                0% { box-shadow: 0 0 30px rgba(0, 255, 0, 0.8); }
                50% { box-shadow: 0 0 50px rgba(0, 255, 0, 1); }
                100% { box-shadow: 0 0 30px rgba(0, 255, 0, 0.8); }
            }

            @media (max-width: 576px) {
                .tutorial-tooltip {
                    max-width: 90%;
                    margin: 0 5%;
                }

                .tutorial-actions {
                    flex-direction: column;
                }
            }
        `;

        document.head.appendChild(styles);
    }
}

// Help System
class HelpSystem {
    constructor() {
        this.topics = {
            'getting-started': {
                title: 'Getting Started',
                content: `
                    <h4>Welcome to HackerExperience!</h4>
                    <p>Start your journey as a hacker by:</p>
                    <ol>
                        <li>Upgrading your hardware for better performance</li>
                        <li>Installing essential software tools</li>
                        <li>Exploring the internet for targets</li>
                        <li>Completing missions to earn money and XP</li>
                    </ol>
                `
            },
            'hardware': {
                title: 'Hardware Guide',
                content: `
                    <h4>Hardware Components</h4>
                    <ul>
                        <li><strong>CPU:</strong> Determines processing speed</li>
                        <li><strong>RAM:</strong> Allows multiple concurrent processes</li>
                        <li><strong>HDD:</strong> Storage for software and files</li>
                        <li><strong>Network:</strong> Upload/download speeds</li>
                    </ul>
                    <p>Better hardware = faster hacking!</p>
                `
            },
            'hacking': {
                title: 'Hacking Guide',
                content: `
                    <h4>How to Hack</h4>
                    <ol>
                        <li>Find a target server using Internet</li>
                        <li>Scan the server to check security</li>
                        <li>Use appropriate software (cracker, exploit)</li>
                        <li>Wait for the process to complete</li>
                        <li>Hide your logs to avoid detection</li>
                    </ol>
                `
            },
            'software': {
                title: 'Software Types',
                content: `
                    <h4>Essential Software</h4>
                    <ul>
                        <li><strong>Cracker:</strong> Break passwords</li>
                        <li><strong>Firewall:</strong> Protect your system</li>
                        <li><strong>Hidder:</strong> Hide your activities</li>
                        <li><strong>Seeker:</strong> Find hidden logs</li>
                        <li><strong>Antivirus:</strong> Remove viruses</li>
                        <li><strong>Exploit:</strong> Advanced hacking</li>
                    </ul>
                `
            },
            'missions': {
                title: 'Missions',
                content: `
                    <h4>Mission Types</h4>
                    <ul>
                        <li><strong>Tutorial:</strong> Learn the basics</li>
                        <li><strong>Hack:</strong> Break into systems</li>
                        <li><strong>Steal:</strong> Download files</li>
                        <li><strong>Delete:</strong> Remove evidence</li>
                        <li><strong>DDoS:</strong> Attack servers</li>
                    </ul>
                    <p>Complete missions to progress and unlock new content!</p>
                `
            },
            'shortcuts': {
                title: 'Keyboard Shortcuts',
                content: `
                    <h4>Quick Actions</h4>
                    <ul>
                        <li><kbd>H</kbd> - Home/Dashboard</li>
                        <li><kbd>I</kbd> - Internet</li>
                        <li><kbd>P</kbd> - Processes</li>
                        <li><kbd>S</kbd> - Software</li>
                        <li><kbd>M</kbd> - Missions</li>
                        <li><kbd>?</kbd> - Help</li>
                        <li><kbd>ESC</kbd> - Close dialogs</li>
                    </ul>
                `
            }
        };
    }

    show(topic = null) {
        this.createHelpDialog(topic);
    }

    createHelpDialog(topic) {
        // Remove existing dialog
        const existing = document.getElementById('help-dialog');
        if (existing) existing.remove();

        const dialog = document.createElement('div');
        dialog.id = 'help-dialog';
        dialog.className = 'help-dialog';
        dialog.innerHTML = `
            <div class="help-dialog-content" role="dialog" aria-label="Help">
                <div class="help-header">
                    <h2>Help & Documentation</h2>
                    <button class="help-close" aria-label="Close help">&times;</button>
                </div>
                <div class="help-body">
                    <div class="help-sidebar">
                        <h3>Topics</h3>
                        <ul class="help-topics">
                            ${Object.keys(this.topics).map(key => `
                                <li><button data-topic="${key}">${this.topics[key].title}</button></li>
                            `).join('')}
                        </ul>
                        <button class="restart-tutorial">Restart Tutorial</button>
                    </div>
                    <div class="help-content">
                        ${topic ? this.topics[topic].content : this.topics['getting-started'].content}
                    </div>
                </div>
            </div>
        `;

        document.body.appendChild(dialog);

        // Add event listeners
        dialog.querySelector('.help-close').addEventListener('click', () => dialog.remove());

        dialog.querySelectorAll('[data-topic]').forEach(button => {
            button.addEventListener('click', (e) => {
                const topicKey = e.target.dataset.topic;
                dialog.querySelector('.help-content').innerHTML = this.topics[topicKey].content;
            });
        });

        dialog.querySelector('.restart-tutorial').addEventListener('click', () => {
            dialog.remove();
            window.tutorial.start();
        });

        this.addHelpStyles();
    }

    addHelpStyles() {
        if (document.getElementById('help-styles')) return;

        const styles = document.createElement('style');
        styles.id = 'help-styles';
        styles.textContent = `
            .help-dialog {
                position: fixed;
                top: 0;
                left: 0;
                right: 0;
                bottom: 0;
                background: rgba(0, 0, 0, 0.9);
                z-index: 9999;
                display: flex;
                align-items: center;
                justify-content: center;
            }

            .help-dialog-content {
                background: #0a0a0a;
                border: 2px solid #00ff00;
                color: #00ff00;
                width: 90%;
                max-width: 800px;
                max-height: 80vh;
                overflow: hidden;
                display: flex;
                flex-direction: column;
            }

            .help-header {
                display: flex;
                justify-content: space-between;
                align-items: center;
                padding: 15px 20px;
                border-bottom: 1px solid #00ff00;
            }

            .help-close {
                background: transparent;
                border: 1px solid #00ff00;
                color: #00ff00;
                cursor: pointer;
                padding: 5px 10px;
                font-size: 20px;
            }

            .help-body {
                display: flex;
                flex: 1;
                overflow: hidden;
            }

            .help-sidebar {
                width: 200px;
                border-right: 1px solid #00ff00;
                padding: 20px;
                overflow-y: auto;
            }

            .help-topics {
                list-style: none;
                padding: 0;
                margin: 10px 0 20px 0;
            }

            .help-topics button {
                width: 100%;
                background: transparent;
                border: 1px solid #003300;
                color: #00ff00;
                padding: 10px;
                cursor: pointer;
                text-align: left;
                margin-bottom: 5px;
            }

            .help-topics button:hover {
                background: #003300;
            }

            .restart-tutorial {
                width: 100%;
                background: #003300;
                border: 1px solid #00ff00;
                color: #00ff00;
                padding: 10px;
                cursor: pointer;
            }

            .help-content {
                flex: 1;
                padding: 20px;
                overflow-y: auto;
            }

            .help-content h4 {
                color: #00ff00;
                margin-top: 0;
            }

            .help-content ul, .help-content ol {
                line-height: 1.8;
            }

            kbd {
                background: #003300;
                padding: 2px 5px;
                border: 1px solid #00ff00;
                border-radius: 3px;
            }

            @media (max-width: 576px) {
                .help-body {
                    flex-direction: column;
                }

                .help-sidebar {
                    width: 100%;
                    border-right: none;
                    border-bottom: 1px solid #00ff00;
                }
            }
        `;

        document.head.appendChild(styles);
    }
}

// Initialize on page load
window.addEventListener('DOMContentLoaded', () => {
    window.tutorial = new OnboardingTutorial();
    window.helpSystem = new HelpSystem();

    // Check if first time user
    if (!localStorage.getItem('tutorial_completed')) {
        setTimeout(() => {
            window.tutorial.start();
        }, 1000);
    }

    // Add help button
    const helpButton = document.createElement('button');
    helpButton.className = 'help-button';
    helpButton.innerHTML = '?';
    helpButton.title = 'Help (Press ?)';
    helpButton.addEventListener('click', () => window.helpSystem.show());
    document.body.appendChild(helpButton);

    // Keyboard shortcuts
    document.addEventListener('keydown', (e) => {
        if (e.key === '?') {
            window.helpSystem.show();
        }
    });
});