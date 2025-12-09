# Nexus-UMMID Demo UI
# Lightweight static file server using nginx

FROM nginx:alpine

# Remove default nginx config
RUN rm /etc/nginx/conf.d/default.conf

# Add custom nginx config for SPA
COPY nginx.conf /etc/nginx/conf.d/

# Copy static files
COPY index.html /usr/share/nginx/html/
COPY llms.txt /usr/share/nginx/html/

# Copy ARW manifest (Agent Ready Web)
COPY .well-known /usr/share/nginx/html/.well-known

# Expose port 8080 for Cloud Run
EXPOSE 8080

# Start nginx
CMD ["nginx", "-g", "daemon off;"]
