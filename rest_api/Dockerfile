FROM node:16.17.1-alpine
LABEL description="Rest Api of Microservice Trip with Node"

# Set working directory
WORKDIR ./rest-api/

COPY src/ ./
RUN npm install

CMD [ "node", "server.js" ]


