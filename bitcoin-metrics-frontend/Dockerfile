# Use an official Node.js image as the base image
FROM node:16-alpine

# Set the working directory inside the container
WORKDIR /usr/src/app

# Copy package.json and package-lock.json to install dependencies
COPY package*.json ./

# Install the dependencies
RUN npm install

# Copy the rest of the application code to the container
COPY . .

# Build the React app (optional if using npm start)
RUN npm run build

# Expose the port the app will run on
EXPOSE 3000

# Command to run the app
CMD ["npm", "start"]
