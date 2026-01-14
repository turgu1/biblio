# Cookie Session Persistence - Testing Guide

## Overview

Biblio now includes automatic session persistence using browser cookies. This guide will help you verify that the feature is working correctly.

## What Gets Saved

The following user preferences are automatically saved and restored:

1. **Current Library**: The last library you selected
2. **Active Filters**: 
   - Selected authors
   - Selected tags
   - Selected series
   - Selected formats
3. **Search Term**: Your search query text
4. **Sort Method**: Your chosen sort order (Recent, Title, Author)
5. **Displayed Books Count**: For pagination/scroll position restoration

## Cookie Details

- **Cookie Name**: `biblioAppState`
- **Expiration**: 30 days from last activity
- **Format**: JSON-encoded object
- **Storage**: Browser local storage (private to each domain)
- **Scope**: Page refresh and browser restart

## Testing Procedures

### Test 1: Basic Cookie Creation

1. Open the application in your browser
2. Open Developer Tools (F12 or right-click → Inspect)
3. Navigate to **Application** → **Cookies** → **http://localhost:8080**
4. You should see a cookie named `biblioAppState`
5. Click on it to view the JSON contents

**Expected Result**: Cookie exists and contains valid JSON structure

---

### Test 2: Filter Persistence

1. Select a library (if multiple are available)
2. Check a few author filter checkboxes
3. Check a few tag filter checkboxes
4. Verify the books displayed have changed
5. **Refresh the page** (F5 or Ctrl+R)
6. Verify that:
   - The same library is still selected
   - The same authors are still checked
   - The same tags are still checked
   - The filtered book list remains the same

**Expected Result**: All filters are restored after page refresh

---

### Test 3: Search Term Persistence

1. Enter a search term in the search box (e.g., "space", "mystery")
2. Verify that the book list filters based on your search
3. **Refresh the page** (F5 or Ctrl+R)
4. Verify that:
   - The search box still contains your search term
   - The book list shows the same filtered results

**Expected Result**: Search term and results are preserved after refresh

---

### Test 4: Sort Method Persistence

1. Change the sort method using the dropdown (select "Title" or "Author")
2. Observe the book list reorders
3. Note which book appears at the top of the list
4. **Refresh the page** (F5 or Ctrl+R)
5. Verify that:
   - The sort dropdown shows your selected sort method
   - The books are still sorted the same way
   - The same book is still at the top

**Expected Result**: Sort method is restored after refresh

---

### Test 5: Combined State Persistence

1. Select a library
2. Check 2-3 filter categories (authors AND tags AND series)
3. Enter a search term
4. Change the sort method
5. Select a book to view its details
6. **Refresh the page** (F5 or Ctrl+R)
7. Verify that:
   - The library is still selected
   - All filters are still checked
   - The search term is still in the search box
   - The sort method is still selected
   - Books are displayed with the filtered, searched, and sorted results

**Expected Result**: All state is restored comprehensively

---

### Test 6: Library Switching

1. If you have multiple libraries:
   - Select Library A
   - Apply filters specific to Library A (e.g., "Author A")
   - Switch to Library B
   - Apply different filters (e.g., "Author B")
   - Switch back to Library A
   - Verify that the original filters for Library A are restored
   - Switch to Library B and verify Library B's filters are restored

**Expected Result**: Each library maintains its own filter state

---

### Test 7: Browser Restart

1. Set up your preferred state:
   - Select a library
   - Select filters
   - Enter a search term
   - Change sort order
2. **Close the entire browser** (not just the tab)
3. **Reopen the browser and navigate back to http://localhost:8080**
4. Verify that:
   - Your library is selected
   - Your filters are applied
   - Your search term is in the box
   - Your sort order is selected

**Expected Result**: State persists across browser restart (as long as cookies haven't been cleared)

---

### Test 8: Cookie Expiration (Optional)

The cookie expires after 30 days of inactivity. To test:

1. Open Developer Tools → Application → Cookies
2. Click on the `biblioAppState` cookie
3. Check the "Expires/Max-Age" field
4. It should show a date 30 days from today

**Expected Result**: Expiration date is approximately 30 days in the future

---

## Troubleshooting

### Cookies Not Working?

**Check 1: Are cookies enabled in your browser?**
- Chrome/Edge: Settings → Privacy and security → Cookies → Ensure "Allow all cookies" is selected
- Firefox: Preferences → Privacy → Cookies → "Accept cookies and site data"
- Safari: Preferences → Privacy → "Cookies and website data" → Allow from "websites I visit"

**Check 2: Private/Incognito mode?**
- Cookies work in private mode but are cleared when the session ends
- Try testing in normal (non-private) mode

**Check 3: Check the browser console**
- Open Developer Tools (F12)
- Look at the Console tab for any error messages
- If you see errors, report them to the developer

### Filter State Not Restoring?

**Possible causes:**
1. You're testing with the same library selected but filters from a different library
2. The checkbox IDs don't match the saved state
3. Try clearing the cookie and starting fresh (Delete cookie in DevTools)

**To clear and reset:**
1. Open DevTools → Application → Cookies
2. Right-click the `biblioAppState` cookie and delete it
3. Refresh the page
4. Start setting your preferences fresh

---

## Cookie Inspector

To inspect the cookie contents manually:

1. Open DevTools → Application → Cookies → http://localhost:8080
2. Click on `biblioAppState`
3. You'll see the cookie value, which looks like:
```json
{
  "currentLibraryId": "12345678-1234-1234-1234-123456789012",
  "selectedAuthors": ["author1_id", "author2_id"],
  "selectedTags": ["tag1_id", "tag2_id"],
  "selectedSeries": ["series1_id"],
  "selectedFormats": ["EPUB", "PDF"],
  "searchTerm": "your search",
  "sortMethod": "recent",
  "displayedBooksCount": 50
}
```

This structure shows exactly what state is being persisted.

---

## Performance Notes

- Cookies are sent with every HTTP request, but our state cookie is small (~1KB)
- Saving state happens immediately when you change filters (not in background)
- Restoring state on page load is instantaneous
- No server-side processing required; all handled by browser

---

## Known Limitations

1. **Cookie Size**: Very large filter selections (100+ items) might hit browser cookie size limits
2. **Private Browsing**: Cookies in private mode are cleared at session end
3. **Disabled Cookies**: If user disables cookies, state won't persist (application still works, just without persistence)
4. **Cross-Domain**: Cookies are domain-specific; state won't transfer if domain/port changes

---

## Success Indicators

✅ **Feature is working correctly if:**
- The `biblioAppState` cookie exists in your browser
- Filters, search, and sort persist after page refresh
- State restores after browser restart
- Different libraries maintain separate filter states
- No console errors appear in DevTools

✅ **You should see:**
- Same book display after refresh
- Same filters checked after refresh
- Same search term visible after refresh
- Same sort order active after refresh

---

For questions or issues, check the main README.md or contact the development team.
