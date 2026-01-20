// Biblio - E-book Library Browser
class BiblioApp {
    constructor() {
        this.libraries = [];
        this.currentLibraryId = null;
        this.currentLibrary = null;
        this.allBooks = [];
        this.filteredBooks = [];
        this.authors = [];
        this.tags = [];
        this.series = [];
        this.selectedBook = null;
        this.selectedBookId = null;
        this.selectedBookLibraryId = null;
        this.selectedAuthors = new Set();
        this.selectedTags = new Set();
        this.selectedSeries = new Set();
        this.selectedFormats = new Set();
        this.sortMethod = 'recent';
        this.searchTerm = '';
        this.booksPerPage = 100;
        this.displayedBooksCount = 0;
        this.isLoadingMore = false;
        this.isAuthenticated = false;
        this.currentUsername = null;

        // Splitter drag state
        this.resizing = null;

        // View mode and table settings
        this.currentViewMode = 'grid';
        this.coverSize = 120;
        this.tableColumnWidths = {
            title: 30,
            authors: 20,
            series: 15,
            publisher: 15,
            rating: 10,
            pubdate: 10
        };
        this.columnVisibility = {
            title: true,
            authors: true,
            series: true,
            publisher: true,
            rating: true,
            pubdate: true
        };

        // Track which library each view was rendered for
        this.gridRenderedForLibrary = null;
        this.tableRenderedForLibrary = null;
    }

    // Cookie Management Methods
    setCookie(name, value, days = 30) {
        const d = new Date();
        d.setTime(d.getTime() + (days * 24 * 60 * 60 * 1000));
        const expires = "expires=" + d.toUTCString();
        document.cookie = name + "=" + encodeURIComponent(JSON.stringify(value)) + ";" + expires + ";path=/";
    }

    getCookie(name) {
        const nameEQ = name + "=";
        const cookies = document.cookie.split(';');
        for (let cookie of cookies) {
            cookie = cookie.trim();
            if (cookie.indexOf(nameEQ) === 0) {
                try {
                    return JSON.parse(decodeURIComponent(cookie.substring(nameEQ.length)));
                } catch (e) {
                    return null;
                }
            }
        }
        return null;
    }

    saveAppState() {
        const state = {
            currentLibraryId: this.currentLibraryId,
            selectedAuthors: Array.from(this.selectedAuthors),
            selectedTags: Array.from(this.selectedTags),
            selectedSeries: Array.from(this.selectedSeries),
            selectedFormats: Array.from(this.selectedFormats),
            searchTerm: this.searchTerm,
            sortMethod: this.sortMethod,
            displayedBooksCount: this.displayedBooksCount,
            selectedBookId: this.selectedBookId,
            selectedBookLibraryId: this.selectedBookLibraryId
        };
        this.setCookie('biblioAppState', state);
    }

    loadAppState() {
        const state = this.getCookie('biblioAppState');
        if (state) {
            this.currentLibraryId = state.currentLibraryId;
            this.selectedAuthors = new Set((state.selectedAuthors || []).map(id => parseInt(id)));
            this.selectedTags = new Set((state.selectedTags || []).map(id => parseInt(id)));
            this.selectedSeries = new Set((state.selectedSeries || []).map(id => parseInt(id)));
            this.selectedFormats = new Set(state.selectedFormats || []);
            this.searchTerm = state.searchTerm || '';
            this.sortMethod = state.sortMethod || 'recent';
            this.displayedBooksCount = state.displayedBooksCount || 0;
            this.selectedBookId = state.selectedBookId || null;
            this.selectedBookLibraryId = state.selectedBookLibraryId || null;
            return state;
        }
        return null;
    }

    restoreFilterUI() {
        // Restore search input
        const searchInput = document.getElementById('searchInput');
        if (searchInput) {
            searchInput.value = this.searchTerm;
        }

        // Restore author checkboxes
        this.selectedAuthors.forEach(authorId => {
            const checkbox = document.getElementById(`author_${authorId}`);
            if (checkbox) {
                checkbox.checked = true;
            }
        });

        // Restore tag checkboxes
        this.selectedTags.forEach(tagId => {
            const checkbox = document.getElementById(`tag_${tagId}`);
            if (checkbox) {
                checkbox.checked = true;
            }
        });

        // Restore series checkboxes
        this.selectedSeries.forEach(seriesId => {
            const checkbox = document.getElementById(`series_${seriesId}`);
            if (checkbox) {
                checkbox.checked = true;
            }
        });

        // Restore format checkboxes
        this.selectedFormats.forEach(format => {
            const checkbox = document.getElementById(`format_${format}`);
            if (checkbox) {
                checkbox.checked = true;
            }
        });

        // Update count displays
        this.updateAuthorCount();
        this.updateFormatCount();
        this.updateTagCount();
        this.updateSeriesCount();
    }

    async init() {
        console.log('Initializing Biblio App...');

        // Load view preferences
        this.loadViewPreferences();

        // Check if user is authenticated
        const authState = this.loadAuthState();
        if (authState && authState.isAuthenticated) {
            this.isAuthenticated = true;
            this.currentUsername = authState.username;
            this.showMainApp();
            this.checkAdminStatus();

            // Load saved app state from cookies
            const savedState = this.loadAppState();

            await this.loadLibraries();
            this.setupEventListeners();

            // Restore filter section collapsed/expanded states
            this.restoreFilterSectionStates();

            // If there was a saved library, select it; otherwise select first library
            if (savedState && savedState.currentLibraryId && this.libraries.some(lib => lib.id === savedState.currentLibraryId)) {
                await this.selectLibrary(savedState.currentLibraryId);
            } else if (this.libraries.length > 0) {
                await this.selectLibrary(this.libraries[0].id);
            }

            // After library is loaded, restore the selected book if it exists
            if (this.selectedBookId && this.selectedBookLibraryId) {
                await this.restoreSelectedBook();
            }

            // Update view display with saved preferences
            this.updateViewDisplay();
        } else {
            this.showLoginPage();
        }
    }

    checkAdminStatus() {
        // Check if user has admin role from localStorage
        const auth = this.loadAuthState();
        const adminBtn = document.getElementById('adminBtn');
        if (!adminBtn) return;

        if (auth && auth.role === 'admin') {
            adminBtn.style.display = 'inline-block';
        } else {
            adminBtn.style.display = 'none';
        }
    }

    // Authentication Methods
    saveAuthState(username, role = 'reader') {
        const authState = {
            isAuthenticated: true,
            username: username,
            role: role,
            timestamp: Date.now()
        };
        localStorage.setItem('biblio_auth', JSON.stringify(authState));
    }

    loadAuthState() {
        try {
            const authState = localStorage.getItem('biblio_auth');
            return authState ? JSON.parse(authState) : null;
        } catch (error) {
            console.error('Error loading auth state:', error);
            this.clearAuthState();
            return null;
        }
    }

    clearAuthState() {
        localStorage.removeItem('biblio_auth');
    }

    showLoginPage() {
        try {
            const contentArea = document.querySelector('.content-area');
            const topPanel = document.querySelector('.top-panel');
            const bottomPanel = document.querySelector('.bottom-panel');

            // Hide the main content and panels
            contentArea.style.display = 'none';
            if (bottomPanel) bottomPanel.style.display = 'none';

            // Create login container
            const loginContainer = document.createElement('div');
            loginContainer.id = 'loginContainer';
            loginContainer.style.cssText = `
                display: flex;
                align-items: center;
                justify-content: center;
                width: 100%;
                height: 100%;
                background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            `;

            loginContainer.innerHTML = `
                <div style="
                    background: white;
                    padding: 40px;
                    border-radius: 8px;
                    box-shadow: 0 10px 25px rgba(0,0,0,0.2);
                    width: 100%;
                    max-width: 400px;
                    text-align: center;
                ">
                    <h1 style="color: #2c3e50; margin-bottom: 10px; font-size: 28px;">Biblio</h1>
                    <p style="color: #95a5a6; margin-bottom: 30px; font-size: 14px;">E-book Library Browser</p>
                    
                    <form id="loginForm" style="text-align: left;" autocomplete="off">
                        <div style="margin-bottom: 20px;">
                            <label style="display: block; margin-bottom: 8px; color: #2c3e50; font-weight: 500;">Username</label>
                            <input 
                                type="text" 
                                id="usernameInput" 
                                placeholder="Enter username"
                                autocomplete="username"
                                style="
                                    width: 100%;
                                    padding: 10px;
                                    border: 1px solid #ecf0f1;
                                    border-radius: 4px;
                                    font-size: 14px;
                                    box-sizing: border-box;
                                "
                            />
                        </div>
                        
                        <div style="margin-bottom: 30px;">
                            <label style="display: block; margin-bottom: 8px; color: #2c3e50; font-weight: 500;">Password</label>
                            <input 
                                type="password" 
                                id="passwordInput" 
                                placeholder="Enter password"
                                autocomplete="current-password"
                                style="
                                    width: 100%;
                                    padding: 10px;
                                    border: 1px solid #ecf0f1;
                                    border-radius: 4px;
                                    font-size: 14px;
                                    box-sizing: border-box;
                                "
                            />
                        </div>
                        
                        <button 
                            type="submit"
                            style="
                                width: 100%;
                                padding: 12px;
                                background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                                color: white;
                                border: none;
                                border-radius: 4px;
                                font-size: 16px;
                                font-weight: 500;
                                cursor: pointer;
                                transition: opacity 0.3s;
                            "
                        >
                            Login
                        </button>
                    </form>
                    
                    <div id="loginError" style="
                        margin-top: 20px;
                        padding: 12px;
                        background-color: #fadbd8;
                        color: #c0392b;
                        border-radius: 4px;
                        display: none;
                        font-size: 14px;
                    "></div>
                </div>
            `;

            document.body.appendChild(loginContainer);

            // Clear password field for security (override browser autofill)
            const passwordField = document.getElementById('passwordInput');
            const usernameField = document.getElementById('usernameInput');

            // Aggressive clearing to prevent browser autofill
            if (passwordField) {
                // Add input event listener to clear field if autofilled
                passwordField.addEventListener('input', () => {
                    // Don't prevent input, just monitor
                });

                // Set readonly briefly to prevent autofill, then remove
                passwordField.setAttribute('readonly', '');
                passwordField.value = '';

                // Remove readonly after browser autofill attempt would have happened
                setTimeout(() => {
                    if (passwordField) {
                        passwordField.removeAttribute('readonly');
                        passwordField.value = '';
                    }
                }, 50);

                // Additional clearing attempts
                setTimeout(() => {
                    if (passwordField) {
                        passwordField.value = '';
                    }
                }, 100);

                setTimeout(() => {
                    if (passwordField) {
                        passwordField.value = '';
                    }
                }, 200);
            }

            // Also clear username field
            if (usernameField) {
                usernameField.value = '';
            }

            // Hide buttons in top panel
            const buttons = topPanel.querySelectorAll('button');
            buttons.forEach(btn => btn.style.display = 'none');

            // Setup login form handler
            const loginForm = document.getElementById('loginForm');
            if (loginForm) {
                loginForm.addEventListener('submit', (e) => this.handleLogin(e));
            }
        } catch (error) {
            console.error('Error showing login page:', error);
        }
    }

