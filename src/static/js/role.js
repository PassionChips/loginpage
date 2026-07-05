// Configuration
const CONFIG = {
    API_BASE_URL: 'http://localhost:3000'
};

/**
 * Handle role selection
 * @param {string} role - Either 'customer' or 'client'
 */
function selectRole(role) {
    // Store the selected role in sessionStorage for the next page
    sessionStorage.setItem('selectedRole', role);
    
    // Redirect to appropriate information form
    if (role === 'customer') {
        window.location.href = '/customer-info';
    } else if (role === 'client') {
        window.location.href = '/client-info';
    } else {
        console.error('Invalid role selected:', role);
    }
}

// Check if user is authenticated (this should be called on page load)
document.addEventListener('DOMContentLoaded', function() {
    // TODO: Add authentication check here
    // For now, we'll assume the user is authenticated if they reached this page
    console.log('Role selection page loaded');
});

// Made with Bob
