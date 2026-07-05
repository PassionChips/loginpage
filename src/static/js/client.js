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
    const messageEl = document.getElementById('client-message');
    messageEl.textContent = message;
    messageEl.className = 'message ' + (isError ? 'error' : 'success');
    messageEl.style.display = 'block';
    
    // Scroll to top to show message
    window.scrollTo({ top: 0, behavior: 'smooth' });
}

/**
 * Validate GST number
 * @param {string} gst - GST number to validate
 * @returns {boolean} - Whether GST is valid
 */
function validateGST(gst) {
    const gstRegex = /^[0-9]{2}[A-Z]{5}[0-9]{4}[A-Z]{1}[1-9A-Z]{1}Z[0-9A-Z]{1}$/;
    return gstRegex.test(gst);
}

/**
 * Validate PAN number
 * @param {string} pan - PAN number to validate
 * @returns {boolean} - Whether PAN is valid
 */
function validatePAN(pan) {
    const panRegex = /^[A-Z]{5}[0-9]{4}[A-Z]{1}$/;
    return panRegex.test(pan);
}

/**
 * Validate phone number
 * @param {string} phone - Phone number to validate
 * @returns {boolean} - Whether phone is valid
 */
function validatePhone(phone) {
    const cleaned = phone.replace(/[\s-]/g, '');
    const phoneRegex = /^(\+91)?[6-9]\d{9}$/;
    return phoneRegex.test(cleaned);
}

/**
 * Validate email
 * @param {string} email - Email to validate
 * @returns {boolean} - Whether email is valid
 */
function validateEmail(email) {
    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    return emailRegex.test(email);
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
    const state = document.getElementById('business-state').value;
    const isTamilNadu = state === 'Tamil Nadu';
    const tamilNaduCompliance = document.getElementById('accept-tamil-nadu').checked;
    
    return {
        company_name: document.getElementById('company-name').value.trim(),
        business_type: document.getElementById('business-type').value,
        gst_number: document.getElementById('gst-number').value.trim().toUpperCase(),
        pan_number: document.getElementById('pan-number').value.trim().toUpperCase(),
        business_phone: document.getElementById('business-phone').value.trim(),
        business_email: document.getElementById('business-email').value.trim(),
        address_line1: document.getElementById('business-address-line1').value.trim(),
        address_line2: document.getElementById('business-address-line2').value.trim() || null,
        city: document.getElementById('business-city').value.trim(),
        state: state,
        pincode: document.getElementById('business-pincode').value.trim(),
        country: document.getElementById('business-country').value.trim(),
        is_tamil_nadu: isTamilNadu,
        tamil_nadu_compliance: isTamilNadu ? tamilNaduCompliance : null,
        terms_accepted: document.getElementById('accept-terms').checked,
        terms_accepted_at: null,
        verification_status: 'pending'
    };
}

/**
 * Validate form data
 * @param {object} data - Form data to validate
 * @returns {string|null} - Error message or null if valid
 */
function validateFormData(data) {
    // Business Information
    if (!data.company_name) {
        return 'Company name is required';
    }
    
    if (!data.business_type) {
        return 'Please select a business type';
    }
    
    if (!data.gst_number) {
        return 'GST number is required';
    }
    
    if (!validateGST(data.gst_number)) {
        return 'Please enter a valid GST number (e.g., 22AAAAA0000A1Z5)';
    }
    
    if (!data.pan_number) {
        return 'PAN number is required';
    }
    
    if (!validatePAN(data.pan_number)) {
        return 'Please enter a valid PAN number (e.g., ABCDE1234F)';
    }
    
    if (!data.business_phone) {
        return 'Business phone is required';
    }
    
    if (!validatePhone(data.business_phone)) {
        return 'Please enter a valid business phone number';
    }
    
    if (!data.business_email) {
        return 'Business email is required';
    }
    
    if (!validateEmail(data.business_email)) {
        return 'Please enter a valid business email';
    }
    
    // Address
    if (!data.address_line1) {
        return 'Address Line 1 is required';
    }
    
    if (!data.city) {
        return 'City is required';
    }
    
    if (!data.state) {
        return 'Please select a state';
    }
    
    if (!data.pincode) {
        return 'Pincode is required';
    }
    
    if (!validatePincode(data.pincode)) {
        return 'Please enter a valid 6-digit pincode';
    }
    
    // Terms
    if (!data.terms_accepted) {
        return 'You must accept the terms and conditions to continue';
    }
    
    // Tamil Nadu specific check
    if (data.state === 'Tamil Nadu' && !data.tamil_nadu_compliance) {
        return 'You must confirm Tamil Nadu compliance requirements';
    }
    
    return null;
}

/**
 * Submit client information
 */
async function submitClientInfo() {
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
        const response = await fetch(`${CONFIG.API_BASE_URL}/profile/clientprofileextraction`, {
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
            showMessage('Registration completed successfully! Your account is under review. Redirecting...', false);
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

/**
 * Handle state selection change
 */
function handleStateChange() {
    const stateSelect = document.getElementById('business-state');
    const tamilNaduWarning = document.getElementById('tamil-nadu-warning');
    const tamilNaduCheckbox = document.getElementById('tamil-nadu-checkbox');
    const tamilNaduCheckboxInput = document.getElementById('accept-tamil-nadu');
    
    if (stateSelect.value === 'Tamil Nadu') {
        tamilNaduWarning.style.display = 'block';
        tamilNaduCheckbox.style.display = 'flex';
        tamilNaduCheckboxInput.required = true;
    } else {
        tamilNaduWarning.style.display = 'none';
        tamilNaduCheckbox.style.display = 'none';
        tamilNaduCheckboxInput.required = false;
        tamilNaduCheckboxInput.checked = false;
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
    if (selectedRole !== 'client') {
        console.warn('Invalid role or no role selected');
        // Optionally redirect back to role selection
        // window.location.href = '/role-selection';
    }
    
    // Add state change listener
    const stateSelect = document.getElementById('business-state');
    stateSelect.addEventListener('change', handleStateChange);
    
    // Add form validation on input
    const form = document.getElementById('client-form');
    const inputs = form.querySelectorAll('input[required], select[required]');
    
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
    
    // GST number formatting
    const gstInput = document.getElementById('gst-number');
    gstInput.addEventListener('input', function(e) {
        e.target.value = e.target.value.toUpperCase().replace(/[^A-Z0-9]/g, '').slice(0, 15);
    });
    
    // PAN number formatting
    const panInput = document.getElementById('pan-number');
    panInput.addEventListener('input', function(e) {
        e.target.value = e.target.value.toUpperCase().replace(/[^A-Z0-9]/g, '').slice(0, 10);
    });
    
    // Phone number formatting
    const phoneInput = document.getElementById('business-phone');
    phoneInput.addEventListener('input', function(e) {
        let value = e.target.value;
        if (value.startsWith('+')) {
            value = '+' + value.slice(1).replace(/\D/g, '');
        } else {
            value = value.replace(/\D/g, '');
        }
        e.target.value = value;
    });
    
    // Pincode validation
    const pincodeInput = document.getElementById('business-pincode');
    pincodeInput.addEventListener('input', function(e) {
        e.target.value = e.target.value.replace(/\D/g, '').slice(0, 6);
    });
    
});

// Made with Bob
