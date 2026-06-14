// Configuration
const CONFIG = {
    API_BASE_URL: 'http://localhost:3000',
    REDIRECT_DELAY: 2000
};

// Cached DOM elements
let signupElements = null;
let loginElements = null;

function clearMessages() {
    const loginMsg = document.getElementById('login-message');
    const signupMsg = document.getElementById('signup-message');
    if (loginMsg) loginMsg.style.display = 'none';
    if (signupMsg) signupMsg.style.display = 'none';
}

function showMessage(elementId, message, isError) {
    const messageEl = document.getElementById(elementId);
    messageEl.textContent = message;
    messageEl.className = 'message ' + (isError ? 'error' : 'success');
    messageEl.style.display = 'block';
}

function getLoginElements() {
    if (!loginElements) {
        loginElements = {
            username: document.getElementById("login-username"),
            password: document.getElementById("login-password")
        };
    }
    return loginElements;
}

function getSignupElements() {
    if (!signupElements) {
        signupElements = {
            firstName: document.getElementById("signup-first-name"),
            lastName: document.getElementById("signup-last-name"),
            username: document.getElementById("signup-username"),
            email: document.getElementById("signup-email"),
            password: document.getElementById("signup-password"),
            confirmPassword: document.getElementById("signup-confirm-password")
        };
    }
    return signupElements;
}

async function apiRequest(endpoint, data) {
    const response = await fetch(`${CONFIG.API_BASE_URL}${endpoint}`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(data)
    });
    
    if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
    }
    
    return response.json();
}

async function login_btn() {
    const elements = getLoginElements();
    
    if (!elements.username.value || !elements.password.value) {
        showMessage('login-message', 'Please fill in all fields', true);
        return;
    }

    const data = {
        username: elements.username.value,
        password: elements.password.value
    };
    
    try {
        const result = await apiRequest('/login', data);
        
        if (result.result === true) {
            location.href = `${CONFIG.API_BASE_URL}/success`;
        } else {
            location.href = `${CONFIG.API_BASE_URL}/error`;
        }
    } catch (error) {
        console.error('Login error:', error);
        showMessage('login-message', 'Login failed. Please try again.', true);
    }
}

function validatePassword(password) {
    // Check length (more than 6 characters)
    if (password.length <= 6) {
        return 'Password must be more than 6 characters';
    }
    
    // Check for uppercase letter
    if (!/[A-Z]/.test(password)) {
        return 'Password must contain at least one uppercase letter';
    }
    
    // Check for lowercase letter
    if (!/[a-z]/.test(password)) {
        return 'Password must contain at least one lowercase letter';
    }
    
    // Check for number
    if (!/[0-9]/.test(password)) {
        return 'Password must contain at least one number';
    }
    
    return null; // Password is valid
}

function validateEmail(email) {
    if (!email) return 'Please enter a valid email address'; // Optional field
    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    return emailRegex.test(email) ? null : 'Please enter a valid email address';
}

function validateSignupForm(firstName, lastName, username, email, password, confirmPassword) {
    if (!firstName || !lastName || !username || !email || !password || !confirmPassword) {
        return 'Please fill in all required fields';
    }
    
    if (password !== confirmPassword) {
        return 'Passwords do not match';
    }
    
    const passwordError = validatePassword(password);
    if (passwordError) return passwordError;
    
    const emailError = validateEmail(email);
    if (emailError) return emailError;
    
    return null;
}

async function signup_btn() {
    const elements = getSignupElements();
    
    // Validation
    const validationError = validateSignupForm(
        elements.firstName.value,
        elements.lastName.value,
        elements.username.value,
        elements.email.value,
        elements.password.value,
        elements.confirmPassword.value
    );
    
    if (validationError) {
        showMessage('signup-message', validationError, true);
        return;
    }
    
    const data = {
        first_name: elements.firstName.value,
        last_name: elements.lastName.value,
        username: elements.username.value,
        password: elements.password.value,
        email: elements.email.value || null
    };
    
    try {
        const result = await apiRequest('/signup', data);
        
        if (result.result === true) {
            showMessage('signup-message', 'Account created successfully! Redirecting to login...', false);
            setTimeout(() => {
                window.location.href = '/login';
            }, CONFIG.REDIRECT_DELAY);
        } else {
            showMessage('signup-message', result.message || 'Sign up failed. Username or Email may already exist.', true);
        }
    } catch (error) {
        console.error('Signup error:', error);
        showMessage('signup-message', 'Sign up failed. Please try again.', true);
    }
}

// Add Enter key support
document.addEventListener('DOMContentLoaded', function() {
    const loginPassword = document.getElementById('login-password');
    const signupConfirmPassword = document.getElementById('signup-confirm-password');
    
    if (loginPassword) {
        loginPassword.addEventListener('keypress', function(e) {
            if (e.key === 'Enter') login_btn();
        });
    }
    
    if (signupConfirmPassword) {
        signupConfirmPassword.addEventListener('keypress', function(e) {
            if (e.key === 'Enter') signup_btn();
        });
    }
});
