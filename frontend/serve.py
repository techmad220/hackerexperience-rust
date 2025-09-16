#!/usr/bin/env python3
"""
Frontend server with proper routing for HackerExperience
Handles both .html and non-.html URLs
"""

from http.server import HTTPServer, SimpleHTTPRequestHandler
import os
import json

class GameHTTPHandler(SimpleHTTPRequestHandler):
    def do_GET(self):
        # Remove query string for file lookup
        path = self.path.split('?')[0]

        # Handle special redirects
        if path == '/list':
            path = '/hacked_database.html'
        elif path == '/processes':
            path = '/task_manager.html'
        # API proxy for dynamic content
        elif path.startswith('/profile') or path.startswith('/news') or \
           path.startswith('/blog') or path.startswith('/clan') or \
           path.startswith('/internet') and '?' in path or \
           path == '/stats' or path == '/logout':
            # Redirect to API server
            self.send_response(302)
            self.send_header('Location', f'http://localhost:3005/api{self.path}')
            self.end_headers()
            return

        # Handle root
        if path == '/':
            path = '/index.html'
        # Add .html if no extension
        elif '.' not in os.path.basename(path) and not path.startswith('/css/') \
             and not path.startswith('/js/') and not path.startswith('/images/'):
            if os.path.exists(f'.{path}.html'):
                path = f'{path}.html'

        # Update the path
        self.path = path

        # Serve the file
        return SimpleHTTPRequestHandler.do_GET(self)

def run_server(port=8080):
    server_address = ('', port)
    httpd = HTTPServer(server_address, GameHTTPHandler)
    print(f'🎮 HackerExperience Frontend Server running on http://localhost:{port}')
    print('📡 API proxy enabled for dynamic content')
    httpd.serve_forever()

if __name__ == '__main__':
    os.chdir(os.path.dirname(os.path.abspath(__file__)))
    run_server()