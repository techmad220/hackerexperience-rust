#!/bin/bash

# SSL Certificate Generation Script for HackerExperience
# This script generates self-signed certificates for development
# For production, use Let's Encrypt or commercial certificates

set -e

DOMAIN=${1:-hackerexperience.local}
DAYS=365
SSL_DIR="/etc/nginx/ssl"

echo "========================================="
echo "SSL Certificate Generator"
echo "========================================="

# Create SSL directory
mkdir -p $SSL_DIR

# Generate private key
openssl genrsa -out $SSL_DIR/private.key 4096

# Generate certificate signing request
openssl req -new -key $SSL_DIR/private.key \
    -out $SSL_DIR/certificate.csr \
    -subj "/C=US/ST=State/L=City/O=HackerExperience/CN=$DOMAIN"

# Generate self-signed certificate
openssl x509 -req -days $DAYS \
    -in $SSL_DIR/certificate.csr \
    -signkey $SSL_DIR/private.key \
    -out $SSL_DIR/certificate.crt

# Generate Diffie-Hellman parameters for extra security
openssl dhparam -out $SSL_DIR/dhparam.pem 2048

# Set proper permissions
chmod 600 $SSL_DIR/private.key
chmod 644 $SSL_DIR/certificate.crt
chmod 644 $SSL_DIR/dhparam.pem

echo ""
echo "✅ SSL certificates generated successfully!"
echo ""
echo "Files created:"
echo "  - Private Key: $SSL_DIR/private.key"
echo "  - Certificate: $SSL_DIR/certificate.crt"
echo "  - DH Params: $SSL_DIR/dhparam.pem"
echo ""
echo "For production, use Let's Encrypt:"
echo "  certbot --nginx -d $DOMAIN"