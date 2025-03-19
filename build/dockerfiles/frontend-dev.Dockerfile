FROM node:23 AS final

WORKDIR /app

RUN <<EOF
set -e
apt-get update
apt-get install -y wget default-jre-headless
EOF

CMD ["npm", "run", "dev"]