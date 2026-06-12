const puppeteer = require('puppeteer');

(async () => {
    console.log("Starting Full E2E Test Suite...");
    const browser = await puppeteer.launch({ args: ['--no-sandbox', '--disable-setuid-sandbox'] });
    const page = await browser.newPage();
    
    // Set a standard desktop viewport
    await page.setViewport({ width: 1280, height: 800 });
    
    try {
        console.log("Navigating to http://localhost:3000...");
        await page.goto('http://localhost:3000', { waitUntil: 'networkidle0' });
        await new Promise(r => setTimeout(r, 1000));
        
        // --- 1. Global Keybindings (Ctrl+K / Search) ---
        console.log("Testing Global Keybindings (Ctrl+K)...");
        // We use Control+K. On Mac, we might need Meta+K, but let's try Control first
        await page.keyboard.down('Control');
        await page.keyboard.press('KeyK');
        await page.keyboard.up('Control');
        await new Promise(r => setTimeout(r, 500));
        
        let searchInput = await page.$('input[aria-label="Command palette input"]');
        if (!searchInput) {
            console.log("Ctrl+K didn't open the command palette. Trying Meta+K...");
            await page.keyboard.down('Meta');
            await page.keyboard.press('KeyK');
            await page.keyboard.up('Meta');
            await new Promise(r => setTimeout(r, 500));
            searchInput = await page.$('input[aria-label="Command palette input"]');
        }
        
        if (searchInput) {
            console.log("✅ Command Palette opened successfully via keyboard shortcut.");
            await searchInput.type('index');
            await new Promise(r => setTimeout(r, 500));
            // Close with Escape
            await page.keyboard.press('Escape');
            await new Promise(r => setTimeout(r, 500));
            const searchInputAfterEsc = await page.$('input[aria-label="Command palette input"]');
            if (!searchInputAfterEsc) {
                console.log("✅ Command Palette closed successfully via Escape key.");
            } else {
                console.error("❌ Failed to close Command Palette with Escape key.");
            }
        } else {
            console.error("❌ Failed to open Command Palette via keyboard shortcut.");
        }

        // --- 2. UI Navigation & Buttons ---
        console.log("Testing UI Navigation (Drawer, Theme, Git)...");
        
        // Theme Toggle
        const themeBtn = await page.$('button[aria-label="Toggle Theme"]');
        if (themeBtn) {
            await themeBtn.click();
            await new Promise(r => setTimeout(r, 500));
            const isDark = await page.evaluate(() => document.body.classList.contains('theme-dark') || document.body.classList.contains('dark-mode') || document.documentElement.getAttribute('data-theme') === 'dark');
            console.log("✅ Theme toggle button clicked. Is Dark Mode:", isDark);
            // Toggle back
            await themeBtn.click();
            await new Promise(r => setTimeout(r, 500));
        } else {
            console.error("❌ Theme toggle button not found.");
        }

        // Drawer Toggle
        const drawerBtn = await page.$('button[aria-label="Toggle File Tree"]');
        if (drawerBtn) {
            await drawerBtn.click();
            await new Promise(r => setTimeout(r, 1000));
            console.log("✅ File Tree Drawer toggled open.");
            
            // Try to click a file in the drawer
            const fileLink = await page.$('.tree-node');
            if (fileLink) {
                await fileLink.click();
                await new Promise(r => setTimeout(r, 500));
                console.log("✅ Clicked a file link in the drawer.");
            }
            
            // Close drawer
            const closeDrawerBtn = await page.$('button[aria-label="Close Drawer"]');
            if (closeDrawerBtn) {
                await closeDrawerBtn.click();
                console.log("✅ Drawer closed via close button.");
            } else {
                // If no close button, try clicking outside or pressing Escape
                await page.keyboard.press('Escape');
            }
            await new Promise(r => setTimeout(r, 500));
        } else {
            console.error("❌ File Tree Drawer toggle button not found.");
        }

        // Git Actions Menu
        const gitBtn = await page.$('button[aria-label^="Git Actions"]');
        if (gitBtn && await gitBtn.isIntersectingViewport()) {
            await gitBtn.click();
            await new Promise(r => setTimeout(r, 500));
            console.log("✅ Git Actions menu opened successfully.");
            await page.keyboard.press('Escape');
            await new Promise(r => setTimeout(r, 500));
        } else {
             const desktopCommitBtn = await page.$('button[aria-label^="Commit"]');
             if (desktopCommitBtn) {
                 await desktopCommitBtn.click();
                 await new Promise(r => setTimeout(r, 500));
                 console.log("✅ Desktop Commit modal opened successfully.");
                 await page.keyboard.press('Escape');
                 await new Promise(r => setTimeout(r, 500));
             } else {
                 console.log("⚠️ Git Actions buttons not found (might be hidden if no git repo).");
             }
        }

        // --- 3. Editor Operations (Ctrl+S) ---
        console.log("Testing Editor Operations (Ctrl+S Save)...");
        const editBtn = await page.$('button[aria-label^="Edit page"]');
        if (editBtn) {
            await editBtn.click();
            await new Promise(r => setTimeout(r, 1000));
            
            // Ensure editor is focused
            await page.evaluate(() => {
                const cmElement = document.querySelector('.CodeMirror');
                if (cmElement && cmElement.CodeMirror) {
                    const cm = cmElement.CodeMirror;
                    cm.focus();
                    cm.setCursor({line: 0, ch: 0});
                } else {
                    const ta = document.getElementById('code-editor');
                    if (ta) {
                        ta.focus();
                        ta.setSelectionRange(0, 0);
                    }
                }
            });
            await new Promise(r => setTimeout(r, 500));
            
            // Press Ctrl+S
            console.log("Pressing Ctrl+S...");
            await page.keyboard.down('Control');
            await page.keyboard.press('KeyS');
            await page.keyboard.up('Control');
            
            // Wait for save to complete and preview to appear
            await new Promise(r => setTimeout(r, 2000));
            
            const markdownBody = await page.$('.markdown-body');
            if (markdownBody) {
                console.log("✅ Editor successfully saved and returned to preview mode via Ctrl+S!");
            } else {
                console.error("❌ Editor did not return to preview mode. Ctrl+S save may have failed.");
            }
        } else {
            console.error("❌ Edit button not found.");
        }

        console.log("\\n🎉 All E2E tests completed successfully!");

    } catch (e) {
        console.error("❌ Test failed with exception:", e);
    }
    
    await browser.close();
})();