    async handleLogin(e) {
        e.preventDefault();

        const username = document.getElementById('usernameInput').value;
        const password = document.getElementById('passwordInput').value;
        const errorDiv = document.getElementById('loginError');

        if (!username || !password) {
            errorDiv.textContent = 'Please enter username and password';
            errorDiv.style.display = 'block';
            return;
        }

        try {
            const response = await fetch('/api/auth/login', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({ username, password })
            });

            const data = await response.json();

            if (data.success) {
                this.isAuthenticated = true;
                this.currentUsername = username;
                // Extract role from login response, default to 'reader' if not provided
                const role = data.data && data.data.role ? data.data.role : 'reader';
                this.saveAuthState(username, role);

                // Remove login page
                const loginContainer = document.getElementById('loginContainer');
                loginContainer.remove();

                // Show main app
                this.showMainApp();

                // Check admin status AFTER auth state is saved
                this.checkAdminStatus();

                // Initialize app
                const savedState = this.loadAppState();
                await this.loadLibraries();
                this.setupEventListeners();

                if (savedState && savedState.currentLibraryId && this.libraries.some(lib => lib.id === savedState.currentLibraryId)) {
                    await this.selectLibrary(savedState.currentLibraryId);
                } else if (this.libraries.length > 0) {
                    await this.selectLibrary(this.libraries[0].id);
                }
            } else {
                errorDiv.textContent = data.error || 'Login failed';
                errorDiv.style.display = 'block';
                // Clear password field on failed login for security
                document.getElementById('passwordInput').value = '';
            }
        } catch (error) {
            console.error('Login error:', error);
            errorDiv.textContent = 'An error occurred during login';
            errorDiv.style.display = 'block';
        }
    }

    showMainApp() {
        const contentArea = document.querySelector('.content-area');
        const topPanel = document.querySelector('.top-panel');
        const bottomPanel = document.querySelector('.bottom-panel');

        contentArea.style.display = 'flex';
        if (bottomPanel) bottomPanel.style.display = 'flex';

        // Show/update buttons in top panel (except admin button, which is handled by checkAdminStatus)
        const buttons = topPanel.querySelectorAll('button');
        buttons.forEach(btn => {
            // Don't show admin button here; let checkAdminStatus handle it
            if (btn.id === 'adminBtn') {
                btn.style.display = 'none';
            } else {
                btn.style.display = 'inline-block';
            }
        });

        // Add user info and logout button
        const userInfo = topPanel.querySelector('#userInfo');
        if (!userInfo) {
            const userInfoDiv = document.createElement('div');
            userInfoDiv.id = 'userInfo';
            userInfoDiv.style.cssText = `
                margin-left: 20px;
                display: flex;
                align-items: center;
                gap: 10px;
                font-size: 14px;
            `;
            userInfoDiv.innerHTML = `
                <span><strong>${this.currentUsername}</strong></span>
                <button id="logoutBtn" style="
                    background-color: #e74c3c;
                    padding: 6px 12px;
                    font-size: 13px;
                ">Logout</button>
            `;
            topPanel.appendChild(userInfoDiv);

            document.getElementById('logoutBtn').addEventListener('click', () => this.handleLogout());
        }
    }

    async handleLogout() {
        try {
            await fetch('/api/auth/logout', {
                method: 'POST'
            });
        } catch (error) {
            console.error('Logout error:', error);
        }

        this.isAuthenticated = false;
        this.currentUsername = null;
        this.clearAuthState();

        // Hide admin button
        const adminBtn = document.getElementById('adminBtn');
        if (adminBtn) {
            adminBtn.style.display = 'none';
        }

        // Remove user info
        const userInfo = document.getElementById('userInfo');
        if (userInfo) userInfo.remove();

        // Hide main content and show login page
        document.querySelector('.content-area').style.display = 'none';
        this.showLoginPage();
    }

    async loadLibraries() {
        try {
            this.updateStatus('Loading libraries...');
            const response = await fetch('/api/libraries');
            const data = await response.json();

            if (data.success) {
                this.libraries = data.data || [];
                this.renderLibraries();
            } else {
                console.error('Failed to load libraries:', data.error);
                this.updateStatus('Failed to load libraries');
            }
        } catch (error) {
            console.error('Error loading libraries:', error);
            this.updateStatus('Error loading libraries');
        }
    }

    renderLibraries() {
        const librariesList = document.getElementById('librariesList');
        librariesList.innerHTML = '';

        if (this.libraries.length === 0) {
            librariesList.innerHTML = '<div style="padding: 10px; color: #95a5a6; font-size: 12px;">No libraries found. Add Calibre libraries to the libraries folder.</div>';
            return;
        }

        this.libraries.forEach(lib => {
            const item = document.createElement('div');
            item.className = 'filter-item';
            item.innerHTML = `
                <input type="radio" name="library" id="lib_${lib.id}" value="${lib.id}" 
                       onchange="app.selectLibrary('${lib.id}')">
                <label for="lib_${lib.id}">${lib.name} (${lib.book_count})</label>
            `;
            librariesList.appendChild(item);
        });
    }

    async selectLibrary(libraryId) {
        try {
            this.currentLibraryId = libraryId;
            this.selectedBook = null;
            this.displayedBooksCount = 0;

            // Check if we're switching to a different library
            const isSwitchingLibrary = this.currentLibrary && this.currentLibrary.id !== libraryId;

            // If switching libraries, clear all filters
            if (isSwitchingLibrary) {
                this.selectedAuthors.clear();
                this.selectedTags.clear();
                this.selectedSeries.clear();
                this.selectedFormats.clear();
                this.searchTerm = '';
                this.sortMethod = 'recent';
            }

            // Update radio button
            document.querySelectorAll('input[name="library"]').forEach(radio => {
                radio.checked = radio.value === libraryId;
            });

            const lib = this.libraries.find(l => l.id === libraryId);
            if (lib) {
                this.currentLibrary = lib;
                document.getElementById('statusLibrary').textContent = lib.name;
            }

            this.updateStatus('Loading books...');
            await this.loadBooks();
            await this.loadFilters();

            // If switching libraries, clear UI; otherwise restore saved state
            if (isSwitchingLibrary) {
                const searchInput = document.getElementById('searchInput');
                if (searchInput) {
                    searchInput.value = '';
                }

                const sortSelect = document.getElementById('sortSelect');
                if (sortSelect) {
                    sortSelect.value = 'recent';
                }
                // Update filter count badges to show 0
                this.updateAuthorCount();
                this.updateTagCount();
                this.updateSeriesCount();
                this.updateFormatCount();
                // Save the new library ID with cleared filters to cookie
                this.saveAppState();
            } else {
                // On initial load, restore saved state if it exists for this library
                const savedState = this.getCookie('biblioAppState');
                if (savedState && savedState.currentLibraryId === libraryId) {
                    this.selectedAuthors = new Set((savedState.selectedAuthors || []).map(id => parseInt(id)));
                    this.selectedTags = new Set((savedState.selectedTags || []).map(id => parseInt(id)));
                    this.selectedSeries = new Set((savedState.selectedSeries || []).map(id => parseInt(id)));
                    this.selectedFormats = new Set(savedState.selectedFormats || []);
                    this.searchTerm = savedState.searchTerm || '';
                    this.sortMethod = savedState.sortMethod || 'recent';
                    this.restoreFilterUI();
                    this.applyFilters();
                }
            }

            this.updateStatus('Ready');
        } catch (error) {
            console.error('Error selecting library:', error);
            this.updateStatus('Error loading library');
        }
    }

    async loadBooks() {
        try {
            const response = await fetch(`/api/libraries/${this.currentLibraryId}/books`);
            const data = await response.json();

            if (data.success) {
                this.allBooks = data.data || [];
                document.getElementById('statusBooks').textContent = this.allBooks.length;
                this.applyFilters();
            } else {
                console.error('Failed to load books:', data.error);
            }
        } catch (error) {
            console.error('Error loading books:', error);
        }
    }

    async loadFilters() {
        try {
            await Promise.all([
                this.loadAuthors(),
                this.loadTags(),
                this.loadSeries(),
                this.loadFormats()
            ]);
        } catch (error) {
            console.error('Error loading filters:', error);
        }
    }

    async loadAuthors() {
        try {
            const response = await fetch(`/api/libraries/${this.currentLibraryId}/authors`);
            const data = await response.json();

            if (data.success) {
                this.renderAuthors(data.data || []);
            }
        } catch (error) {
            console.error('Error loading authors:', error);
        }
    }

    renderAuthors(authors) {
        this.authors = authors;
        const authorsList = document.getElementById('authorsList');
        authorsList.innerHTML = '';

        if (authors.length === 0) {
            authorsList.innerHTML = '<div style="padding: 10px; color: #95a5a6; font-size: 12px;">No authors found.</div>';
            return;
        }

        // Sort authors by sort field in lowercase
        const sortedAuthors = [...authors].sort((a, b) => {
            const sortA = (a.sort || a.name).toLowerCase();
            const sortB = (b.sort || b.name).toLowerCase();
            return sortA.localeCompare(sortB);
        });

        // If more than 100 authors, group by first letter of sort field
        if (sortedAuthors.length > 100) {
            this.renderAuthorsGrouped(sortedAuthors);
        } else {
            this.renderAuthorsFlat(sortedAuthors);
        }
    }

    renderAuthorsFlat(sortedAuthors) {
        const authorsList = document.getElementById('authorsList');
        sortedAuthors.forEach(author => {
            const item = document.createElement('div');
            item.className = 'filter-item';
            const displayName = this.formatAuthorName(author.name);
            const bookCount = author.book_count || 0;
            item.innerHTML = `
                <input type="checkbox" id="author_${author.id}" value="${author.id}" 
                       onchange="app.toggleAuthorFilter(${author.id})">
                <label for="author_${author.id}">${displayName} (${bookCount})</label>
            `;
            authorsList.appendChild(item);
        });
    }

    renderAuthorsGrouped(sortedAuthors) {
        const authorsList = document.getElementById('authorsList');

        // Group authors by first letter of sort field
        const groups = {};
        sortedAuthors.forEach(author => {
            const lastNameKey = this.getLastNameFirstLetter(author);
            if (!groups[lastNameKey]) {
                groups[lastNameKey] = [];
            }
            groups[lastNameKey].push(author);
        });

        // Sort group keys alphabetically
        const sortedKeys = Object.keys(groups).sort();

        // Create collapsible sections for each letter
        sortedKeys.forEach((letter, index) => {
            const groupContainer = document.createElement('div');
            groupContainer.className = 'author-group';

            // Group header (clickable to expand/collapse)
            const header = document.createElement('div');
            header.className = 'author-group-header';
            header.style.cssText = 'cursor: pointer; padding: 8px 10px; background: #ecf0f1; font-weight: 500; border-radius: 3px; margin-bottom: 5px; user-select: none; display: flex; align-items: center; justify-content: space-between;';

            const letterSpan = document.createElement('span');
            letterSpan.textContent = `${letter} (${groups[letter].length})`;
            header.appendChild(letterSpan);

            const toggleIcon = document.createElement('span');
            toggleIcon.textContent = '▼';
            toggleIcon.style.cssText = 'font-size: 12px; transition: transform 0.2s;';
            header.appendChild(toggleIcon);

            // Group content (authors)
            const content = document.createElement('div');
            content.className = 'author-group-content';
            content.style.cssText = 'padding-left: 10px; display: none;';

            groups[letter].forEach(author => {
                const item = document.createElement('div');
                item.className = 'filter-item';
                const displayName = this.formatAuthorName(author.name);
                const bookCount = author.book_count || 0;
                item.innerHTML = `
                    <input type="checkbox" id="author_${author.id}" value="${author.id}" 
                           onchange="app.toggleAuthorFilter(${author.id})">
                    <label for="author_${author.id}">${displayName} (${bookCount})</label>
                `;
                content.appendChild(item);
            });

            // Toggle functionality
            header.addEventListener('click', () => {
                const isOpen = content.style.display !== 'none';
                content.style.display = isOpen ? 'none' : 'block';
                toggleIcon.style.transform = isOpen ? 'rotate(0deg)' : 'rotate(-180deg)';
            });

            // Expand first group by default
            if (index === 0) {
                content.style.display = 'block';
                toggleIcon.style.transform = 'rotate(-180deg)';
            }

            groupContainer.appendChild(header);
            groupContainer.appendChild(content);
            authorsList.appendChild(groupContainer);
        });
    }

    getLastNameFirstLetter(author) {
        // Use the sort field for consistent grouping
        const sortField = author.sort || author.name;

        // Get first letter and convert to uppercase
        const firstLetter = sortField.charAt(0).toUpperCase();
        // Return letter or # for non-alphabetic
        return /[A-Z]/.test(firstLetter) ? firstLetter : '#';
    }

    getFirstLetterFromString(str) {
        // Get first letter from a string and convert to uppercase
        const firstLetter = str.charAt(0).toUpperCase();
        // Return letter or # for non-alphabetic
        return /[A-Z]/.test(firstLetter) ? firstLetter : '#';
    }

    formatAuthorName(name) {
        // If name contains "|", reverse the parts (lastname|firstname -> firstname lastname)
        if (name.includes('|')) {
            const parts = name.split('|');
            if (parts.length === 2) {
                const lastname = parts[0].trim();
                const firstname = parts[1].trim();
                return firstname && lastname ? `${firstname} ${lastname}` : name;
            }
        }
        return name;
    }

    async loadTags() {
        try {
            const response = await fetch(`/api/libraries/${this.currentLibraryId}/tags`);
            const data = await response.json();

            if (data.success) {
                this.renderTags(data.data || []);
            }
        } catch (error) {
            console.error('Error loading tags:', error);
        }
    }

    renderTags(tags) {
        this.tags = tags;
        const tagsList = document.getElementById('tagsList');
        tagsList.innerHTML = '';

        if (tags.length === 0) {
            tagsList.innerHTML = '<div style="padding: 10px; color: #95a5a6; font-size: 12px;">No tags found.</div>';
            return;
        }

        // Sort tags by name in lowercase
        const sortedTags = [...tags].sort((a, b) => {
            const sortA = a.name.toLowerCase();
            const sortB = b.name.toLowerCase();
            return sortA.localeCompare(sortB);
        });

        // If more than 100 tags, group by first letter
        if (sortedTags.length > 100) {
            this.renderTagsGrouped(sortedTags);
        } else {
            this.renderTagsFlat(sortedTags);
        }
    }

    renderTagsFlat(sortedTags) {
        const tagsList = document.getElementById('tagsList');
        sortedTags.forEach(tag => {
            const item = document.createElement('div');
            item.className = 'filter-item';
            const bookCount = tag.book_count || 0;
            item.innerHTML = `
                <input type="checkbox" id="tag_${tag.id}" value="${tag.id}" 
                       onchange="app.toggleTagFilter(${tag.id})">
                <label for="tag_${tag.id}">${tag.name} (${bookCount})</label>
            `;
            tagsList.appendChild(item);
        });
    }

    renderTagsGrouped(sortedTags) {
        const tagsList = document.getElementById('tagsList');

        // Group tags by first letter
        const groups = {};
        sortedTags.forEach(tag => {
            const firstLetter = this.getFirstLetterFromString(tag.name);
            if (!groups[firstLetter]) {
                groups[firstLetter] = [];
            }
            groups[firstLetter].push(tag);
        });

        // Sort group keys alphabetically
        const sortedKeys = Object.keys(groups).sort();

        // Create collapsible sections for each letter
        sortedKeys.forEach((letter, index) => {
            const groupContainer = document.createElement('div');
            groupContainer.className = 'filter-group';

            // Group header (clickable to expand/collapse)
            const header = document.createElement('div');
            header.className = 'filter-group-header';
            header.style.cssText = 'cursor: pointer; padding: 8px 10px; background: #ecf0f1; font-weight: 500; border-radius: 3px; margin-bottom: 5px; user-select: none; display: flex; align-items: center; justify-content: space-between;';

            const letterSpan = document.createElement('span');
            letterSpan.textContent = `${letter} (${groups[letter].length})`;
            header.appendChild(letterSpan);

            const toggleIcon = document.createElement('span');
            toggleIcon.textContent = '▼';
            toggleIcon.style.cssText = 'font-size: 12px; transition: transform 0.2s;';
            header.appendChild(toggleIcon);

            // Group content (tags)
            const content = document.createElement('div');
            content.className = 'filter-group-content';
            content.style.cssText = 'padding-left: 10px; display: none;';

            groups[letter].forEach(tag => {
                const item = document.createElement('div');
                item.className = 'filter-item';
                const bookCount = tag.book_count || 0;
                item.innerHTML = `
                    <input type="checkbox" id="tag_${tag.id}" value="${tag.id}" 
                           onchange="app.toggleTagFilter(${tag.id})">
                    <label for="tag_${tag.id}">${tag.name} (${bookCount})</label>
                `;
                content.appendChild(item);
            });

            // Toggle functionality
            header.addEventListener('click', () => {
                const isOpen = content.style.display !== 'none';
                content.style.display = isOpen ? 'none' : 'block';
                toggleIcon.style.transform = isOpen ? 'rotate(0deg)' : 'rotate(-180deg)';
            });

            // Expand first group by default
            if (index === 0) {
                content.style.display = 'block';
                toggleIcon.style.transform = 'rotate(-180deg)';
            }

            groupContainer.appendChild(header);
            groupContainer.appendChild(content);
            tagsList.appendChild(groupContainer);
        });
    }

    async loadSeries() {
        try {
            const response = await fetch(`/api/libraries/${this.currentLibraryId}/series`);
            const data = await response.json();

            if (data.success) {
                this.renderSeries(data.data || []);
            }
        } catch (error) {
            console.error('Error loading series:', error);
        }
    }

    renderSeries(series) {
        this.series = series;
        const seriesList = document.getElementById('seriesList');
        seriesList.innerHTML = '';

        if (series.length === 0) {
            seriesList.innerHTML = '<div style="padding: 10px; color: #95a5a6; font-size: 12px;">No series found.</div>';
            return;
        }

        // Sort series by sort field in lowercase
        const sortedSeries = [...series].sort((a, b) => {
            const sortA = (a.sort || a.name).toLowerCase();
            const sortB = (b.sort || b.name).toLowerCase();
            return sortA.localeCompare(sortB);
        });

        // If more than 100 series, group by first letter of sort field
        if (sortedSeries.length > 100) {
            this.renderSeriesGrouped(sortedSeries);
        } else {
            this.renderSeriesFlat(sortedSeries);
        }
    }

    renderSeriesFlat(sortedSeries) {
        const seriesList = document.getElementById('seriesList');
        sortedSeries.forEach(s => {
            const item = document.createElement('div');
            item.className = 'filter-item';
            const bookCount = s.book_count || 0;
            item.innerHTML = `
                <input type="checkbox" id="series_${s.id}" value="${s.id}" 
                       onchange="app.toggleSeriesFilter(${s.id})">
                <label for="series_${s.id}">${s.name} (${bookCount})</label>
            `;
            seriesList.appendChild(item);
        });
    }

    renderSeriesGrouped(sortedSeries) {
        const seriesList = document.getElementById('seriesList');

        // Group series by first letter of sort field
        const groups = {};
        sortedSeries.forEach(s => {
            const sortField = s.sort || s.name;
            const firstLetter = this.getFirstLetterFromString(sortField);
            if (!groups[firstLetter]) {
                groups[firstLetter] = [];
            }
            groups[firstLetter].push(s);
        });

        // Sort group keys alphabetically
        const sortedKeys = Object.keys(groups).sort();

        // Create collapsible sections for each letter
        sortedKeys.forEach((letter, index) => {
            const groupContainer = document.createElement('div');
            groupContainer.className = 'filter-group';

            // Group header (clickable to expand/collapse)
            const header = document.createElement('div');
            header.className = 'filter-group-header';
            header.style.cssText = 'cursor: pointer; padding: 8px 10px; background: #ecf0f1; font-weight: 500; border-radius: 3px; margin-bottom: 5px; user-select: none; display: flex; align-items: center; justify-content: space-between;';

            const letterSpan = document.createElement('span');
            letterSpan.textContent = `${letter} (${groups[letter].length})`;
            header.appendChild(letterSpan);

            const toggleIcon = document.createElement('span');
            toggleIcon.textContent = '▼';
            toggleIcon.style.cssText = 'font-size: 12px; transition: transform 0.2s;';
            header.appendChild(toggleIcon);

            // Group content (series)
            const content = document.createElement('div');
            content.className = 'filter-group-content';
            content.style.cssText = 'padding-left: 10px; display: none;';

            groups[letter].forEach(s => {
                const item = document.createElement('div');
                item.className = 'filter-item';
                const bookCount = s.book_count || 0;
                item.innerHTML = `
                    <input type="checkbox" id="series_${s.id}" value="${s.id}" 
                           onchange="app.toggleSeriesFilter(${s.id})">
                    <label for="series_${s.id}">${s.name} (${bookCount})</label>
                `;
                content.appendChild(item);
            });

            // Toggle functionality
            header.addEventListener('click', () => {
                const isOpen = content.style.display !== 'none';
                content.style.display = isOpen ? 'none' : 'block';
                toggleIcon.style.transform = isOpen ? 'rotate(0deg)' : 'rotate(-180deg)';
            });

            // Expand first group by default
            if (index === 0) {
                content.style.display = 'block';
                toggleIcon.style.transform = 'rotate(-180deg)';
            }

            groupContainer.appendChild(header);
            groupContainer.appendChild(content);
            seriesList.appendChild(groupContainer);
        });
    }

    async loadFormats() {
        try {
            // Extract all unique formats from current books and count them
            this.formatsCount = {};
            this.allBooks.forEach(book => {
                if (book.formats && Array.isArray(book.formats)) {
                    book.formats.forEach(format => {
                        this.formatsCount[format] = (this.formatsCount[format] || 0) + 1;
                    });
                }
            });

            this.formats = Object.keys(this.formatsCount).sort();
            this.renderFormats(this.formats, this.formatsCount);
        } catch (error) {
            console.error('Error loading formats:', error);
        }
    }

    renderFormats(formats, formatsCount) {
        const formatsList = document.getElementById('formatsList');
        formatsList.innerHTML = '';

        if (formats.length === 0) {
            formatsList.innerHTML = '<div style="padding: 10px; color: #95a5a6; font-size: 12px;">No formats found.</div>';
            return;
        }

        formats.forEach(format => {
            const item = document.createElement('div');
            item.className = 'filter-item';
            const count = formatsCount[format] || 0;
            item.innerHTML = `
                <input type="checkbox" id="format_${format}" value="${format}" 
                       onchange="app.toggleFormatFilter('${format}')">
                <label for="format_${format}">${format.toUpperCase()} (${count})</label>
            `;
            formatsList.appendChild(item);
        });
    }

    toggleFormatFilter(format) {
        if (this.selectedFormats.has(format)) {
            this.selectedFormats.delete(format);
        } else {
            this.selectedFormats.add(format);
        }
        this.saveAppState();
        this.updateFormatCount();
        this.applyFilters();
    }

    toggleAuthorFilter(authorId) {
        authorId = parseInt(authorId);
        if (this.selectedAuthors.has(authorId)) {
            this.selectedAuthors.delete(authorId);
        } else {
            this.selectedAuthors.add(authorId);
        }
        this.saveAppState();
        this.updateAuthorCount();
        this.applyFilters();
    }

    toggleTagFilter(tagId) {
        tagId = parseInt(tagId);
        if (this.selectedTags.has(tagId)) {
            this.selectedTags.delete(tagId);
        } else {
            this.selectedTags.add(tagId);
        }
        this.saveAppState();
        this.updateTagCount();
        this.applyFilters();
    }

    toggleSeriesFilter(seriesId) {
        seriesId = parseInt(seriesId);
        if (this.selectedSeries.has(seriesId)) {
            this.selectedSeries.delete(seriesId);
        } else {
            this.selectedSeries.add(seriesId);
        }
        this.saveAppState();
        this.updateSeriesCount();
        this.applyFilters();
    }

    clearAuthors() {
        this.selectedAuthors.clear();
        this.saveAppState();
        this.renderAuthors(this.authors);
        this.updateAuthorCount();
        this.applyFilters();
    }

    clearFormats() {
        this.selectedFormats.clear();
        this.saveAppState();
        this.renderFormats(this.formats, this.formatsCount);
        this.updateFormatCount();
        this.applyFilters();
    }

    clearTags() {
        this.selectedTags.clear();
        this.saveAppState();
        this.renderTags(this.tags);
        this.updateTagCount();
        this.applyFilters();
    }

    clearSeries() {
        this.selectedSeries.clear();
        this.saveAppState();
        this.renderSeries(this.series);
        this.updateSeriesCount();
        this.applyFilters();
    }

    updateAuthorCount() {
        const span = document.getElementById('authorCount');
        if (span) {
            span.textContent = this.selectedAuthors.size > 0 ? `(${this.selectedAuthors.size})` : '';
        }
    }

    updateFormatCount() {
        const span = document.getElementById('formatCount');
        if (span) {
            span.textContent = this.selectedFormats.size > 0 ? `(${this.selectedFormats.size})` : '';
        }
    }

    updateTagCount() {
        const span = document.getElementById('tagCount');
        if (span) {
            span.textContent = this.selectedTags.size > 0 ? `(${this.selectedTags.size})` : '';
        }
    }

    updateSeriesCount() {
        const span = document.getElementById('seriesCount');
        if (span) {
            span.textContent = this.selectedSeries.size > 0 ? `(${this.selectedSeries.size})` : '';
        }
    }

    applyFilters() {
        let filtered = [...this.allBooks];

        // Apply search filter
        if (this.searchTerm) {
            const searchLower = this.searchTerm.toLowerCase();
            filtered = filtered.filter(book =>
                book.title.toLowerCase().includes(searchLower) ||
                book.authors.some(a => a.toLowerCase().includes(searchLower))
            );
        }

        // Apply author filter
        if (this.selectedAuthors.size > 0) {
            const selectedAuthorNames = Array.from(this.selectedAuthors).map(id => {
                const author = this.authors.find(a => a.id === id);
                return author ? author.name : null;
            }).filter(name => name !== null);

            filtered = filtered.filter(book =>
                book.authors.some(bookAuthor =>
                    selectedAuthorNames.some(selectedName =>
                        bookAuthor.toLowerCase() === selectedName.toLowerCase()
                    )
                )
            );
        }

        // Apply tag filter
        if (this.selectedTags.size > 0) {
            const selectedTagNames = Array.from(this.selectedTags).map(id => {
                const tag = this.tags.find(t => t.id === id);
                return tag ? tag.name : null;
            }).filter(name => name !== null);

            filtered = filtered.filter(book =>
                book.tags.some(bookTag =>
                    selectedTagNames.some(selectedName =>
                        bookTag.toLowerCase() === selectedName.toLowerCase()
                    )
                )
            );
        }

        // Apply series filter
        if (this.selectedSeries.size > 0) {
            const selectedSeriesNames = Array.from(this.selectedSeries).map(id => {
                const series = this.series.find(s => s.id === id);
                console.log(`Looking for series id ${id} in:`, this.series.map(s => ({ id: s.id, name: s.name })));
                console.log(`Found series:`, series);
                return series ? series.name : null;
            }).filter(name => name !== null);

            console.log('Selected series names:', selectedSeriesNames);
            console.log('Sample books:', filtered.slice(0, 3).map(b => ({ title: b.title, series: b.series })));

            filtered = filtered.filter(book => {
                const match = book.series && selectedSeriesNames.some(selectedName =>
                    book.series.toLowerCase() === selectedName.toLowerCase()
                );
                if (book.series) {
                    console.log(`Book "${book.title}" series="${book.series}", matches=${match}`);
                }
                return match;
            });
        }

        // Apply format filter
        if (this.selectedFormats.size > 0) {
            const selectedFormatsUpper = Array.from(this.selectedFormats).map(f => f.toUpperCase());
            filtered = filtered.filter(book =>
                book.formats && book.formats.some(bookFormat =>
                    selectedFormatsUpper.includes(bookFormat.toUpperCase())
                )
            );
        }

        // Apply sorting
        filtered = this.sortBooks(filtered);

        this.filteredBooks = filtered;
        document.getElementById('statusFiltered').textContent = filtered.length;
        this.displayedBooksCount = 0;
        this.renderBooks();

        // Fill screen with available books if there's room
        if (this.currentViewMode === 'grid') {
            setTimeout(() => this.fillScreenWithCovers(), 100);
        } else if (this.currentViewMode === 'table') {
            setTimeout(() => this.fillScreenWithRows(), 100);
        }
    }

    sortBooks(books) {
        const sorted = [...books];

        switch (this.sortMethod) {
            case 'title':
                sorted.sort((a, b) => {
                    const sortA = (a.sort || a.title).toLowerCase();
                    const sortB = (b.sort || b.title).toLowerCase();
                    return sortA.localeCompare(sortB);
                });
                break;
            case 'author':
                sorted.sort((a, b) => {
                    const authorA = a.authors[0] || '';
                    const authorB = b.authors[0] || '';
                    // Find the author objects to get their sort fields
                    const authorObjA = this.authors.find(auth => auth.name === authorA);
                    const authorObjB = this.authors.find(auth => auth.name === authorB);
                    const sortA = (authorObjA?.sort || authorA).toLowerCase();
                    const sortB = (authorObjB?.sort || authorB).toLowerCase();
                    return sortA.localeCompare(sortB);
                });
                break;
            case 'recent':
            default:
                // Keep original order (most recent first)
                break;
        }

        return sorted;
    }

    renderBooks() {
        // Check current view mode and render accordingly
        if (this.currentViewMode === 'table') {
            this.renderBooksTable();
        } else {
            this.renderBooksGrid();
        }
    }

    renderBooksGrid() {
        const booksGrid = document.getElementById('booksGrid');

        // Clear grid on first render
        if (this.displayedBooksCount === 0) {
            booksGrid.innerHTML = '';
        }

        if (this.filteredBooks.length === 0) {
            booksGrid.innerHTML = '<div class="no-selection">No books found</div>';
            return;
        }

        // Calculate how many books to display
        const endIndex = Math.min(this.displayedBooksCount + this.booksPerPage, this.filteredBooks.length);
        const booksToDisplay = this.filteredBooks.slice(this.displayedBooksCount, endIndex);

        booksToDisplay.forEach(book => {
            const bookItem = document.createElement('div');
            bookItem.className = 'book-item' + (this.selectedBook?.id === book.id ? ' selected' : '');
            bookItem.onclick = () => this.selectBook(book);

            const coverDiv = document.createElement('div');
            coverDiv.className = 'book-cover';

            if (book.has_cover) {
                const img = document.createElement('img');
                img.src = `/api/libraries/${this.currentLibraryId}/books/${book.id}/cover`;
                img.onerror = () => {
                    img.parentElement.innerHTML = '<div class="no-image">No Cover</div>';
                };
                coverDiv.appendChild(img);
            } else {
                const img = document.createElement('img');
                const authors = book.authors && book.authors.length > 0 ? book.authors[0] : 'Unknown Author';
                img.src = this.generateTemporaryCover(book.title, authors);
                coverDiv.appendChild(img);
            }

            const titleDiv = document.createElement('div');
            titleDiv.className = 'book-title';
            titleDiv.textContent = book.title;

            bookItem.appendChild(coverDiv);
            bookItem.appendChild(titleDiv);
            booksGrid.appendChild(bookItem);
        });

        this.displayedBooksCount = endIndex;
        this.gridRenderedForLibrary = this.currentLibraryId;

        // Setup infinite scroll on first render
        if (this.displayedBooksCount === this.booksPerPage) {
            this.setupInfiniteScroll();
        }
    }

    setupInfiniteScroll() {
        const booksGrid = document.getElementById('booksGrid');

        // Remove existing listener if any
        booksGrid.removeEventListener('scroll', this.handleScroll);

        // Add new scroll listener
        this.handleScroll = () => {
            if (this.isLoadingMore || this.displayedBooksCount >= this.filteredBooks.length) {
                return;
            }

            const scrollTop = booksGrid.scrollTop;
            const scrollHeight = booksGrid.scrollHeight;
            const clientHeight = booksGrid.clientHeight;

            // Load more books when user scrolls to 80% of the way down
            if (scrollTop + clientHeight >= scrollHeight * 0.8) {
                this.isLoadingMore = true;
                setTimeout(() => {
                    this.renderBooks();
                    this.isLoadingMore = false;
                }, 100);
            }
        };

        booksGrid.addEventListener('scroll', this.handleScroll);
    }

    async selectBook(book, libraryId) {
        this.selectedBook = book;
        this.selectedBookId = book.id;
        this.selectedBookLibraryId = libraryId || this.currentLibraryId;
        this.saveAppState();

        // Update selected styling for grid view
        document.querySelectorAll('.book-item').forEach(item => {
            item.classList.remove('selected');
        });

        // Update selected styling for table view
        document.querySelectorAll('.books-table tbody tr').forEach(row => {
            row.classList.remove('selected');
        });

        // Add selected class to appropriate element
        if (this.currentViewMode === 'grid') {
            // For grid view, try to find the element that was clicked
            if (event && event.target) {
                const bookItem = event.target.closest('.book-item');
                if (bookItem) {
                    bookItem.classList.add('selected');
                }
            }
        } else if (this.currentViewMode === 'table') {
            // For table view, find and highlight the row
            const selectedRow = document.querySelector(`tr[data-book-id="${book.id}"][data-library-id="${this.selectedBookLibraryId}"]`);
            if (selectedRow) {
                selectedRow.classList.add('selected');
            }
        }

        this.renderBookDetails(book);
    }

    async restoreSelectedBook() {
        try {
            // Try to find the book in the current library's all books
            let book = this.allBooks.find(b => b.id === this.selectedBookId);

            if (book) {
                // Book is in the current library, restore it
                this.selectedBook = book;
                this.renderBookDetails(book);

                // Highlight the book in the grid if visible
                document.querySelectorAll('.book-item').forEach(item => {
                    item.classList.remove('selected');
                });
                // Find and highlight the book item
                const bookItems = document.querySelectorAll('.book-item');
                for (let item of bookItems) {
                    if (item.querySelector('img')?.src?.includes(`/books/${this.selectedBookId}/`)) {
                        item.classList.add('selected');
                        break;
                    }
                }
            } else {
                // Book is not in the current library
                // Show the first book of the current library if there is one
                if (this.allBooks.length > 0) {
                    const firstBook = this.allBooks[0];
                    this.selectedBook = firstBook;
                    this.selectedBookId = firstBook.id;
                    this.selectedBookLibraryId = this.currentLibraryId;
                    this.saveAppState();
                    this.renderBookDetails(firstBook);

                    // Highlight the first book in the grid
                    document.querySelectorAll('.book-item').forEach(item => {
                        item.classList.remove('selected');
                    });
                    const bookItems = document.querySelectorAll('.book-item');
                    if (bookItems.length > 0) {
                        bookItems[0].classList.add('selected');
                    }
                } else {
                    // No books in current library, keep it clear
                    this.selectedBook = null;
                    this.selectedBookId = null;
                    this.selectedBookLibraryId = null;
                    this.saveAppState();
                    const noSelection = document.getElementById('noSelection');
                    const bookDetails = document.getElementById('bookDetails');
                    if (noSelection && bookDetails) {
                        noSelection.style.display = 'block';
                        bookDetails.classList.remove('active');
                    }
                }
            }
        } catch (error) {
            console.error('Error restoring selected book:', error);
        }
    }

    renderBookDetails(book) {
        const noSelection = document.getElementById('noSelection');
        const bookDetails = document.getElementById('bookDetails');

        noSelection.style.display = 'none';
        bookDetails.classList.add('active');

        document.getElementById('detailTitle').textContent = book.title;
        const formattedAuthors = book.authors.map(author => this.formatAuthorName(author)).join(' & ');
        document.getElementById('detailAuthors').textContent = formattedAuthors || 'Unknown';

        // Handle Series section
        let seriesText = 'None';
        if (book.series && book.series_index) {
            seriesText = `${book.series} #${book.series_index}`;
        } else if (book.series) {
            seriesText = book.series;
        }
        document.getElementById('detailSeries').textContent = seriesText;
        this.toggleDetailSection('detailSeriesSection', seriesText !== 'None');

        // Handle Tags section
        let tagsText = book.tags.length > 0 ? book.tags.join(', ') : '';
        document.getElementById('detailTags').textContent = tagsText;
        this.toggleDetailSection('detailTagsSection', tagsText !== '');

        // Handle Publisher section
        let publisherText = book.publisher || '';
        document.getElementById('detailPublisher').textContent = publisherText;
        this.toggleDetailSection('detailPublisherSection', publisherText !== '');

        // Handle Published section
        let pubdateText = book.pubdate ? new Date(book.pubdate).toLocaleDateString() : '';
        document.getElementById('detailPubdate').textContent = pubdateText;
        this.toggleDetailSection('detailPubdateSection', pubdateText !== '');

        // Handle Rating section
        let ratingText = book.rating ? `${book.rating}/10` : '';
        document.getElementById('detailRating').textContent = ratingText;
        this.toggleDetailSection('detailRatingSection', ratingText !== '');

        // Handle Comments section - use innerHTML to render HTML formatting
        if (book.comments && book.comments.trim()) {
            document.getElementById('detailComments').innerHTML = book.comments;
            this.toggleDetailSection('detailCommentsSection', true);
        } else {
            document.getElementById('detailComments').innerHTML = '';
            this.toggleDetailSection('detailCommentsSection', false);
        }

        // Update cover image
        const coverImage = document.getElementById('coverImage');
        if (book.has_cover) {
            coverImage.src = `/api/libraries/${this.currentLibraryId}/books/${book.id}/cover`;
            coverImage.style.display = 'block';
        } else {
            // Generate a temporary cover with title and author
            const tempCover = this.generateTemporaryCover(book.title, book.authors.join(', ') || 'Unknown');
            coverImage.src = tempCover;
            coverImage.style.display = 'block';
        }

        // Load and display formats
        this.loadAndDisplayFormats(book.id);
    }

    toggleDetailSection(sectionId, show) {
        const section = document.getElementById(sectionId);
        if (section) {
            section.style.display = show ? 'flex' : 'none';
        }
    }

    toggleFilterSection(sectionId) {
        const content = document.getElementById(sectionId);
        const header = content ? content.previousElementSibling : null;
        const toggle = header ? header.querySelector('.filter-section-toggle') : null;

        if (content && toggle) {
            content.classList.toggle('collapsed');
            toggle.classList.toggle('collapsed');
            this.saveFilterSectionStates();
        }
    }

    saveFilterSectionStates() {
        const sections = ['librariesList', 'authorsList', 'seriesList', 'tagsList', 'formatsList'];
        const states = {};
        sections.forEach(sectionId => {
            const content = document.getElementById(sectionId);
            if (content) {
                states[sectionId] = content.classList.contains('collapsed');
            }
        });
        localStorage.setItem('biblioFilterSectionStates', JSON.stringify(states));
    }

    restoreFilterSectionStates() {
        const states = localStorage.getItem('biblioFilterSectionStates');
        if (states) {
            try {
                const stateObj = JSON.parse(states);
                Object.keys(stateObj).forEach(sectionId => {
                    const content = document.getElementById(sectionId);
                    const header = content ? content.previousElementSibling : null;
                    const toggle = header ? header.querySelector('.filter-section-toggle') : null;

                    if (content && toggle) {
                        if (stateObj[sectionId]) {
                            content.classList.add('collapsed');
                            toggle.classList.add('collapsed');
                        } else {
                            content.classList.remove('collapsed');
                            toggle.classList.remove('collapsed');
                        }
                    }
                });
            } catch (e) {
                console.error('Failed to restore filter section states:', e);
            }
        }
    }

    generateTemporaryCover(title, author) {
        const width = 300;
        const height = 450;

        // Create a canvas element
        const canvas = document.createElement('canvas');
        canvas.width = width;
        canvas.height = height;
        const ctx = canvas.getContext('2d');

        // Generate a color based on title hash for variety
        const hash = title.split('').reduce((acc, char) => acc + char.charCodeAt(0), 0);
        const colors = [
            '#3498db', '#e74c3c', '#2ecc71', '#f39c12', '#9b59b6',
            '#1abc9c', '#e67e22', '#34495e', '#c0392b', '#16a085'
        ];
        const backgroundColor = colors[hash % colors.length];

        // Fill background with gradient
        const gradient = ctx.createLinearGradient(0, 0, 0, height);
        gradient.addColorStop(0, backgroundColor);
        gradient.addColorStop(1, this.shadeColor(backgroundColor, -30));
        ctx.fillStyle = gradient;
        ctx.fillRect(0, 0, width, height);

        // Add a decorative border
        ctx.strokeStyle = 'rgba(255, 255, 255, 0.2)';
        ctx.lineWidth = 2;
        ctx.strokeRect(10, 10, width - 20, height - 20);

        // Draw title
        ctx.fillStyle = '#ffffff';
        ctx.font = 'bold 28px Arial, sans-serif';
        ctx.textAlign = 'center';
        ctx.textBaseline = 'top';

        // Wrap title text
        const titleLines = this.wrapText(ctx, title, width - 40, 28);
        const titleStartY = height / 2 - (titleLines.length * 35) / 2;

        titleLines.forEach((line, index) => {
            ctx.fillText(line, width / 2, titleStartY + index * 35);
        });

        // Draw author
        ctx.fillStyle = 'rgba(255, 255, 255, 0.85)';
        ctx.font = '16px Arial, sans-serif';
        const authorY = titleStartY + titleLines.length * 35 + 30;
        ctx.fillText(author, width / 2, authorY);

        // Convert canvas to data URL
        return canvas.toDataURL('image/png');
    }

    wrapText(ctx, text, maxWidth, lineHeight) {
        const words = text.split(' ');
        const lines = [];
        let currentLine = '';

        words.forEach(word => {
            const testLine = currentLine + (currentLine ? ' ' : '') + word;
            const metrics = ctx.measureText(testLine);

            if (metrics.width > maxWidth && currentLine) {
                lines.push(currentLine);
                currentLine = word;
            } else {
                currentLine = testLine;
            }
        });

        if (currentLine) {
            lines.push(currentLine);
        }

        return lines;
    }

    shadeColor(color, percent) {
        const num = parseInt(color.replace('#', ''), 16);
        const amt = Math.round(2.55 * percent);
        const R = Math.max(0, Math.min(255, (num >> 16) + amt));
        const G = Math.max(0, Math.min(255, (num >> 8 & 0x00FF) + amt));
        const B = Math.max(0, Math.min(255, (num & 0x0000FF) + amt));
        return '#' + (0x1000000 + (R << 16) + (G << 8) + B).toString(16).slice(1);
    }

    async loadAndDisplayFormats(bookId) {
        const formatsContainer = document.getElementById('detailFormats');

        // Get formats from the selected book object
        if (this.selectedBook && this.selectedBook.formats && this.selectedBook.formats.length > 0) {
            formatsContainer.innerHTML = this.selectedBook.formats.map(format => {
                return `<a href="/api/libraries/${this.currentLibraryId}/books/${bookId}/formats/${format}" target="_blank" class="format-button">${format.toUpperCase()}</a>`;
            }).join('');
        } else {
            formatsContainer.innerHTML = '<span class="no-formats">No formats available</span>';
        }
    }

    updateSort() {
        this.sortMethod = document.getElementById('sortSelect').value;
        this.saveAppState();
        this.applyFilters();
    }

    setupEventListeners() {
        document.getElementById('searchInput').addEventListener('input', (e) => {
            this.searchTerm = e.target.value;

            // Show/hide clear button based on input value
            const clearBtn = document.getElementById('clearSearchBtn');
            if (clearBtn) {
                clearBtn.style.display = this.searchTerm ? 'flex' : 'none';
            }

            this.saveAppState();
            this.applyFilters();
        });

        // Setup splitter event listeners
        this.setupSplitters();

        // Setup responsive panel sizing
        this.setupResponsivePanelSizing();
    }

    clearSearch() {
        const searchInput = document.getElementById('searchInput');
        const clearBtn = document.getElementById('clearSearchBtn');

        // Clear the input and search term
        searchInput.value = '';
        this.searchTerm = '';

        // Hide clear button
        if (clearBtn) {
            clearBtn.style.display = 'none';
        }

        // Focus back on input
        searchInput.focus();

        // Reapply filters
        this.displayedBooksCount = 0;
        this.saveAppState();
        this.applyFilters();
    }

    setupSplitters() {
        const splitterLeft = document.getElementById('splitterLeft');
        const splitterRight = document.getElementById('splitterRight');

        // Helper function to get X coordinate from mouse or touch event
        const getClientX = (e) => {
            return e.clientX !== undefined ? e.clientX : (e.touches && e.touches[0] ? e.touches[0].clientX : 0);
        };

        // Helper function to start resizing from either mouse or touch
        const startResize = (side, e) => {
            e.preventDefault();
            this.resizing = {
                side: side,
                startX: getClientX(e),
                startLeftWidth: document.querySelector('.left-panel').offsetWidth,
                startRightWidth: document.querySelector('.right-panel').offsetWidth,
                splitter: e.target
            };
            e.target.classList.add('active');
        };

        if (splitterLeft) {
            splitterLeft.addEventListener('mousedown', (e) => startResize('left', e));
            splitterLeft.addEventListener('touchstart', (e) => startResize('left', e), { passive: false });
        }

        if (splitterRight) {
            splitterRight.addEventListener('mousedown', (e) => startResize('right', e));
            splitterRight.addEventListener('touchstart', (e) => startResize('right', e), { passive: false });
        }

        document.addEventListener('mousemove', (e) => this.handleResize(e));
        document.addEventListener('touchmove', (e) => this.handleResize(e), { passive: false });
        document.addEventListener('mouseup', () => this.stopResizing());
        document.addEventListener('touchend', () => this.stopResizing());

        // Load saved pane sizes
        this.loadPaneSizes();
    }

    handleResize(e) {
        if (!this.resizing) return;

        // Get X coordinate from mouse or touch event
        const clientX = e.clientX !== undefined ? e.clientX : (e.touches && e.touches[0] ? e.touches[0].clientX : null);
        if (clientX === null) return;

        const leftPanel = document.querySelector('.left-panel');
        const rightPanel = document.querySelector('.right-panel');

        if (!leftPanel || !rightPanel) return;

        if (this.resizing.side === 'left') {
            const deltaX = clientX - this.resizing.startX;
            const newLeftWidth = Math.max(150, Math.min(500, this.resizing.startLeftWidth + deltaX));
            leftPanel.style.width = newLeftWidth + 'px';
            leftPanel.style.flex = `0 0 ${newLeftWidth}px`;
        } else if (this.resizing.side === 'right') {
            const deltaX = clientX - this.resizing.startX;
            const newRightWidth = Math.max(150, Math.min(500, this.resizing.startRightWidth - deltaX));
            rightPanel.style.width = newRightWidth + 'px';
            rightPanel.style.flex = `0 0 ${newRightWidth}px`;
        }
    }

    stopResizing() {
        if (this.resizing) {
            if (this.resizing.splitter) {
                this.resizing.splitter.classList.remove('active');
            }
            this.savePaneSizes();

            // Re-apply table column widths when panels are resized
            if (this.currentViewMode === 'table') {
                this.loadTableColumnWidths();
            }

            this.resizing = null;
        }
    }

    savePaneSizes() {
        const leftWidth = document.querySelector('.left-panel').offsetWidth;
        const rightWidth = document.querySelector('.right-panel').offsetWidth;

        const paneSizes = {
            leftWidth: leftWidth,
            rightWidth: rightWidth
        };

        localStorage.setItem('biblioPaneSizes', JSON.stringify(paneSizes));
    }

    loadPaneSizes() {
        const paneSizes = localStorage.getItem('biblioPaneSizes');
        if (paneSizes) {
            try {
                const sizes = JSON.parse(paneSizes);
                const leftPanel = document.querySelector('.left-panel');
                const rightPanel = document.querySelector('.right-panel');

                if (sizes.leftWidth && leftPanel) {
                    leftPanel.style.width = sizes.leftWidth + 'px';
                    leftPanel.style.flex = `0 0 ${sizes.leftWidth}px`;
                }

                if (sizes.rightWidth && rightPanel) {
                    rightPanel.style.width = sizes.rightWidth + 'px';
                    rightPanel.style.flex = `0 0 ${sizes.rightWidth}px`;
                }
            } catch (error) {
                console.error('Error loading pane sizes:', error);
            }
        }

        // Check if adjustment is needed for current viewport
        // Use setTimeout to ensure DOM is fully rendered
        setTimeout(() => {
            this.adjustPanelsForViewport();
        }, 100);
    }

    // Responsive Panel Sizing - Auto-adjust when content exceeds screen
    setupResponsivePanelSizing() {
        // Auto-adjust on load
        this.adjustPanelsForViewport();

        // Re-adjust on window resize with debouncing
        let resizeTimeout;
        window.addEventListener('resize', () => {
            clearTimeout(resizeTimeout);
            resizeTimeout = setTimeout(() => {
                this.adjustPanelsForViewport();
            }, 250);
        });
    }

    adjustPanelsForViewport() {
        const contentArea = document.querySelector('.content-area');
        if (!contentArea) return;

        const leftPanel = document.querySelector('.left-panel');
        const rightPanel = document.querySelector('.right-panel');
        const splitterLeft = document.getElementById('splitterLeft');
        const splitterRight = document.getElementById('splitterRight');

        if (!leftPanel || !rightPanel || !splitterLeft || !splitterRight) return;

        // Get bounding rectangles for position tracking (kept for debug capability)
        const contentAreaBounds = contentArea.getBoundingClientRect();
        const leftPanelBounds = leftPanel.getBoundingClientRect();
        const rightPanelBounds = rightPanel.getBoundingClientRect();
        const splitterLeftBounds = splitterLeft.getBoundingClientRect();
        const splitterRightBounds = splitterRight.getBoundingClientRect();

        // Get available width (excluding padding and splitters)
        const contentAreaWidth = contentArea.offsetWidth;
        const contentAreaStyle = window.getComputedStyle(contentArea);
        const availableGap = 10 * 2; // padding: 10px on both sides
        const splitterWidth = 4; // 2px splitter * 2
        const minCenterWidth = Math.floor(contentAreaWidth * 0.55); // Minimum 55% of screen width for center panel

        let availableWidth = contentAreaWidth - availableGap - splitterWidth;
        let leftWidth = leftPanel.offsetWidth;
        let rightWidth = rightPanel.offsetWidth;

        const totalPanelWidth = leftWidth + rightWidth + minCenterWidth;

        let scaleFactor = null;
        let newLeftWidth = null;
        let newRightWidth = null;
        let adjustmentNeeded = false;

        // Check if panels exceed available space
        if (totalPanelWidth > availableWidth) {
            adjustmentNeeded = true;
            // Calculate scaling factor
            scaleFactor = (availableWidth - minCenterWidth) / (leftWidth + rightWidth);

            // Scale down panels proportionally
            newLeftWidth = Math.max(150, Math.floor(leftWidth * scaleFactor));
            newRightWidth = Math.max(150, Math.floor(rightWidth * scaleFactor));

            // Apply the new sizes
            leftPanel.style.width = newLeftWidth + 'px';
            leftPanel.style.flex = `0 0 ${newLeftWidth}px`;
            rightPanel.style.width = newRightWidth + 'px';
            rightPanel.style.flex = `0 0 ${newRightWidth}px`;

            // Save the adjusted sizes
            this.savePaneSizes();
        }

        // Send diagnostics to server (includes geometry data for debugging if needed)
        this.sendDiagnosticsToServer({
            window_width: window.innerWidth,
            window_height: window.innerHeight,
            content_area_width: contentAreaWidth,
            content_area_x: Math.round(contentAreaBounds.x),
            content_area_y: Math.round(contentAreaBounds.y),
            content_area_left: Math.round(contentAreaBounds.left),
            content_area_top: Math.round(contentAreaBounds.top),
            content_area_right: Math.round(contentAreaBounds.right),
            content_area_bottom: Math.round(contentAreaBounds.bottom),
            content_area_flex_direction: contentAreaStyle.flexDirection,
            left_panel_width: leftWidth,
            left_panel_x: Math.round(leftPanelBounds.x),
            left_panel_y: Math.round(leftPanelBounds.y),
            left_panel_left: Math.round(leftPanelBounds.left),
            left_panel_top: Math.round(leftPanelBounds.top),
            left_panel_right: Math.round(leftPanelBounds.right),
            left_panel_bottom: Math.round(leftPanelBounds.bottom),
            left_panel_height: Math.round(leftPanelBounds.height),
            right_panel_width: rightWidth,
            right_panel_x: Math.round(rightPanelBounds.x),
            right_panel_y: Math.round(rightPanelBounds.y),
            right_panel_left: Math.round(rightPanelBounds.left),
            right_panel_top: Math.round(rightPanelBounds.top),
            right_panel_right: Math.round(rightPanelBounds.right),
            right_panel_bottom: Math.round(rightPanelBounds.bottom),
            right_panel_height: Math.round(rightPanelBounds.height),
            splitter_left_x: Math.round(splitterLeftBounds.x),
            splitter_left_y: Math.round(splitterLeftBounds.y),
            splitter_right_x: Math.round(splitterRightBounds.x),
            splitter_right_y: Math.round(splitterRightBounds.y),
            available_width: availableWidth,
            min_center_width: minCenterWidth,
            total_required_width: totalPanelWidth,
            adjustment_needed: adjustmentNeeded,
            scale_factor: scaleFactor,
            new_left_width: newLeftWidth,
            new_right_width: newRightWidth
        });
    }

    sendDiagnosticsToServer(diagnostics) {
        fetch('/api/diagnostics/client', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify(diagnostics)
        }).catch(err => {
            // Silently fail if server endpoint not available (no console logging to avoid clutter)
        });
    }

    // View Mode and Table Column Width Management
    setViewMode(mode) {
        this.currentViewMode = mode;
        this.saveViewPreferences();
        this.updateViewDisplay();
    }

    updateViewDisplay() {
        const booksGrid = document.getElementById('booksGrid');
        const booksTable = document.getElementById('booksTable');
        const gridRadio = document.querySelector('input[name="viewMode"][value="grid"]');
        const tableRadio = document.querySelector('input[name="viewMode"][value="table"]');
        const columnVisibilityItem = document.getElementById('columnVisibilityItem');
        const coverSizeItem = document.getElementById('coverSizeItem');

        if (this.currentViewMode === 'grid') {
            booksGrid.style.display = 'grid';
            booksTable.style.display = 'none';
            if (gridRadio) gridRadio.checked = true;
            if (columnVisibilityItem) columnVisibilityItem.style.display = 'none';
            if (coverSizeItem) coverSizeItem.style.display = 'flex';

            // Apply current cover size (set CSS custom properties)
            this.applyCoverSizeCSS();

            // Always reset displayed count when switching to grid mode to show current filters
            this.displayedBooksCount = 0;
            this.renderBooks();
            this.setupInfiniteScroll();
        } else {
            booksGrid.style.display = 'none';
            booksTable.style.display = 'flex';
            if (tableRadio) tableRadio.checked = true;
            if (columnVisibilityItem) columnVisibilityItem.style.display = 'flex';
            if (coverSizeItem) coverSizeItem.style.display = 'none';
            // Always reset displayed count when switching to table mode to show current filters
            this.displayedBooksCount = 0;
            this.renderBooksTable();
            this.setupTableInfiniteScroll();
        }
    }

    saveViewPreferences() {
        const preferences = {
            viewMode: this.currentViewMode,
            coverSize: this.coverSize,
            tableColumnWidths: this.tableColumnWidths,
            columnVisibility: this.columnVisibility
        };
        localStorage.setItem('biblioViewPreferences', JSON.stringify(preferences));
    }

    loadViewPreferences() {
        const preferences = localStorage.getItem('biblioViewPreferences');
        if (preferences) {
            try {
                const prefs = JSON.parse(preferences);
                if (prefs.viewMode) {
                    this.currentViewMode = prefs.viewMode;
                }
                if (prefs.coverSize) {
                    this.coverSize = prefs.coverSize;
                }
                if (prefs.tableColumnWidths) {
                    this.tableColumnWidths = prefs.tableColumnWidths;
                }
                if (prefs.columnVisibility) {
                    this.columnVisibility = prefs.columnVisibility;
                }
            } catch (error) {
                console.error('Error loading view preferences:', error);
            }
        }

        // Sync checkboxes with loaded column visibility state
        this.syncColumnVisibilityCheckboxes();
    }

    syncColumnVisibilityCheckboxes() {
        const columns = ['title', 'authors', 'series', 'publisher', 'rating', 'pubdate'];
        columns.forEach(colName => {
            const checkbox = document.querySelector(`input[data-col="${colName}"]`);
            if (checkbox) {
                checkbox.checked = this.columnVisibility[colName];
            }
        });
    }

    toggleColumnVisibility(colName) {
        this.columnVisibility[colName] = !this.columnVisibility[colName];

        // If showing a column, ensure it has a valid width
        if (this.columnVisibility[colName]) {
            const currentWidth = this.tableColumnWidths[colName];
            if (!currentWidth || currentWidth <= 0) {
                // Assign default widths for columns
                const defaultWidths = {
                    title: 30,
                    authors: 20,
                    series: 15,
                    publisher: 15,
                    rating: 10,
                    pubdate: 10
                };
                this.tableColumnWidths[colName] = defaultWidths[colName] || 15;
            }
        }

        this.saveViewPreferences();
        this.updateColumnVisibility();

        // Update checkbox state
        const checkbox = document.querySelector(`input[data-col="${colName}"]`);
        if (checkbox) {
            checkbox.checked = this.columnVisibility[colName];
        }
    }

    applyCoverSizeCSS() {
        const size = this.coverSize;
        const grid = document.getElementById('booksGrid');
        if (grid) {
            // Calculate proportional spacing based on default 120px with 15px gap
            // Ratio: 15/120 = 0.125 for gap, 8/120 = 0.067 for margin
            const proportionalGap = Math.round(size * 0.125);
            const proportionalPadding = proportionalGap;
            const proportionalMargin = Math.round(size * 0.067);

            grid.style.setProperty('--cover-width', size + 'px');
            grid.style.setProperty('--item-gap', proportionalGap + 'px');
            grid.style.setProperty('--grid-padding', proportionalPadding + 'px');
            grid.style.setProperty('--cover-margin', proportionalMargin + 'px');

            // Calculate title font size and line clamp based on cover size
            const { fontSize, lineClamp } = this.calculateTitleMetrics(size);
            grid.style.setProperty('--title-font-size', fontSize + 'px');
            grid.style.setProperty('--title-lines', lineClamp.toString());

            // After CSS properties are applied, check if we need to load more books to fill screen
            setTimeout(() => this.fillScreenWithCovers(), 50);
        }

        // Update range slider and label
        const slider = document.getElementById('coverSizeSlider');
        const label = document.getElementById('coverSizeLabel');
        if (slider) slider.value = size;
        if (label) label.textContent = size;
    }

    fillScreenWithCovers() {
        const booksGrid = document.getElementById('booksGrid');
        if (!booksGrid || this.currentViewMode !== 'grid') return;

        const scrollHeight = booksGrid.scrollHeight;
        const clientHeight = booksGrid.clientHeight;

        // If there's vertical space available and we have more books to load, load them
        if (scrollHeight <= clientHeight && this.displayedBooksCount < this.filteredBooks.length) {
            this.isLoadingMore = true;
            setTimeout(() => {
                this.renderBooks();
                this.isLoadingMore = false;
                // Recursively check again in case more space is available
                setTimeout(() => this.fillScreenWithCovers(), 100);
            }, 50);
        }
    }

    fillScreenWithRows() {
        const booksTable = document.getElementById('booksTable');
        if (!booksTable || this.currentViewMode !== 'table') return;

        const scrollHeight = booksTable.scrollHeight;
        const clientHeight = booksTable.clientHeight;

        // If there's vertical space available and we have more books to load, load them
        if (scrollHeight <= clientHeight && this.displayedBooksCount < this.filteredBooks.length) {
            this.isLoadingMore = true;
            setTimeout(() => {
                this.renderBooksTable();
                this.isLoadingMore = false;
                // Recursively check again in case more space is available
                setTimeout(() => this.fillScreenWithRows(), 100);
            }, 50);
        }
    }

    calculateTitleMetrics(coverWidth) {
        // Calculate dynamic font size: scale from 10px (at 50px) to 14px (at 250px)
        // Linear interpolation: fontSize = 10 + (coverWidth - 50) * (14 - 10) / (250 - 50)
        const fontSize = Math.max(10, Math.min(14, 10 + (coverWidth - 50) * 4 / 200));

        // Calculate line clamp based on cover width
        // At small sizes: 1 line, at medium: 2 lines, at large: 3 lines
        let lineClamp = 2;
        if (coverWidth < 80) {
            lineClamp = 1;
        } else if (coverWidth < 120) {
            lineClamp = 2;
        } else if (coverWidth < 180) {
            lineClamp = 2;
        } else if (coverWidth < 220) {
            lineClamp = 3;
        } else {
            lineClamp = 3;
        }

        return { fontSize: Math.round(fontSize * 10) / 10, lineClamp };
    }

    setCoverSize(sizeInPixels) {
        const size = parseInt(sizeInPixels);
        this.coverSize = size;
        this.applyCoverSizeCSS();
        this.saveViewPreferences();
    }

    updateColumnVisibility() {
        const table = document.getElementById('booksTableElement');
        if (!table) return;

        const headers = table.querySelectorAll('th.resizable-col');
        const columns = ['title', 'authors', 'series', 'publisher', 'rating', 'pubdate'];

        // Hide/show headers
        headers.forEach((th, index) => {
            const colName = columns[index];
            th.style.display = this.columnVisibility[colName] ? 'table-cell' : 'none';
        });

        // Hide/show cells in rows
        const rows = table.querySelectorAll('tbody tr');
        rows.forEach(row => {
            const cells = row.querySelectorAll('td');
            cells.forEach((td, index) => {
                const colName = columns[index];
                td.style.display = this.columnVisibility[colName] ? 'table-cell' : 'none';
            });
        });

        // Recalculate visible column percentages
        this.recalculateColumnWidths();
    }

    recalculateColumnWidths() {
        // Recalculate percentages based on visible columns only
        const visibleColumns = Object.entries(this.columnVisibility)
            .filter(([_, visible]) => visible)
            .map(([col, _]) => col);

        const table = document.getElementById('booksTableElement');
        if (!table || visibleColumns.length === 0) return;

        const headers = table.querySelectorAll('th.resizable-col');
        const columns = ['title', 'authors', 'series', 'publisher', 'rating', 'pubdate'];

        // Default widths for columns
        const defaultWidths = {
            title: 30,
            authors: 20,
            series: 15,
            publisher: 15,
            rating: 10,
            pubdate: 10
        };

        // Ensure all visible columns have valid widths
        visibleColumns.forEach(colName => {
            if (!this.tableColumnWidths[colName] || this.tableColumnWidths[colName] <= 0) {
                this.tableColumnWidths[colName] = defaultWidths[colName] || 15;
            }
        });

        // Calculate new percentages distributed among visible columns
        let totalVisiblePercentage = 0;
        headers.forEach((th, index) => {
            if (th.style.display !== 'none') {
                totalVisiblePercentage += this.tableColumnWidths[columns[index]] || 0;
            }
        });

        // Redistribute percentages
        headers.forEach((th, index) => {
            const colName = columns[index];
            if (this.columnVisibility[colName]) {
                const newPercent = totalVisiblePercentage > 0
                    ? (this.tableColumnWidths[colName] / totalVisiblePercentage) * 100
                    : (100 / visibleColumns.length);
                th.style.width = newPercent + '%';
            }
        });
    }

    renderBooksTable() {
        const tbody = document.getElementById('booksTableBody');
        const table = document.getElementById('booksTableElement');
        if (!tbody || !table) return;

        // Clear tbody on initial render
        if (this.displayedBooksCount === 0) {
            tbody.innerHTML = '';
        }

        // Calculate which books to display
        const startIndex = this.displayedBooksCount;
        const endIndex = Math.min(this.displayedBooksCount + this.booksPerPage, this.filteredBooks.length);
        const booksToDisplay = this.filteredBooks.slice(startIndex, endIndex);

        // Add new rows to table
        booksToDisplay.forEach(book => {
            const row = document.createElement('tr');
            row.dataset.bookId = book.id;
            row.dataset.libraryId = this.currentLibraryId;
            if (this.selectedBookId === book.id && this.selectedBookLibraryId === this.currentLibraryId) {
                row.classList.add('selected');
            }

            row.innerHTML = `
                <td>${this.escapeHtml(book.title || '')}</td>
                <td>${this.escapeHtml((book.authors || []).map(author => this.formatAuthorName(author)).join(', '))}</td>
                <td>${this.escapeHtml(book.series && book.series_index ? `${book.series} #${book.series_index}` : (book.series || ''))}</td>
                <td>${this.escapeHtml((book.publisher || ''))}</td>
                <td>${book.rating ? (book.rating / 2).toFixed(1) : 'N/A'}</td>
                <td>${book.pubdate ? new Date(book.pubdate).toLocaleDateString() : 'N/A'}</td>
            `;

            row.addEventListener('click', (e) => {
                this.selectBook(book, this.currentLibraryId);
            });

            tbody.appendChild(row);
        });

        // Update displayed count
        this.displayedBooksCount = endIndex;
        this.tableRenderedForLibrary = this.currentLibraryId;

        // Load and apply saved column widths (only on first render)
        if (startIndex === 0) {
            this.loadTableColumnWidths();
            // Setup column resizing
            this.setupTableColumnResizing();
            // Setup resize observer for responsive column scaling
            this.setupTableResizeObserver();
        }

        // Apply column visibility
        this.updateColumnVisibility();

        // Setup infinite scroll on first render
        if (startIndex === 0 && this.displayedBooksCount === this.booksPerPage) {
            this.setupTableInfiniteScroll();
        }
    }

    setupTableInfiniteScroll() {
        const booksTable = document.getElementById('booksTable');
        if (!booksTable) return;

        // Remove existing listener if any
        booksTable.removeEventListener('scroll', this.handleTableScroll);

        // Add new scroll listener
        this.handleTableScroll = () => {
            if (this.isLoadingMore || this.displayedBooksCount >= this.filteredBooks.length) {
                return;
            }

            const scrollTop = booksTable.scrollTop;
            const scrollHeight = booksTable.scrollHeight;
            const clientHeight = booksTable.clientHeight;

            // Load more books when user scrolls to 80% of the way down
            if (scrollTop + clientHeight >= scrollHeight * 0.8) {
                this.isLoadingMore = true;
                setTimeout(() => {
                    this.renderBooksTable();
                    this.isLoadingMore = false;
                }, 100);
            }
        };

        booksTable.addEventListener('scroll', this.handleTableScroll);
    }

    setupTableColumnResizing() {
        const resizeHandles = document.querySelectorAll('.col-resize');

        resizeHandles.forEach((handle, index) => {
            handle.addEventListener('mousedown', (e) => {
                e.preventDefault();
                e.stopPropagation();

                const th = handle.closest('th');
                const table = document.getElementById('booksTableElement');
                const startX = e.clientX;
                const startWidth = th.offsetWidth;
                const startTableWidth = table.offsetWidth;
                const colName = th.getAttribute('data-col') || this.getColumnNameByIndex(index);

                const handleMouseMove = (e) => {
                    const deltaX = e.clientX - startX;
                    const newWidth = Math.max(50, startWidth + deltaX);
                    // Convert to percentage for responsive behavior
                    const percentage = (newWidth / startTableWidth) * 100;
                    th.style.width = percentage + '%';
                };

                const handleMouseUp = () => {
                    document.removeEventListener('mousemove', handleMouseMove);
                    document.removeEventListener('mouseup', handleMouseUp);

                    // Save new column widths as percentages
                    this.saveTableColumnWidths();
                };

                document.addEventListener('mousemove', handleMouseMove);
                document.addEventListener('mouseup', handleMouseUp);
            });
        });
    }

    getColumnNameByIndex(index) {
        const columns = ['title', 'authors', 'series', 'publisher', 'rating', 'pubdate'];
        return columns[index] || 'title';
    }

    saveTableColumnWidths() {
        const table = document.getElementById('booksTableElement');
        if (!table) return;

        const headers = table.querySelectorAll('th.resizable-col');
        const columns = ['title', 'authors', 'series', 'publisher', 'rating', 'pubdate'];

        headers.forEach((th, index) => {
            const colName = columns[index];
            const width = th.offsetWidth;
            const tableWidth = table.offsetWidth;
            const percentage = (width / tableWidth) * 100;
            this.tableColumnWidths[colName] = percentage;
        });

        this.saveViewPreferences();
    }

    loadTableColumnWidths() {
        const table = document.getElementById('booksTableElement');
        if (!table) return;

        const headers = table.querySelectorAll('th.resizable-col');
        const columns = ['title', 'authors', 'series', 'publisher', 'rating', 'pubdate'];

        // Default widths for columns
        const defaultWidths = {
            title: 30,
            authors: 20,
            series: 15,
            publisher: 15,
            rating: 10,
            pubdate: 10
        };

        headers.forEach((th, index) => {
            const colName = columns[index];
            let width = this.tableColumnWidths[colName];

            // Ensure width is valid
            if (!width || width <= 0) {
                width = defaultWidths[colName] || 15;
                this.tableColumnWidths[colName] = width;
            }

            // Always apply as percentage for responsive scaling
            th.style.width = width + '%';
        });
    }

    setupTableResizeObserver() {
        const booksTable = document.getElementById('booksTable');
        if (!booksTable) return;

        // Create resize observer to detect when panel is resized
        const resizeObserver = new ResizeObserver(() => {
            // Re-apply column widths when container is resized
            this.loadTableColumnWidths();
        });

        resizeObserver.observe(booksTable);
    }

    escapeHtml(text) {
        const map = {
            '&': '&amp;',
            '<': '&lt;',
            '>': '&gt;',
            '"': '&quot;',
            "'": '&#039;'
        };
        return text.replace(/[&<>"']/g, m => map[m]);
    }

    async refreshLibraries() {
        try {
            this.updateStatus('Refreshing libraries...');
            // Save current app state before refresh
            const savedState = this.loadAppState();

            const response = await fetch('/api/libraries/refresh', {
                method: 'POST'
            });
            const data = await response.json();

            if (data.success) {
                // Update libraries but preserve current state
                this.libraries = data.data || [];

                // Re-render libraries
                this.renderLibraries();

                // Recheck panel sizes on refresh
                this.adjustPanelsForViewport();

                // Restore previous selection if the library still exists
                if (savedState && savedState.currentLibraryId && this.libraries.some(lib => lib.id === savedState.currentLibraryId)) {
                    await this.selectLibrary(savedState.currentLibraryId);
                } else if (this.libraries.length > 0) {
                    await this.selectLibrary(this.libraries[0].id);
                }
                this.updateStatus('Libraries refreshed successfully');
            } else {
                console.error('Failed to refresh libraries:', data.error);
                this.updateStatus('Failed to refresh libraries: ' + (data.error || 'Unknown error'));
            }
        } catch (error) {
            console.error('Error refreshing libraries:', error);
            this.updateStatus('Error refreshing libraries');
        }
    }

    showAbout() {
        alert('Biblio - E-book Library Browser\n\nVersion 0.1.0\n\nA web-based application for browsing Calibre e-book libraries.\n\nBuilt with Rust (Actix-web) and modern web technologies');
    }

    updateStatus(message) {
        document.getElementById('statusMessage').textContent = message;
    }
}

// Initialize the application when the DOM is ready
const app = new BiblioApp();

document.addEventListener('DOMContentLoaded', () => {
    app.init();
});
