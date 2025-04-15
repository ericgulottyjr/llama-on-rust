document.addEventListener('DOMContentLoaded', () => {
    const chatForm = document.getElementById('chat-form');
    const userInput = document.getElementById('user-input');
    const chatMessages = document.getElementById('chat-messages');
    
    // Session ID for tracking conversation
    let sessionId = null;
    
    // Fixed token count (for future slider implementation)
    //const DEFAULT_MAX_TOKENS = 1000;
    
    // Initialize the chat
    function initChat() {
        // Add a welcome message
        addBotMessage('Welcome to LLaMa Chat! How can I help you today?');
    }
    
    // Add a user message to the chat
    function addUserMessage(message) {
        const messageContainer = document.createElement('div');
        messageContainer.classList.add('message-container', 'user-container');
        
        const msgElement = document.createElement('div');
        msgElement.classList.add('message', 'user-message');
        msgElement.textContent = message;
        
        messageContainer.appendChild(msgElement);
        chatMessages.appendChild(messageContainer);
        scrollToBottom();
    }
    
    // Add a bot message to the chat
    function addBotMessage(message) {
        const messageContainer = document.createElement('div');
        messageContainer.classList.add('message-container', 'bot-container');
        
        const msgElement = document.createElement('div');
        msgElement.classList.add('message', 'bot-message');
        msgElement.textContent = message;
        
        messageContainer.appendChild(msgElement);
        chatMessages.appendChild(messageContainer);
        scrollToBottom();
    }
    
    // Add a loading indicator
    function addLoadingIndicator() {
        const messageContainer = document.createElement('div');
        messageContainer.classList.add('message-container', 'bot-container');
        messageContainer.id = 'loading-container';
        
        const loadingElement = document.createElement('div');
        loadingElement.classList.add('message', 'bot-message', 'loading');
        loadingElement.id = 'loading-indicator';
        
        loadingElement.innerHTML = `
            Thinking
            <div class="loading-dots">
                <span></span>
                <span></span>
                <span></span>
            </div>
        `;
        
        messageContainer.appendChild(loadingElement);
        chatMessages.appendChild(messageContainer);
        scrollToBottom();
    }
    
    // Remove loading indicator
    function removeLoadingIndicator() {
        const loadingContainer = document.getElementById('loading-container');
        if (loadingContainer) {
            loadingContainer.remove();
        }
    }
    
    // Scroll to the bottom of the chat
    function scrollToBottom() {
        chatMessages.scrollTop = chatMessages.scrollHeight;
    }
    
    // Send a message to the API
    async function sendMessage(message) {
        try {
            addLoadingIndicator();
            
            console.log(`Sending request to server`);
            
            const response = await fetch('/api/chat', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    message,
                    session_id: sessionId,
                })
            });
            
            if (!response.ok) {
                const errorText = await response.text();
                console.error('Error response:', errorText);
                throw new Error(`Server responded with status: ${response.status}`);
            }
            
            const data = await response.json();
            
            // Save the session ID
            sessionId = data.session_id;
            removeLoadingIndicator();
            addBotMessage(data.response);
        } catch (error) {
            removeLoadingIndicator();
            addBotMessage('Error: ' + error.message);
            console.error('Error:', error);
        }
    }
    
    // Handle form submission
    chatForm.addEventListener('submit', (e) => {
        e.preventDefault();
        
        const message = userInput.value.trim();
        if (message) {
            addUserMessage(message);
            userInput.value = '';
            sendMessage(message);
        }
    });
    
    // Initialize the chat
    initChat();
}); 