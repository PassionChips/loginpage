// Configuration
const CONFIG = {
    API_BASE_URL: 'http://localhost:3000',
    REDIRECT_DELAY: 2000
};

/**
 * Show message to user
 * @param {string} message - Message to display
 * @param {boolean} isError - Whether it's an error message
 */
function showMessage(message, isError) {
    const messageEl = document.getElementById('customer-message');
    messageEl.textContent = message;
    messageEl.className = 'message ' + (isError ? 'error' : 'success');
    messageEl.style.display = 'block';
    
    // Scroll to top to show message
    window.scrollTo({ top: 0, behavior: 'smooth' });
}

/**
 * Validate phone number
 * @param {string} phone - Phone number to validate
 * @returns {boolean} - Whether phone is valid
 */
function validatePhone(phone) {
    // Remove spaces and dashes
    const cleaned = phone.replace(/[\s-]/g, '');
    // Check if it's a valid Indian phone number (10 digits) with optional +91
    const phoneRegex = /^(\+91)?[6-9]\d{9}$/;
    return phoneRegex.test(cleaned);
}

/**
 * Validate pincode
 * @param {string} pincode - Pincode to validate
 * @returns {boolean} - Whether pincode is valid
 */
function validatePincode(pincode) {
    return /^\d{6}$/.test(pincode);
}

/**
 * Get form data
 * @returns {object} - Form data object
 */
function getFormData() {
    const termsAccepted = document.getElementById('accept-terms').checked;
    
    return {
        phone: document.getElementById('phone').value.trim(),
        alternate_phone: document.getElementById('alternate-phone').value.trim() || null,
        address_line1: document.getElementById('address-line1').value.trim(),
        address_line2: document.getElementById('address-line2').value.trim() || null,
        city: document.getElementById('city').value.trim(),
        state: document.getElementById('state').value.trim(),
        pincode: document.getElementById('pincode').value.trim(),
        country: document.getElementById('country').value.trim(),
        terms_accepted: termsAccepted,
        terms_accepted_at: null
    };
}

/**
 * Validate form data
 * @param {object} data - Form data to validate
 * @returns {string|null} - Error message or null if valid
 */
function validateFormData(data) {
    if (!data.phone) {
        return 'Phone number is required';
    }
    
    if (!validatePhone(data.phone)) {
        return 'Please enter a valid Indian phone number';
    }
    
    if (data.alternate_phone && !validatePhone(data.alternate_phone)) {
        return 'Please enter a valid alternate phone number';
    }
    
    if (!data.address_line1) {
        return 'Address Line 1 is required';
    }
    
    if (!data.city) {
        return 'City is required';
    }
    
    if (!data.state) {
        return 'State is required';
    }
    
    if (!data.pincode) {
        return 'Pincode is required';
    }
    
    if (!validatePincode(data.pincode)) {
        return 'Please enter a valid 6-digit pincode';
    }
    
    if (!data.country) {
        return 'Country is required';
    }
    
    if (!data.terms_accepted) {
        return 'You must accept the terms and conditions to continue';
    }
    
    return null;
}

/**
 * Submit customer information
 */
async function submitCustomerInfo() {
    const submitBtn = document.getElementById('submit-btn');
    
    // Get user_id from sessionStorage
    const userId = sessionStorage.getItem('userId');
    if (!userId) {
        showMessage('Session expired. Please login again.', true);
        setTimeout(() => window.location.href = '/login', 2000);
        return;
    }
    
    // Get form data
    const formData = getFormData();
    
    // Validate form data
    const validationError = validateFormData(formData);
    if (validationError) {
        showMessage(validationError, true);
        return;
    }
    
    // Add user_id to form data (send only user_id, not username)
    const requestData = {
        user_id: parseInt(userId),
        ...formData
    };
    
    // Disable submit button
    submitBtn.disabled = true;
    submitBtn.textContent = 'Submitting...';
    
    try {
        // Profile creation endpoint
        const response = await fetch(`${CONFIG.API_BASE_URL}/profile/customerprofilecreation`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify(requestData)
        });
        
        if (!response.ok) {
            throw new Error(`HTTP error! status: ${response.status}`);
        }
        
        const result = await response.json();
        
        if (result.result === true) {
            showMessage('Registration completed successfully! Redirecting...', false);
            setTimeout(() => {
                window.location.href = '/success';
            }, CONFIG.REDIRECT_DELAY);
        } else {
            showMessage(result.message || 'Registration failed. Please try again.', true);
            submitBtn.disabled = false;
            submitBtn.textContent = 'Complete Registration';
        }
    } catch (error) {
        console.error('Registration error:', error);
        showMessage('An error occurred. Please try again later.', true);
        submitBtn.disabled = false;
        submitBtn.textContent = 'Complete Registration';
    }
}

// Initialize page
document.addEventListener('DOMContentLoaded', function() {
    // Auto-fill user ID and username from sessionStorage
    const userId = sessionStorage.getItem('userId');
    const username = sessionStorage.getItem('username');
    
    const userIdInput = document.getElementById('user-id');
    const usernameInput = document.getElementById('display-username');
    
    if (userId && username) {
        userIdInput.value = userId;
        usernameInput.value = username;
    } else {
        userIdInput.value = 'Error: Session expired';
        usernameInput.value = 'Error: Session expired';
        showMessage('Session expired. Please login again.', true);
        setTimeout(() => window.location.href = '/login', 2000);
        return;
    }
    
    // Check if role was selected
    const selectedRole = sessionStorage.getItem('role');
    if (selectedRole !== 'customer') {
        console.warn('Invalid role or no role selected');
        // Optionally redirect back to role selection
        // window.location.href = '/role-selection';
    }
    
    // Add form validation on input
    const form = document.getElementById('customer-form');
    const inputs = form.querySelectorAll('input[required]');
    
    inputs.forEach(input => {
        input.addEventListener('blur', function() {
            if (!this.value.trim()) {
                this.style.borderColor = 'var(--error-color)';
            } else {
                this.style.borderColor = 'var(--border-color)';
            }
        });
        
        input.addEventListener('input', function() {
            if (this.style.borderColor === 'rgb(244, 67, 54)') {
                this.style.borderColor = 'var(--border-color)';
            }
        });
    });
    
    // Phone number formatting
    const phoneInputs = [
        document.getElementById('phone'),
        document.getElementById('alternate-phone')
    ];
    
    phoneInputs.forEach(input => {
        if (input) {
            input.addEventListener('input', function(e) {
                // Remove non-numeric characters except + at the start
                let value = e.target.value;
                if (value.startsWith('+')) {
                    value = '+' + value.slice(1).replace(/\D/g, '');
                } else {
                    value = value.replace(/\D/g, '');
                }
                e.target.value = value;
            });
        }
    });
    
    // Pincode validation
    const pincodeInput = document.getElementById('pincode');
    pincodeInput.addEventListener('input', function(e) {
        e.target.value = e.target.value.replace(/\D/g, '').slice(0, 6);
    });
});

// Made with Bob
